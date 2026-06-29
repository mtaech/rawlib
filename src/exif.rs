//! EXIF metadata extraction support
//!
//! This module provides EXIF metadata extraction from RAW files,
//! including camera information, shooting parameters, and timestamps.

use std::path::Path;
use std::collections::HashMap;

// Re-export exif crate types
use exif::{Reader, Tag, Value};

use serde::Serialize;

/// EXIF data container
#[derive(Debug, Clone, Default, Serialize)]
pub struct ExifData {
    /// Camera make (e.g., "NIKON CORPORATION")
    pub make: Option<String>,
    /// Camera model (e.g., "NIKON D850")
    pub model: Option<String>,
    /// Lens model
    pub lens_model: Option<String>,
    /// Date and time original
    pub date_time_original: Option<String>,
    /// Exposure time (e.g., "1/250")
    pub exposure_time: Option<String>,
    /// F-number (e.g., "f/2.8")
    pub f_number: Option<String>,
    /// ISO speed rating
    pub iso: Option<u32>,
    /// Focal length (e.g., "50.0 mm")
    pub focal_length: Option<String>,
    /// Image width
    pub image_width: Option<u32>,
    /// Image height
    pub image_height: Option<u32>,
    /// Orientation
    pub orientation: Option<u16>,
    /// GPS latitude
    pub gps_latitude: Option<(f64, f64, f64)>,
    /// GPS longitude
    pub gps_longitude: Option<(f64, f64, f64)>,
    /// GPS altitude
    pub gps_altitude: Option<f64>,
    /// All EXIF fields as key-value pairs
    #[serde(skip)]
    pub raw_fields: HashMap<String, String>,
}

impl ExifData {
    /// Returns a formatted summary of key EXIF data
    pub fn summary(&self) -> String {
        let mut parts = Vec::new();
        
        if let Some(ref make) = self.make {
            parts.push(format!("相机: {}", make));
        }
        if let Some(ref model) = self.model {
            parts.push(format!("型号: {}", model));
        }
        if let Some(ref lens) = self.lens_model {
            parts.push(format!("镜头: {}", lens));
        }
        if let Some(ref date) = self.date_time_original {
            parts.push(format!("拍摄时间: {}", date));
        }
        if let Some(ref exp) = self.exposure_time {
            parts.push(format!("快门: {}", exp));
        }
        if let Some(ref fnum) = self.f_number {
            parts.push(format!("光圈: {}", fnum));
        }
        if let Some(iso) = self.iso {
            parts.push(format!("ISO: {}", iso));
        }
        if let Some(ref focal) = self.focal_length {
            parts.push(format!("焦距: {}", focal));
        }
        if let (Some(w), Some(h)) = (self.image_width, self.image_height) {
            parts.push(format!("尺寸: {}x{}", w, h));
        }
        
        parts.join(" | ")
    }

    /// Check if GPS data is available
    pub fn has_gps(&self) -> bool {
        self.gps_latitude.is_some() && self.gps_longitude.is_some()
    }

    /// Get GPS coordinates as (latitude, longitude) tuple
    pub fn gps_coordinates(&self) -> Option<(f64, f64)> {
        match (self.gps_latitude, self.gps_longitude) {
            (Some((d1, m1, s1)), Some((d2, m2, s2))) => {
                let lat = d1 + m1 / 60.0 + s1 / 3600.0;
                let lon = d2 + m2 / 60.0 + s2 / 3600.0;
                Some((lat, lon))
            }
            _ => None,
        }
    }
}

/// Extract EXIF data from a RAW file
///
/// # Arguments
///
/// * `path` - Path to the RAW file
///
/// # Returns
///
/// Returns `Ok(ExifData)` if EXIF data was successfully extracted,
/// or an error if the file doesn't exist or has no EXIF data.
///
/// # Example
///
/// ```no_run
/// use rawlib::exif::extract_exif;
///
/// let exif = extract_exif("photo.nef").unwrap();
/// println!("Camera: {:?}", exif.model);
/// println!("ISO: {:?}", exif.iso);
/// ```
pub fn extract_exif<P: AsRef<Path>>(path: P) -> Result<ExifData, ExifError> {
    let path = path.as_ref();
    
    if !path.exists() {
        return Err(ExifError::FileNotFound(path.to_path_buf()));
    }

    let file = std::fs::File::open(path)
        .map_err(|e| ExifError::Io(e))?;
    
    let mut bufreader = std::io::BufReader::new(file);
    let exifreader = Reader::new();
    
    let exif = exifreader.read_from_container(&mut bufreader)
        .map_err(|e: exif::Error| ExifError::ParseError(e.to_string()))?;

    let mut data = ExifData::default();
    let mut raw_fields = HashMap::new();

    for field in exif.fields() {
        let tag_name = format!("{:?}", field.tag);
        let value = field.display_value().to_string();
        raw_fields.insert(tag_name.clone(), value.clone());

        match field.tag {
            Tag::Make => {
                data.make = Some(value.trim_matches('"').to_string());
            }
            Tag::Model => {
                data.model = Some(value.trim_matches('"').to_string());
            }
            Tag::LensModel => {
                data.lens_model = Some(value.trim_matches('"').to_string());
            }
            Tag::DateTimeOriginal => {
                data.date_time_original = Some(value.trim_matches('"').to_string());
            }
            Tag::ExposureTime => {
                data.exposure_time = Some(value);
            }
            Tag::FNumber => {
                data.f_number = Some(value);
            }
            Tag::ISOSpeed => {
                if let Ok(iso) = value.parse::<u32>() {
                    data.iso = Some(iso);
                }
            }
            Tag::FocalLength => {
                data.focal_length = Some(value);
            }
            Tag::ImageWidth | Tag::PixelXDimension => {
                if let Ok(w) = value.parse::<u32>() {
                    data.image_width = Some(w);
                }
            }
            Tag::ImageLength | Tag::PixelYDimension => {
                if let Ok(h) = value.parse::<u32>() {
                    data.image_height = Some(h);
                }
            }
            Tag::Orientation => {
                if let Ok(o) = value.parse::<u16>() {
                    data.orientation = Some(o);
                }
            }
            Tag::GPSLatitude => {
                data.gps_latitude = parse_gps_coordinate(&field.value);
            }
            Tag::GPSLongitude => {
                data.gps_longitude = parse_gps_coordinate(&field.value);
            }
            Tag::GPSAltitude => {
                if let Value::Rational(ref r) = field.value {
                    if let Some(rat) = r.first() {
                        let num = rat.num as f64;
                        let den = rat.denom as f64;
                        if den != 0.0 {
                            data.gps_altitude = Some(num / den);
                        }
                    }
                }
            }
            _ => {}
        }
    }

    data.raw_fields = raw_fields;
    Ok(data)
}

fn parse_gps_coordinate(value: &Value) -> Option<(f64, f64, f64)> {
    if let Value::Rational(ref r) = value {
        if r.len() >= 3 {
            let d = r[0].num as f64 / r[0].denom as f64;
            let m = r[1].num as f64 / r[1].denom as f64;
            let s = r[2].num as f64 / r[2].denom as f64;
            return Some((d, m, s));
        }
    }
    None
}

/// EXIF extraction errors
#[derive(Debug, thiserror::Error)]
pub enum ExifError {
    #[error("File not found: {0}")]
    FileNotFound(std::path::PathBuf),
    
    #[error("IO error: {0}")]
    Io(#[from] std::io::Error),
    
    #[error("Failed to parse EXIF data: {0}")]
    ParseError(String),
}

/// Extract EXIF data from multiple files in parallel
///
/// # Arguments
///
/// * `paths` - Slice of file paths
/// * `jobs` - Number of parallel jobs (None = use all CPU cores)
///
/// # Returns
///
/// Returns a vector of tuples (path, result)
///
/// # Example
///
/// ```no_run
/// use rawlib::exif::extract_exif_parallel;
/// use std::path::PathBuf;
///
/// let files = vec![
///     PathBuf::from("photo1.nef"),
///     PathBuf::from("photo2.cr2"),
/// ];
///
/// let results = extract_exif_parallel(&files, None);
/// for (path, result) in &results {
///     match result {
///         Ok(exif) => println!("{}: {:?}", path.display(), exif.model),
///         Err(e) => println!("{}: error - {}", path.display(), e),
///     }
/// }
/// ```
pub fn extract_exif_parallel<P: AsRef<Path> + Send + Sync>(
    paths: &[P],
    jobs: Option<usize>,
) -> Vec<(std::path::PathBuf, Result<ExifData, ExifError>)> {
    use rayon::prelude::*;

    let num_threads = jobs.unwrap_or_else(num_cpus::get);
    let pool = rayon::ThreadPoolBuilder::new()
        .num_threads(num_threads)
        .build()
        .ok();

    let process_fn = |path: &P| {
        let path = path.as_ref();
        let result = extract_exif(path);
        (path.to_path_buf(), result)
    };

    match pool {
        Some(pool) => pool.install(|| paths.par_iter().map(process_fn).collect()),
        None => paths.par_iter().map(process_fn).collect(),
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_exif_data_default() {
        let exif = ExifData::default();
        assert!(exif.make.is_none());
        assert!(exif.model.is_none());
        assert!(!exif.has_gps());
    }

    #[test]
    fn test_exif_data_summary() {
        let exif = ExifData {
            make: Some("NIKON".to_string()),
            model: Some("D850".to_string()),
            iso: Some(100),
            ..Default::default()
        };
        let summary = exif.summary();
        assert!(summary.contains("NIKON"));
        assert!(summary.contains("D850"));
        assert!(summary.contains("ISO"));
    }
}
