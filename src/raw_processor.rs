//! 高级安全的 LibRaw 包装器
//!
//! 这个模块提供了一个类型安全的 Rust API 来操作 LibRaw 库。
//! 它封装了底层的 C FFI 调用，提供了 Rust 风格的错误处理和内存管理。
//!
//! 主要功能：
//! - 安全的 LibRaw 实例管理（RAII 模式）
//! - 缩略图提取和处理
//! - 错误转换和处理
//! - 多平台文件名支持

use crate::ffi;
use std::ffi::CStr;
#[cfg(not(windows))]
use std::ffi::CString;
use std::fmt;
use std::path::Path;

/// LibRaw 操作的错误类型
///
/// 封装了 LibRaw C 库的错误代码，并提供用户友好的错误信息
#[derive(Debug, Clone)]
pub struct RawError {
    /// LibRaw 错误代码
    pub code: i32,
    /// 错误描述信息
    pub message: String,
}

impl fmt::Display for RawError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "LibRaw error {}: {}", self.code, self.message)
    }
}

impl std::error::Error for RawError {}

/// 模块级别的结果类型别名
pub type Result<T> = std::result::Result<T, RawError>;

/// 包含格式信息的缩略图数据结构
///
/// 这个结构体包含了从 RAW 文件中提取的缩略图的完整信息，
/// 包括图像格式、尺寸和原始数据。
#[derive(Debug, Clone)]
pub struct ThumbnailData {
    /// 图像格式（JPEG 或位图）
    pub format: ImageFormat,
    /// 缩略图宽度（像素）
    pub width: u16,
    /// 缩略图高度（像素）
    pub height: u16,
    /// 颜色通道数
    pub colors: u16,
    /// 每样本位数
    pub bits: u16,
    /// 原始图像数据字节数组
    pub data: Vec<u8>,
}

/// 图像格式枚举
///
/// 定义了 LibRaw 支持的输出图像格式
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ImageFormat {
    /// JPEG 压缩图像
    Jpeg,
    /// 未压缩的位图（RGB 数据）
    Bitmap,
    /// Unknown format
    Unknown(i32),
}

impl ImageFormat {
    fn from_code(code: i32) -> Self {
        match code {
            ffi::LIBRAW_IMAGE_JPEG => ImageFormat::Jpeg,
            ffi::LIBRAW_IMAGE_BITMAP => ImageFormat::Bitmap,
            _ => ImageFormat::Unknown(code),
        }
    }
    
    /// Get MIME type for the format
    pub fn mime_type(&self) -> &'static str {
        match self {
            ImageFormat::Jpeg => "image/jpeg",
            ImageFormat::Bitmap => "image/bmp",
            ImageFormat::Unknown(_) => "application/octet-stream",
        }
    }
}

/// RAW 图像文件的主要处理器
///
/// 这是 RawLib 库的核心结构体，提供了与 LibRaw 库交互的高级接口。
/// 它使用 RAII 模式管理 LibRaw 实例的生命周期，确保资源被正确释放。
///
/// # 安全性
/// - 使用 RAII 模式自动管理 LibRaw 实例
/// - 实现了 Send trait，支持跨线程使用
/// - 所有 FFI 调用都被包装在安全的接口中
pub struct RawProcessor {
    /// 指向 LibRaw 数据结构的不透明指针
    /// 注意：这个指针由 LibRaw 管理，不应该被直接操作
    data: *mut ffi::libraw_data_t,
}

impl RawProcessor {
    /// 创建新的 RawProcessor 实例
    ///
    /// 初始化一个 LibRaw 实例并准备处理 RAW 文件。
    /// 如果初始化失败，将返回错误。
    ///
    /// # 返回值
    /// `Ok(RawProcessor)` - 成功创建的处理器实例
    /// `Err(RawError)` - 初始化失败时的错误信息
    ///
    /// # 示例
    /// ```no_run
    /// use rawlib::RawProcessor;
    /// 
    /// let processor = RawProcessor::new()?;
    /// # Ok::<(), rawlib::RawError>(())
    /// ```
    pub fn new() -> Result<Self> {
        // 调用 LibRaw C API 初始化实例
        let data = unsafe { ffi::libraw_init(ffi::LIBRAW_OPTIONS_NONE) };

        if data.is_null() {
            return Err(RawError {
                code: -1,
                message: "Failed to initialize LibRaw".to_string(),
            });
        }

        Ok(RawProcessor { data })
    }
    
    /// 从文件系统打开 RAW 文件
    ///
    /// 打开指定的 RAW 文件并读取其基本信息。这个方法会：
    /// 1. 检查文件是否存在
    /// 2. 根据平台选择合适的 API（Windows 使用宽字符，Unix 使用 UTF-8）
    /// 3. 调用 LibRaw 打开文件
    ///
    /// # 参数
    /// * `path` - RAW 文件的路径
    ///
    /// # 返回值
    /// `Ok(())` - 文件成功打开
    /// `Err(RawError)` - 打开失败时的错误信息
    ///
    /// # 平台支持
    /// - Windows: 使用宽字符 API 支持完整的 Unicode 文件名
    /// - Unix/Linux/macOS: 使用 UTF-8 字符串
    pub fn open_file<P: AsRef<Path>>(&mut self, path: P) -> Result<()> {
        let path_ref = path.as_ref();

        // 验证文件是否存在
        if !path_ref.exists() {
            return Err(RawError {
                code: -1,
                message: format!("File does not exist: {}", path_ref.display()),
            });
        }

        // Windows 平台：使用宽字符 API 以获得更好的 Unicode 支持
        #[cfg(windows)]
        {
            use std::os::windows::ffi::OsStrExt;
            // 将路径转换为 UTF-16 宽字符字符串，并以 null 结尾
            let wide: Vec<u16> = path_ref
                .as_os_str()
                .encode_wide()
                .chain(std::iter::once(0))
                .collect();

            // 调用 LibRaw 的宽字符 API
            let ret = unsafe { ffi::libraw_open_wfile(self.data, wide.as_ptr()) };

            if ret != ffi::LIBRAW_SUCCESS {
                let mut err = self.make_error(ret);
                err.message = format!("{} (file: {})", err.message, path_ref.display());
                return Err(err);
            }
        }

        // Unix 平台：使用常规的 UTF-8 路径
        #[cfg(not(windows))]
        {
            // 确保 Path 可以转换为有效的 UTF-8 字符串
            let path_str = path_ref.to_str().ok_or_else(|| RawError {
                code: -1,
                message: format!("Invalid path encoding: {}", path_ref.display()),
            })?;
            
            let c_path = CString::new(path_str).map_err(|e| RawError {
                code: -1,
                message: format!("Path contains null byte: {} (error: {})", path_str, e),
            })?;
            
            let ret = unsafe { ffi::libraw_open_file(self.data, c_path.as_ptr()) };
            
            if ret != ffi::LIBRAW_SUCCESS {
                let mut err = self.make_error(ret);
                err.message = format!("{} (file: {})", err.message, path_str);
                return Err(err);
            }
        }
        
        Ok(())
    }
    
    /// Unpack the RAW data
    pub fn unpack(&mut self) -> Result<()> {
        let ret = unsafe { ffi::libraw_unpack(self.data) };
        
        if ret != ffi::LIBRAW_SUCCESS {
            return Err(self.make_error(ret));
        }
        
        Ok(())
    }
    
    /// Process the RAW data (demosaic, white balance, etc.)
    pub fn dcraw_process(&mut self) -> Result<()> {
        let ret = unsafe { ffi::libraw_dcraw_process(self.data) };
        
        if ret != ffi::LIBRAW_SUCCESS {
            return Err(self.make_error(ret));
        }
        
        Ok(())
    }
    
    /// Recycle internal buffers
    pub fn recycle(&mut self) {
        unsafe { ffi::libraw_recycle(self.data) };
    }
    
    /// Unpack thumbnail data
    pub fn unpack_thumb(&mut self) -> Result<()> {
        let ret = unsafe { ffi::libraw_unpack_thumb(self.data) };
        
        if ret != ffi::LIBRAW_SUCCESS {
            return Err(self.make_error(ret));
        }
        
        Ok(())
    }
    
    /// Extract thumbnail as raw bytes
    /// 
    /// This method opens the RAW file, extracts the embedded thumbnail,
    /// and returns it as a byte vector. The thumbnail is typically in JPEG format.
    /// 
    /// # Arguments
    /// 
    /// * `path` - Path to the RAW file
    /// 
    /// # Returns
    /// 
    /// Returns `ThumbnailData` containing the thumbnail image data and metadata
    /// 
    /// # Example
    /// 
    /// ```no_run
    /// use rawlib::RawProcessor;
    /// 
    /// let thumb_data = RawProcessor::extract_thumbnail("image.cr2").unwrap();
    /// std::fs::write("thumb.jpg", &thumb_data.data).unwrap();
    /// ```
    ///
    /// 这是提取缩略图的最简单方法，它处理了整个流程：
    /// 1. 创建新的处理器实例
    /// 2. 打开 RAW 文件
    /// 3. 解包缩略图数据
    /// 4. 获取处理后的缩略图数据
    pub fn extract_thumbnail<P: AsRef<Path>>(path: P) -> Result<ThumbnailData> {
        let mut processor = RawProcessor::new()?;
        processor.open_file(path)?;
        processor.unpack_thumb()?;
        processor.get_thumbnail()
    }

    /// 从已打开和解包的文件中获取缩略图数据
    ///
    /// 这个方法假设：
    /// - 文件已经被 `open_file()` 打开
    /// - 缩略图数据已经被 `unpack_thumb()` 解包
    ///
    /// # 返回值
    /// `Ok(ThumbnailData)` - 包含格式、尺寸和原始数据的缩略图信息
    /// `Err(RawError)` - 获取缩略图失败时的错误信息
    pub fn get_thumbnail(&self) -> Result<ThumbnailData> {
        let mut errc: i32 = 0;

        // 调用 LibRaw 创建内存中的缩略图图像
        let img_ptr = unsafe {
            ffi::libraw_dcraw_make_mem_thumb(self.data, &mut errc as *mut i32)
        };
        
        if img_ptr.is_null() {
            return Err(self.make_error(errc));
        }
        
        // SAFETY: img_ptr is valid and we'll copy the data before freeing
        let thumbnail = unsafe {
            let img = &*img_ptr;
            let format = ImageFormat::from_code(img.image_type);
            
            // Calculate the actual data size
            let data_size = img.data_size as usize;
            
            // Copy the image data
            // SAFETY: data field is a flexible array member, 
            // actual size is data_size bytes
            let data_ptr = img.data.as_ptr();
            let mut data = Vec::with_capacity(data_size);
            // SAFETY: data 的容量为 data_size，紧随其后会被 copy_nonoverlapping 完全填充
            data.set_len(data_size);
            std::ptr::copy_nonoverlapping(data_ptr, data.as_mut_ptr(), data_size);
            
            ThumbnailData {
                format,
                width: img.width,
                height: img.height,
                colors: img.colors,
                bits: img.bits,
                data,
            }
        };
        
        // Free the LibRaw allocated memory
        unsafe {
            ffi::libraw_dcraw_clear_mem(img_ptr);
        }
        
        Ok(thumbnail)
    }
    
    /// 获取 LibRaw 版本字符串
    ///
    /// 返回当前链接的 LibRaw 库的版本信息字符串，例如 "0.21.4-Release"。
    /// 这个方法对于调试和兼容性检查很有用。
    ///
    /// # 返回值
    /// 包含版本信息的字符串
    pub fn version() -> String {
        unsafe {
            // 调用 LibRaw 获取版本字符串
            let ver = ffi::libraw_version();
            CStr::from_ptr(ver)
                .to_string_lossy()
                .into_owned()
        }
    }

    /// 获取 LibRaw 版本号（整数格式）
    ///
    /// 返回 LibRaw 库的数值版本号，例如 5380（对应 0.21.4）。
    /// 版本号编码格式为：major * 10000 + minor * 100 + patch
    ///
    /// # 返回值
    /// 整数格式的版本号
    pub fn version_number() -> i32 {
        unsafe { ffi::libraw_versionNumber() }
    }
    
    /// 根据 LibRaw 错误代码创建错误信息
    ///
    /// 调用 LibRaw 的错误字符串函数获取用户友好的错误描述
    fn make_error(&self, code: i32) -> RawError {
        let msg = unsafe {
            // 获取 LibRaw 提供的错误描述字符串
            let err_str = ffi::libraw_strerror(code);
            CStr::from_ptr(err_str)
                .to_string_lossy()
                .into_owned()
        };

        RawError {
            code,
            message: msg,
        }
    }
}

/// RAII 资源清理实现
///
/// 当 RawProcessor 实例离开作用域时，自动释放 LibRaw 分配的资源。
/// 这确保了即使在发生错误时也不会出现内存泄漏。
impl Drop for RawProcessor {
    fn drop(&mut self) {
        if !self.data.is_null() {
            // 调用 LibRaw 的清理函数释放所有资源
            unsafe { ffi::libraw_close(self.data) };
        }
    }
}

/// 线程安全实现
///
/// LibRaw 本身不是线程安全的，但是 RawProcessor 实例可以安全地
/// 在不同线程之间传递（只要不在多个线程中同时使用同一个实例）。
/// Send trait 允许我们将 RawProcessor 实例发送到其他线程。
unsafe impl Send for RawProcessor {}
