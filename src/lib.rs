//! Rust bindings for LibRaw - library for reading RAW image files
//!
//! This crate provides safe Rust bindings to the LibRaw C++ library,
//! which is used for reading RAW files from digital cameras.

#![allow(non_upper_case_globals)]
#![allow(non_camel_case_types)]
#![allow(non_snake_case)]

pub mod ffi;
pub mod raw_processor;
pub mod parallel;
pub mod exif;

// Public exports
pub use raw_processor::{RawProcessor, ThumbnailData, ImageFormat, RawError};
pub use parallel::{ParallelProcessor, ProcessResult, ParallelConfig};
pub use exif::{ExifData, ExifError, extract_exif, extract_exif_parallel};

#[cfg(test)]
mod tests {
    #[test]
    fn it_works() {
        // Basic smoke test
        assert_eq!(2 + 2, 4);
    }
}

/// Convenience function to extract thumbnail from a RAW file path
/// 
/// This is a shorthand for `RawProcessor::extract_thumbnail(path)`
/// 
/// # Arguments
/// 
/// * `path` - Path to the RAW file
/// 
/// # Returns
/// 
/// Returns the thumbnail image data as a byte vector (typically JPEG format)
/// 
/// # Example
/// 
/// ```no_run
/// use rawlib::extract_thumbnail;
/// 
/// let thumb_bytes = extract_thumbnail("photo.cr2").unwrap();
/// std::fs::write("thumbnail.jpg", &thumb_bytes).unwrap();
/// ```
pub fn extract_thumbnail<P: AsRef<std::path::Path>>(path: P) -> Result<Vec<u8>, RawError> {
    let thumb_data = RawProcessor::extract_thumbnail(path)?;
    Ok(thumb_data.data)
}

/// Convenience function to extract thumbnail with metadata
/// 
/// Similar to `extract_thumbnail` but returns full `ThumbnailData` 
/// with format information and dimensions
/// 
/// # Arguments
/// 
/// * `path` - Path to the RAW file
/// 
/// # Returns
/// 
/// Returns `ThumbnailData` with image format, dimensions, and raw bytes
/// 
/// # Example
/// 
/// ```no_run
/// use rawlib::extract_thumbnail_with_info;
/// 
/// let thumb = extract_thumbnail_with_info("photo.nef").unwrap();
/// println!("Format: {:?}, Size: {}x{}", thumb.format, thumb.width, thumb.height);
/// std::fs::write("thumbnail.jpg", &thumb.data).unwrap();
/// ```
pub fn extract_thumbnail_with_info<P: AsRef<std::path::Path>>(path: P) -> Result<ThumbnailData, RawError> {
    RawProcessor::extract_thumbnail(path)
}
