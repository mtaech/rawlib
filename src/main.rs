use clap::Parser;
use log::{error, info};
use std::process;

mod cli;
mod error;
mod processor;
mod utils;

/// RAW 文件缩略图提取器 - 快速批量提取 RAW 图像中的嵌入缩略图
#[derive(Parser, Debug)]
#[command(
    name = "rawlib",
    version,
    author,
    about = "Extract thumbnails from RAW image files",
    long_about = "A fast and efficient tool to extract embedded thumbnails from RAW camera files.\n\
                  Supports batch processing, recursive directory scanning, and multiple output options.\n\n\
                  Supported formats: CR2, CR3, NEF, NRW, ARW, SRF, SR2, RAF, ORF, RW2, DNG, and more.",
    after_help = "EXAMPLES:\n    \
        rawlib photo.NEF                          # Extract to photo.jpg\n    \
        rawlib photo.NEF -o thumb.jpg             # Specify output\n    \
        rawlib ./photos/ -o ./thumbs/             # Batch process\n    \
        rawlib ./photos/ -r --progress            # Recursive with progress\n    \
        rawlib ./photos/ --overwrite rename       # Rename conflicts\n    \
        RUST_LOG=debug rawlib photo.NEF -v        # Debug logging"
)]
struct Cli {
    /// 输入文件或目录路径（支持多个）
    #[arg(
        value_name = "INPUT",
        required = true,
        help = "Input RAW file(s) or directory"
    )]
    inputs: Vec<String>,

    /// 输出目录或文件路径
    #[arg(
        short = 'o',
        long = "output",
        value_name = "PATH",
        help = "Output file or directory (default: same as input with .jpg extension)"
    )]
    output: Option<String>,

    /// 文件覆盖策略
    #[arg(
        long = "overwrite",
        default_value = "skip",
        value_parser = ["skip", "overwrite", "rename"],
        help = "File overwrite behavior"
    )]
    overwrite: String,

    /// 递归扫描目录
    #[arg(
        short = 'r',
        long = "recursive",
        help = "Recursively scan directories"
    )]
    recursive: bool,

    /// 详细输出模式
    #[arg(
        short = 'v',
        long = "verbose",
        help = "Show detailed information"
    )]
    verbose: bool,

    /// 静默模式
    #[arg(
        short = 'q',
        long = "quiet",
        conflicts_with = "verbose",
        help = "Suppress output except errors"
    )]
    quiet: bool,

    /// 显示进度条
    #[arg(
        long = "progress",
        help = "Show progress bar"
    )]
    progress: bool,

    /// 并行处理的工作线程数
    #[arg(
        short = 'j',
        long = "jobs",
        value_name = "N",
        help = "Number of parallel jobs (default: number of CPU cores)"
    )]
    jobs: Option<usize>,

    /// 输出文件格式
    #[arg(
        short = 'f',
        long = "format",
        default_value = "auto",
        value_parser = ["auto", "jpg", "jpeg", "bmp"],
        help = "Output format"
    )]
    format: String,

    /// 显示 EXIF 信息（不提取缩略图）
    #[arg(
        long = "exif",
        help = "Show EXIF metadata instead of extracting thumbnail"
    )]
    exif: bool,

    /// 以 JSON 格式输出 EXIF 信息
    #[arg(
        long = "json",
        help = "Output EXIF data in JSON format (requires --exif)"
    )]
    json: bool,

    /// RAW 文件扩展名过滤
    #[arg(
        long = "extensions",
        value_delimiter = ',',
        default_value = "cr2,cr3,nef,nrw,arw,srf,sr2,raf,orf,rw2,dng",
        help = "Comma-separated RAW file extensions"
    )]
    extensions: Vec<String>,
}

fn main() {
    // 初始化日志系统
    env_logger::Builder::from_default_env()
        .filter_level(log::LevelFilter::Info)
        .init();

    // 解析 CLI 参数
    let cli = Cli::parse();
    
    // 检查是否是 EXIF 模式
    if cli.exif {
        run_exif_mode(&cli);
        return;
    }
    
    info!("Starting rawlib thumbnail extractor v{}", env!("CARGO_PKG_VERSION"));
    
    // 转换为配置
    let config = match cli::Config::from_cli(cli) {
        Ok(config) => config,
        Err(e) => {
            error!("Configuration error: {}", e);
            eprintln!("错误: {}", e);
            process::exit(1);
        }
    };
    
    // 执行处理
    match cli::run(config) {
        Ok(stats) => {
            if !stats.quiet {
                stats.print_summary();
            }
            info!("Processing completed successfully");
            process::exit(0);
        }
        Err(e) => {
            error!("Processing failed: {}", e);
            eprintln!("错误: {}", e);
            process::exit(1);
        }
    }
}

/// Run EXIF extraction mode
fn run_exif_mode(cli: &Cli) {
    use rawlib::exif::{extract_exif, extract_exif_parallel};
    use serde_json::json;
    use cli::Verbosity;
    
    // Collect input files
    let verbosity = if cli.quiet {
        Verbosity::Quiet
    } else if cli.verbose {
        Verbosity::Verbose
    } else {
        Verbosity::Normal
    };
    let files = match cli::collect_input_files(&cli.inputs, cli.recursive, &cli.extensions, verbosity) {
        Ok(f) => f,
        Err(e) => {
            eprintln!("错误: {}", e);
            process::exit(1);
        }
    };
    
    if files.is_empty() {
        eprintln!("错误: 未找到 RAW 文件");
        process::exit(1);
    }
    
    let results = if files.len() == 1 || cli.jobs == Some(1) {
        // Sequential processing
        files.iter()
            .map(|f| (f.clone(), extract_exif(f)))
            .collect::<Vec<_>>()
    } else {
        // Parallel processing
        extract_exif_parallel(&files, cli.jobs)
    };
    
    if cli.json {
        let json_results: Vec<serde_json::Value> = results.iter().map(|(path, result)| {
            match result {
                Ok(exif) => build_exif_json(path, exif),
                Err(e) => json!({
                    "path": path.display().to_string(),
                    "error": e.to_string(),
                }),
            }
        }).collect();
        println!("{}", serde_json::to_string_pretty(&json_results).unwrap());
    } else {
        for (path, result) in results {
            match result {
                Ok(exif) => {
                    println!("\n{}:", path.display());
                    print_exif_human(&exif);
                }
                Err(e) => {
                    eprintln!("✗ {}: {}", path.display(), e);
                }
            }
        }
    }
}

/// Print EXIF data in human-readable format
fn print_exif_human(exif: &rawlib::exif::ExifData) {
    if let Some(ref make) = exif.make {
        println!("  相机厂商: {}", make);
    }
    if let Some(ref model) = exif.model {
        println!("  相机型号: {}", model);
    }
    if let Some(ref lens) = exif.lens_model {
        println!("  镜头型号: {}", lens);
    }
    if let Some(ref date) = exif.date_time_original {
        println!("  拍摄时间: {}", date);
    }
    if let Some(ref exp) = exif.exposure_time {
        println!("  快门速度: {}", exp);
    }
    if let Some(ref fnum) = exif.f_number {
        println!("  光圈: {}", fnum);
    }
    if let Some(iso) = exif.iso {
        println!("  ISO: {}", iso);
    }
    if let Some(ref focal) = exif.focal_length {
        println!("  焦距: {}", focal);
    }
    if let (Some(w), Some(h)) = (exif.image_width, exif.image_height) {
        println!("  图像尺寸: {}x{}", w, h);
    }
    if exif.has_gps() {
        if let Some((lat, lon)) = exif.gps_coordinates() {
            println!("  GPS: {:.6}, {:.6}", lat, lon);
        }
    }
}

/// Build EXIF data as a JSON Value (uses serde_json for proper escaping)
fn build_exif_json(path: &std::path::Path, exif: &rawlib::exif::ExifData) -> serde_json::Value {
    use serde_json::{json, Map, Value};
    
    let mut m = Map::new();
    m.insert("path".into(), json!(path.display().to_string()));
    
    if let Some(ref v) = exif.make { m.insert("make".into(), json!(v)); }
    if let Some(ref v) = exif.model { m.insert("model".into(), json!(v)); }
    if let Some(ref v) = exif.lens_model { m.insert("lens_model".into(), json!(v)); }
    if let Some(ref v) = exif.date_time_original { m.insert("date_time".into(), json!(v)); }
    if let Some(ref v) = exif.exposure_time { m.insert("exposure_time".into(), json!(v)); }
    if let Some(ref v) = exif.f_number { m.insert("f_number".into(), json!(v)); }
    if let Some(iso) = exif.iso { m.insert("iso".into(), json!(iso)); }
    if let Some(ref v) = exif.focal_length { m.insert("focal_length".into(), json!(v)); }
    if let (Some(w), Some(h)) = (exif.image_width, exif.image_height) {
        m.insert("width".into(), json!(w));
        m.insert("height".into(), json!(h));
    }
    if let Some((lat, lon)) = exif.gps_coordinates() {
        m.insert("gps_latitude".into(), json!(lat));
        m.insert("gps_longitude".into(), json!(lon));
    }
    
    Value::Object(m)
}



#[cfg(test)]
mod test{
    use rawlib::extract_thumbnail;
    use std::path::Path;
    
    #[test]
    #[ignore = "需要真实的 RAW 文件路径，适合手动运行验证"]
    pub fn img(){
        // 使用原始字符串字面量，避免转义问题
        let test_file = r"C:\Users\huang\图片\2025\2025-11-30\DSC_5432.NEF";
        
        // 先检查文件是否存在
        if !Path::new(test_file).exists() {
            eprintln!("警告: 测试文件不存在: {}", test_file);
            eprintln!("请将此测试文件路径替换为你本地实际存在的 RAW 文件路径");
            // 跳过测试而不是失败
            return;
        }
        
        let thumb_bytes = extract_thumbnail(test_file)
            .expect("Extract thumbnail failed.");

        // 保存到文件 (使用正确的扩展名，因为通常是 JPEG)
        std::fs::write("./img.jpg", &thumb_bytes).unwrap();
        
        println!("成功提取缩略图，大小: {} 字节", thumb_bytes.len());
    }
}