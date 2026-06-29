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
pub use parallel::{ParallelProcessor, ProcessResult, ParallelConfig, ProcessingStats, process_files_parallel};
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

/// Convenience function to extract thumbnail and write directly to a file
///
/// This combines `extract_thumbnail` and `std::fs::write` in one call.
///
/// # Arguments
///
/// * `input` - Path to the RAW file
/// * `output` - Path to save the thumbnail image
///
/// # Returns
///
/// Returns the number of bytes written on success
///
/// # Example
///
/// ```no_run
/// use rawlib::extract_thumbnail_to_file;
///
/// let bytes_written = extract_thumbnail_to_file("photo.cr2", "thumb.jpg").unwrap();
/// println!("Saved {} bytes", bytes_written);
/// ```
pub fn extract_thumbnail_to_file<P: AsRef<std::path::Path>, Q: AsRef<std::path::Path>>(
    input: P,
    output: Q,
) -> Result<usize, RawError> {
    let data = extract_thumbnail(input)?;
    std::fs::write(output.as_ref(), &data).map_err(|e| RawError {
        code: -1,
        message: format!("Failed to write thumbnail: {}", e),
    })?;
    Ok(data.len())
}
