//! Rust bindings for LibRaw - library for reading RAW image files
//!
//! This crate provides safe Rust bindings to the LibRaw C++ library,
//! which is used for reading RAW files from digital cameras.

#![allow(non_upper_case_globals)]
#![allow(non_camel_case_types)]
#![allow(non_snake_case)]

pub mod exif;
pub mod ffi;
pub mod parallel;
pub mod raw_processor;

// Public exports
pub use exif::{extract_exif, extract_exif_parallel, ExifData, ExifError};
pub use parallel::{
    process_files_parallel, ParallelConfig, ParallelProcessor, ProcessResult, ProcessingStats,
};
pub use raw_processor::{ImageFormat, RawError, RawProcessor, ThumbnailData};

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
pub fn extract_thumbnail_with_info<P: AsRef<std::path::Path>>(
    path: P,
) -> Result<ThumbnailData, RawError> {
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

/// Convenience function to extract full decoded RAW image data
///
/// This decodes the entire RAW file through the full pipeline:
/// open → unpack → demosaic/process → pixel data.
/// Returns raw RGB bitmap pixel data suitable for image viewers.
///
/// # Arguments
///
/// * `path` - Path to the RAW file
///
/// # Returns
///
/// Returns `ThumbnailData` with the decoded image (usually Bitmap format,
/// 3-channel RGB, 8 or 16 bits per channel).
///
/// # Example
///
/// ```no_run
/// use rawlib::extract_image;
///
/// let img = extract_image("photo.cr2").unwrap();
/// // img.data contains raw RGB pixel bytes
/// // img.width / img.height give dimensions
/// // img.colors = 3 (RGB), img.bits = 8 or 16
/// ```
pub fn extract_image<P: AsRef<std::path::Path>>(path: P) -> Result<ThumbnailData, RawError> {
    RawProcessor::extract_image(path)
}

/// Convenience function to extract full RAW image and write directly to a file
///
/// Decodes the RAW file and writes the resulting image data to disk.
/// The output format is raw bitmap data (not a standard image format like PNG/JPEG).
/// For a viewable file, pipe the data through an encoder or save as `.ppm`.
///
/// # Arguments
///
/// * `input` - Path to the RAW file
/// * `output` - Path to save the decoded image data
///
/// # Returns
///
/// Returns the number of bytes written on success
pub fn extract_image_to_file<P: AsRef<std::path::Path>, Q: AsRef<std::path::Path>>(
    input: P,
    output: Q,
) -> Result<usize, RawError> {
    let data = extract_image(input)?;
    std::fs::write(output.as_ref(), &data.data).map_err(|e| RawError {
        code: -1,
        message: format!("Failed to write image: {}", e),
    })?;
    Ok(data.data.len())
}
