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
}

impl ProcessStats {
    pub fn print_summary(&self) {
        println!("\n========================================");
        println!("处理完成");
        println!("========================================");
        println!("  总计: {}", self.total);
        println!("  ✓ 成功: {}", self.success);
        if self.skipped > 0 {
            println!("  ⊘ 跳过: {}", self.skipped);
        }
        if self.failed > 0 {
            println!("  ✗ 失败: {}", self.failed);
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
