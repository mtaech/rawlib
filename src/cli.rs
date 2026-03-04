//! CLI configuration and file collection

use crate::error::{Error, Result};
use log::{debug, info, warn};
use std::path::{Path, PathBuf};
use walkdir::WalkDir;

/// 处理模式
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ProcessMode {
    /// 单文件处理
    SingleFile,
    /// 批量处理
    Batch,
}

/// 文件覆盖策略
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum OverwritePolicy {
    /// 跳过已存在的文件
    Skip,
    /// 覆盖已存在的文件
    Overwrite,
    /// 重命名（添加数字后缀）
    Rename,
}

impl OverwritePolicy {
    pub fn from_str(s: &str) -> Result<Self> {
        match s.to_lowercase().as_str() {
            "skip" => Ok(Self::Skip),
            "overwrite" => Ok(Self::Overwrite),
            "rename" => Ok(Self::Rename),
            _ => Err(Error::InvalidOverwritePolicy { 
                policy: s.to_string() 
            }),
        }
    }
}

/// 输出详细程度
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord)]
pub enum Verbosity {
    Quiet,
    Normal,
    Verbose,
}

/// 输出格式
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum OutputFormat {
    Auto,
    Jpeg,
    Bitmap,
}

impl OutputFormat {
    pub fn from_str(s: &str) -> Result<Self> {
        match s.to_lowercase().as_str() {
            "auto" => Ok(Self::Auto),
            "jpg" | "jpeg" => Ok(Self::Jpeg),
            "bmp" => Ok(Self::Bitmap),
            _ => Err(Error::InvalidOutputFormat { 
                format: s.to_string() 
            }),
        }
    }
    
    pub fn extension(&self) -> &str {
        match self {
            Self::Auto | Self::Jpeg => "jpg",
            Self::Bitmap => "bmp",
        }
    }
}

/// 程序配置
#[derive(Debug, Clone)]
pub struct Config {
    /// 输入文件列表（展开后的所有文件）
    pub input_files: Vec<PathBuf>,
    
    /// 输出目录或文件
    pub output: PathBuf,
    
    /// 处理模式
    pub mode: ProcessMode,
    
    /// 覆盖策略
    pub overwrite_policy: OverwritePolicy,
    
    /// 递归扫描
    pub recursive: bool,
    
    /// 输出详细程度
    pub verbosity: Verbosity,
    
    /// 显示进度条
    pub show_progress: bool,
    
    /// 输出格式
    pub format: OutputFormat,
    
    /// 允许的文件扩展名
    pub extensions: Vec<String>,
    
    /// 并行工作线程数 (None 表示使用 CPU 核心数)
    pub jobs: Option<usize>,
}

impl Config {
    /// 从 CLI 参数创建配置
    pub fn from_cli(cli: crate::Cli) -> Result<Self> {
        debug!("Parsing CLI arguments");
        
        // 确定详细程度
        let verbosity = if cli.quiet {
            Verbosity::Quiet
        } else if cli.verbose {
            Verbosity::Verbose
        } else {
            Verbosity::Normal
        };
        
        // 解析策略
        let overwrite_policy = OverwritePolicy::from_str(&cli.overwrite)?;
        let format = OutputFormat::from_str(&cli.format)?;
        
        // 收集输入文件
        let input_files = collect_input_files(
            &cli.inputs,
            cli.recursive,
            &cli.extensions,
            verbosity,
        )?;
        
        if input_files.is_empty() {
            return Err(Error::NoRawFiles);
        }
        
        info!("Found {} RAW file(s)", input_files.len());
        
        // 确定输出策略
        let (mode, output) = determine_output_strategy(
            &input_files,
            cli.output.as_deref(),
        )?;
        
        debug!("Process mode: {:?}, Output: {}", mode, output.display());
        
        Ok(Config {
            input_files,
            output,
            mode,
            overwrite_policy,
            recursive: cli.recursive,
            verbosity,
            show_progress: cli.progress,
            format,
            extensions: cli.extensions,
            jobs: cli.jobs,
        })
    }
}

/// 处理统计
#[derive(Debug, Default)]
pub struct ProcessStats {
    pub total: usize,
    pub success: usize,
    pub skipped: usize,
    pub failed: usize,
    pub quiet: bool,
    /// 处理开始时间
    pub start_time: Option<std::time::Instant>,
    /// 处理结束时间
    pub end_time: Option<std::time::Instant>,
    /// 总处理字节数（输入文件）
    pub total_input_bytes: u64,
    /// 总输出字节数（缩略图）
    pub total_output_bytes: u64,
}

impl ProcessStats {
    /// 开始计时
    pub fn start(&mut self) {
        self.start_time = Some(std::time::Instant::now());
    }
    
    /// 结束计时
    pub fn finish(&mut self) {
        self.end_time = Some(std::time::Instant::now());
    }
    
    /// 获取处理耗时
    pub fn elapsed(&self) -> Option<std::time::Duration> {
        match (self.start_time, self.end_time) {
            (Some(start), Some(end)) => Some(end.duration_since(start)),
            (Some(start), None) => Some(start.elapsed()),
            _ => None,
        }
    }
    
    /// 格式化字节大小为人类可读格式
    fn format_bytes(bytes: u64) -> String {
        const UNITS: &[&str] = &["B", "KB", "MB", "GB", "TB"];
        if bytes == 0 {
            return "0 B".to_string();
        }
        let exp = (bytes as f64).log(1024.0).min(UNITS.len() as f64 - 1.0) as usize;
        let value = bytes as f64 / 1024_f64.powi(exp as i32);
        if exp == 0 {
            format!("{} {}", bytes, UNITS[exp])
        } else {
            format!("{:.2} {}", value, UNITS[exp])
        }
    }
    
    /// 格式化耗时为人类可读格式
    fn format_duration(duration: std::time::Duration) -> String {
        let secs = duration.as_secs();
        let millis = duration.subsec_millis();
        
        if secs >= 60 {
            let mins = secs / 60;
            let remaining_secs = secs % 60;
            format!("{}分{}秒", mins, remaining_secs)
        } else if secs > 0 {
            format!("{}.{:03}秒", secs, millis)
        } else {
            format!("{}毫秒", millis)
        }
    }
    
    pub fn print_summary(&self) {
        let elapsed = self.elapsed();
        
        println!("\n========================================");
        println!("处理完成");
        println!("========================================");
        
        // 文件统计
        println!("📁 文件统计:");
        println!("   总计: {} 个文件", self.total);
        println!("   ✓ 成功: {} 个文件", self.success);
        if self.skipped > 0 {
            println!("   ⊘ 跳过: {} 个文件", self.skipped);
        }
        if self.failed > 0 {
            println!("   ✗ 失败: {} 个文件", self.failed);
        }
        
        // 时间统计
        if let Some(duration) = elapsed {
            println!("\n⏱️  时间统计:");
            println!("   总耗时: {}", Self::format_duration(duration));
            if self.total > 0 {
                let secs = duration.as_secs_f64();
                let files_per_sec = self.total as f64 / secs;
                let ms_per_file = secs * 1000.0 / self.total as f64;
                println!("   处理速度: {:.1} 文件/秒", files_per_sec);
                println!("   平均耗时: {:.1} 毫秒/文件", ms_per_file);
            }
        }
        
        // 大小统计
        if self.total_output_bytes > 0 {
            println!("\n💾 大小统计:");
            println!("   输出总大小: {}", Self::format_bytes(self.total_output_bytes));
            if self.total > 0 {
                let avg_size = self.total_output_bytes / self.total as u64;
                println!("   平均大小: {}/文件", Self::format_bytes(avg_size));
            }
            if self.total_input_bytes > 0 {
                let ratio = self.total_output_bytes as f64 / self.total_input_bytes as f64 * 100.0;
                println!("   压缩率: {:.1}%", ratio);
            }
        }
        
        println!("========================================");
    }
}

/// 主执行函数
pub fn run(config: Config) -> Result<ProcessStats> {
    use crate::processor::BatchProcessor;
    
    let processor = BatchProcessor::new(config);
    processor.process()
}

/// 收集输入文件
fn collect_input_files(
    inputs: &[String],
    recursive: bool,
    extensions: &[String],
    verbosity: Verbosity,
) -> Result<Vec<PathBuf>> {
    debug!("Collecting input files (recursive: {})", recursive);
    
    let mut files = Vec::new();
    let extensions_lower: Vec<String> = extensions
        .iter()
        .map(|e| e.to_lowercase())
        .collect();
    
    for input in inputs {
        let path = Path::new(input);
        
        if !path.exists() {
            warn!("Path does not exist: {}", input);
            if verbosity >= Verbosity::Normal {
                eprintln!("警告: 路径不存在: {}", input);
            }
            continue;
        }
        
        if path.is_file() {
            if is_raw_file(path, &extensions_lower) {
                debug!("Added file: {}", path.display());
                files.push(path.to_path_buf());
            } else if verbosity >= Verbosity::Normal {
                warn!("Skipping non-RAW file: {}", input);
                eprintln!("警告: 跳过非 RAW 文件: {}", input);
            }
        } else if path.is_dir() {
            let walker = if recursive {
                WalkDir::new(path).follow_links(false)
            } else {
                WalkDir::new(path).max_depth(1).follow_links(false)
            };
            
            for entry in walker.into_iter().filter_map(|e| e.ok()) {
                if entry.file_type().is_file() {
                    let file_path = entry.path();
                    if is_raw_file(file_path, &extensions_lower) {
                        debug!("Added file: {}", file_path.display());
                        files.push(file_path.to_path_buf());
                    }
                }
            }
        }
    }
    
    Ok(files)
}

/// 检查是否为 RAW 文件
fn is_raw_file(path: &Path, extensions: &[String]) -> bool {
    path.extension()
        .and_then(|ext| ext.to_str())
        .map(|ext| extensions.contains(&ext.to_lowercase()))
        .unwrap_or(false)
}

/// 确定输出策略
fn determine_output_strategy(
    input_files: &[PathBuf],
    output: Option<&str>,
) -> Result<(ProcessMode, PathBuf)> {
    match (input_files.len(), output) {
        // 单文件 + 指定输出 = 单文件模式
        (1, Some(out)) => {
            debug!("Single file mode with specified output");
            Ok((ProcessMode::SingleFile, PathBuf::from(out)))
        }
        
        // 单文件 + 无输出 = 同目录，替换扩展名
        (1, None) => {
            debug!("Single file mode with auto output");
            let input = &input_files[0];
            let output = input.with_extension("jpg");
            Ok((ProcessMode::SingleFile, output))
        }
        
        // 多文件 + 指定输出目录 = 批处理模式
        (_, Some(out)) => {
            debug!("Batch mode with specified output directory");
            Ok((ProcessMode::Batch, PathBuf::from(out)))
        }
        
        // 多文件 + 无输出 = 批处理模式，使用 "thumbnails" 目录
        (_, None) => {
            debug!("Batch mode with default output directory");
            Ok((ProcessMode::Batch, PathBuf::from("thumbnails")))
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    
    #[test]
    fn test_overwrite_policy_from_str() {
        assert!(matches!(
            OverwritePolicy::from_str("skip").unwrap(),
            OverwritePolicy::Skip
        ));
        assert!(matches!(
            OverwritePolicy::from_str("overwrite").unwrap(),
            OverwritePolicy::Overwrite
        ));
        assert!(matches!(
            OverwritePolicy::from_str("rename").unwrap(),
            OverwritePolicy::Rename
        ));
        assert!(OverwritePolicy::from_str("invalid").is_err());
    }
    
    #[test]
    fn test_output_format_from_str() {
        assert!(matches!(
            OutputFormat::from_str("auto").unwrap(),
            OutputFormat::Auto
        ));
        assert!(matches!(
            OutputFormat::from_str("jpg").unwrap(),
            OutputFormat::Jpeg
        ));
        assert!(matches!(
            OutputFormat::from_str("jpeg").unwrap(),
            OutputFormat::Jpeg
        ));
        assert!(matches!(
            OutputFormat::from_str("bmp").unwrap(),
            OutputFormat::Bitmap
        ));
        assert!(OutputFormat::from_str("invalid").is_err());
    }
    
    #[test]
    fn test_output_format_extension() {
        assert_eq!(OutputFormat::Auto.extension(), "jpg");
        assert_eq!(OutputFormat::Jpeg.extension(), "jpg");
        assert_eq!(OutputFormat::Bitmap.extension(), "bmp");
    }
}
