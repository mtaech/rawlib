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


#[cfg(test)]
mod test{
    use rawlib::extract_thumbnail;
    use std::path::Path;
    
    #[test]
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