//! Error types for rawlib

use thiserror::Error;
use std::path::PathBuf;

/// 程序错误类型
#[derive(Error, Debug)]
pub enum Error {
    /// LibRaw 处理错误
    #[error("LibRaw error: {0}")]
    LibRaw(#[from] rawlib::RawError),
    
    /// IO 错误
    #[error("IO error: {0}")]
    Io(#[from] std::io::Error),
    
    /// 文件不存在
    #[error("File not found: {path}")]
    FileNotFound { path: PathBuf },
    
    /// 无效的文件扩展名
    #[error("Invalid file extension: {path}")]
    InvalidExtension { path: PathBuf },
    
    /// 没有找到 RAW 文件
    #[error("No RAW files found")]
    NoRawFiles,
    
    /// 无效的覆盖策略
    #[error("Invalid overwrite policy: {policy}")]
    InvalidOverwritePolicy { policy: String },
    
    /// 无效的输出格式
    #[error("Invalid output format: {format}")]
    InvalidOutputFormat { format: String },
    
    /// 无法创建输出文件（重命名失败）
    #[error("Could not create output file after {attempts} attempts: {path}")]
    CannotCreateOutput { path: PathBuf, attempts: u32 },
    
    /// 配置错误
    #[error("Configuration error: {message}")]
    Config { message: String },
}

pub type Result<T> = std::result::Result<T, Error>;
