//! High-level safe wrapper around LibRaw

use crate::ffi;
use std::ffi::CStr;
#[cfg(not(windows))]
use std::ffi::CString;
use std::fmt;
use std::path::Path;

/// Error type for LibRaw operations
#[derive(Debug, Clone)]
pub struct RawError {
    code: i32,
    message: String,
}

impl fmt::Display for RawError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "LibRaw error {}: {}", self.code, self.message)
    }
}

impl std::error::Error for RawError {}

pub type Result<T> = std::result::Result<T, RawError>;

/// Thumbnail data with format information
#[derive(Debug, Clone)]
pub struct ThumbnailData {
    /// Image format (JPEG or Bitmap)
    pub format: ImageFormat,
    /// Width of the thumbnail
    pub width: u16,
    /// Height of the thumbnail
    pub height: u16,
    /// Number of color channels
    pub colors: u16,
    /// Bits per sample
    pub bits: u16,
    /// Raw image data bytes
    pub data: Vec<u8>,
}

/// Image format enumeration
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ImageFormat {
    /// JPEG compressed image
    Jpeg,
    /// Uncompressed bitmap
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

/// Main processor for working with RAW image files
pub struct RawProcessor {
    data: *mut ffi::libraw_data_t,
}

impl RawProcessor {
    /// Create a new RawProcessor instance
    pub fn new() -> Result<Self> {
        let data = unsafe { ffi::libraw_init(ffi::LIBRAW_OPTIONS_NONE) };
        
        if data.is_null() {
            return Err(RawError {
                code: -1,
                message: "Failed to initialize LibRaw".to_string(),
            });
        }
        
        Ok(RawProcessor { data })
    }
    
    /// Open a RAW file from the filesystem
    pub fn open_file<P: AsRef<Path>>(&mut self, path: P) -> Result<()> {
        let path_ref = path.as_ref();
        
        // Check if file exists
        if !path_ref.exists() {
            return Err(RawError {
                code: -1,
                message: format!("File does not exist: {}", path_ref.display()),
            });
        }
        
        // On Windows, use wide character API for better Unicode support
        #[cfg(windows)]
        {
            use std::os::windows::ffi::OsStrExt;
            let wide: Vec<u16> = path_ref
                .as_os_str()
                .encode_wide()
                .chain(std::iter::once(0))
                .collect();
            
            let ret = unsafe { ffi::libraw_open_wfile(self.data, wide.as_ptr()) };
            
            if ret != ffi::LIBRAW_SUCCESS {
                let mut err = self.make_error(ret);
                err.message = format!("{} (file: {})", err.message, path_ref.display());
                return Err(err);
            }
        }
        
        // On Unix, use regular UTF-8 path
        #[cfg(not(windows))]
        {
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
    pub fn extract_thumbnail<P: AsRef<Path>>(path: P) -> Result<ThumbnailData> {
        let mut processor = RawProcessor::new()?;
        processor.open_file(path)?;
        processor.unpack_thumb()?;
        processor.get_thumbnail()
    }
    
    /// Get the thumbnail data from an already opened and unpacked file
    pub fn get_thumbnail(&self) -> Result<ThumbnailData> {
        let mut errc: i32 = 0;
        
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
            let mut data = vec![0u8; data_size];
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
    
    /// Get LibRaw version string
    pub fn version() -> String {
        unsafe {
            let ver = ffi::libraw_version();
            CStr::from_ptr(ver)
                .to_string_lossy()
                .into_owned()
        }
    }
    
    /// Get LibRaw version number
    pub fn version_number() -> i32 {
        unsafe { ffi::libraw_versionNumber() }
    }
    
    fn make_error(&self, code: i32) -> RawError {
        let msg = unsafe {
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

impl Drop for RawProcessor {
    fn drop(&mut self) {
        if !self.data.is_null() {
            unsafe { ffi::libraw_close(self.data) };
        }
    }
}

unsafe impl Send for RawProcessor {}

impl Default for RawProcessor {
    fn default() -> Self {
        Self::new().expect("Failed to create RawProcessor")
    }
}
