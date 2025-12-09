//! Foreign Function Interface bindings to LibRaw C API

use libc::{c_char, c_int, c_ushort, c_uchar};

#[cfg(windows)]
use std::os::raw::c_ushort as wchar_t;

// Opaque pointer to libraw_data_t structure
pub enum libraw_data_t {}

// LibRaw processed image structure
#[repr(C)]
pub struct libraw_processed_image_t {
    pub image_type: c_int,      // Image format (JPEG, bitmap, etc.)
    pub height: c_ushort,
    pub width: c_ushort,
    pub colors: c_ushort,
    pub bits: c_ushort,
    pub data_size: u32,
    pub data: [c_uchar; 1],     // Flexible array member
}

#[link(name = "raw", kind = "static")]
extern "C" {
    // Library version
    pub fn libraw_version() -> *const c_char;
    pub fn libraw_versionNumber() -> c_int;
    
    // Constructor/Destructor
    pub fn libraw_init(flags: c_int) -> *mut libraw_data_t;
    pub fn libraw_close(data: *mut libraw_data_t);
    
    // File operations
    pub fn libraw_open_file(data: *mut libraw_data_t, filename: *const c_char) -> c_int;
    
    #[cfg(windows)]
    pub fn libraw_open_wfile(data: *mut libraw_data_t, filename: *const wchar_t) -> c_int;
    
    pub fn libraw_unpack(data: *mut libraw_data_t) -> c_int;
    pub fn libraw_dcraw_process(data: *mut libraw_data_t) -> c_int;
    
    // Thumbnail operations
    pub fn libraw_unpack_thumb(data: *mut libraw_data_t) -> c_int;
    pub fn libraw_dcraw_make_mem_thumb(data: *mut libraw_data_t, errc: *mut c_int) -> *mut libraw_processed_image_t;
    pub fn libraw_dcraw_clear_mem(img: *mut libraw_processed_image_t);
    
    // Error handling
    pub fn libraw_strerror(error_code: c_int) -> *const c_char;
    
    // Memory management
    pub fn libraw_recycle(data: *mut libraw_data_t);
}

// LibRaw initialization flags
pub const LIBRAW_OPTIONS_NONE: c_int = 0;
pub const LIBRAW_OPIONS_NO_MEMERR_CALLBACK: c_int = 1;
pub const LIBRAW_OPIONS_NO_DATAERR_CALLBACK: c_int = 1 << 1;

// Return codes
pub const LIBRAW_SUCCESS: c_int = 0;
pub const LIBRAW_UNSPECIFIED_ERROR: c_int = -1;
pub const LIBRAW_FILE_UNSUPPORTED: c_int = -2;
pub const LIBRAW_REQUEST_FOR_NONEXISTENT_IMAGE: c_int = -3;
pub const LIBRAW_OUT_OF_ORDER_CALL: c_int = -4;
pub const LIBRAW_NO_THUMBNAIL: c_int = -5;
pub const LIBRAW_UNSUPPORTED_THUMBNAIL: c_int = -6;
pub const LIBRAW_INPUT_CLOSED: c_int = -7;
pub const LIBRAW_INSUFFICIENT_MEMORY: c_int = -100;
pub const LIBRAW_DATA_ERROR: c_int = -101;
pub const LIBRAW_IO_ERROR: c_int = -102;
pub const LIBRAW_CANCELLED_BY_CALLBACK: c_int = -103;
pub const LIBRAW_BAD_CROP: c_int = -104;
pub const LIBRAW_TOO_BIG: c_int = -105;
pub const LIBRAW_MEMPOOL_OVERFLOW: c_int = -106;

// Image types
pub const LIBRAW_IMAGE_JPEG: c_int = 1;
pub const LIBRAW_IMAGE_BITMAP: c_int = 2;
