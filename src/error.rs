//! RawLib 应用程序的错误类型定义
//!
//! 这个模块定义了整个 RawLib 应用程序使用的错误类型体系。
//! 它使用 thiserror 库来创建结构化的、用户友好的错误信息。
//!
//! 错误层次结构：
//! - 应用程序级别错误（这个模块的 Error 枚举）
//!   - LibRaw 库错误（rawlib::RawError）
//!   - 文件系统错误（IO、文件不存在等）
//!   - 用户输入错误（无效格式、策略等）
//!   - 配置错误

use thiserror::Error;
use std::path::PathBuf;

/// RawLib 应用程序的统一错误类型
///
/// 这个枚举包含了应用程序可能遇到的所有错误情况，
/// 从底层 LibRaw 错误到用户输入验证错误。
///
/// 使用 `#[error]` 属性宏自动生成 Display 实现，
/// 提供用户友好的错误信息。
#[derive(Error, Debug)]
pub enum Error {
    /// LibRaw 处理错误
    ///
    /// 当 LibRaw 库处理 RAW 文件时遇到问题，如文件格式不支持、
    /// 文件损坏、内存不足等。这个错误会自动转换底层 RawError。
    #[error("LibRaw error: {0}")]
    LibRaw(#[from] rawlib::RawError),

    /// IO 错误
    ///
    /// 封装标准库的 IO 错误，包括文件读写权限问题、磁盘空间不足等。
    #[error("IO error: {0}")]
    Io(#[from] std::io::Error),

    /// 文件不存在错误
    ///
    /// 当指定的文件路径不存在时返回。
    /// 包含具体的文件路径信息以便调试。
    #[error("File not found: {path}")]
    FileNotFound { path: PathBuf },

    /// 无效的文件扩展名错误
    ///
    /// 当文件的扩展名不在支持的 RAW 格式列表中时返回。
    /// 这通常用于用户输入验证。
    #[error("Invalid file extension: {path}")]
    InvalidExtension { path: PathBuf },

    /// 没有找到 RAW 文件
    ///
    /// 在批量处理模式中，当指定的目录中没有找到任何 RAW 文件时返回。
    #[error("No RAW files found")]
    NoRawFiles,

    /// 无效的覆盖策略错误
    ///
    /// 当用户提供的覆盖策略参数无效时返回。
    /// 有效值包括：skip、overwrite、rename。
    #[error("Invalid overwrite policy: {policy}")]
    InvalidOverwritePolicy { policy: String },

    /// 无效的输出格式错误
    ///
    /// 当用户指定的输出格式不被支持时返回。
    /// 有效值包括：auto、jpg、jpeg、bmp。
    #[error("Invalid output format: {format}")]
    InvalidOutputFormat { format: String },

    /// 无法创建输出文件错误
    ///
    /// 当自动重命名功能无法创建唯一的输出文件名时返回。
    /// 包含尝试的次数和目标路径。
    #[error("Could not create output file after {attempts} attempts: {path}")]
    CannotCreateOutput { path: PathBuf, attempts: u32 },

    /// 配置错误
    ///
    /// 用于各种配置相关的错误，如参数验证失败、
    /// 不兼容的设置组合等。
    #[error("Configuration error: {message}")]
    Config { message: String },
}

/// 应用程序级别结果类型别名
///
/// 这个别名使得函数返回类型更加简洁，统一使用 Error 类型。
pub type Result<T> = std::result::Result<T, Error>;
