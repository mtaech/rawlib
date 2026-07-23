//! Utility functions

use crate::error::{Error, Result};
use log::debug;
use std::path::{Path, PathBuf};

/// 查找可用的文件名（通过添加数字后缀）
///
/// 例如: file.jpg -> file_1.jpg -> file_2.jpg ...
pub fn find_available_filename(base: &Path) -> Result<PathBuf> {
    if !base.exists() {
        return Ok(base.to_path_buf());
    }

    let parent = base.parent().unwrap_or_else(|| Path::new("."));
    let stem = base
        .file_stem()
        .and_then(|s| s.to_str())
        .ok_or_else(|| Error::Config {
            message: format!("Invalid file name: {}", base.display()),
        })?;
    let extension = base.extension().and_then(|e| e.to_str()).unwrap_or("");

    for i in 1..10000 {
        let new_name = if extension.is_empty() {
            format!("{}_{}", stem, i)
        } else {
            format!("{}_{}.{}", stem, i, extension)
        };

        let new_path = parent.join(&new_name);
        if !new_path.exists() {
            debug!("Found available filename: {}", new_path.display());
            return Ok(new_path);
        }
    }

    Err(Error::CannotCreateOutput {
        path: base.to_path_buf(),
        attempts: 10000,
    })
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::fs;
    use tempfile::TempDir;

    #[test]
    fn test_find_available_filename() {
        let temp_dir = TempDir::new().unwrap();
        let base = temp_dir.path().join("test.jpg");

        // 文件不存在时返回原文件名
        let result = find_available_filename(&base).unwrap();
        assert_eq!(result, base);

        // 创建文件后，应该返回 test_1.jpg
        fs::write(&base, b"test").unwrap();
        let result = find_available_filename(&base).unwrap();
        assert_eq!(result, temp_dir.path().join("test_1.jpg"));

        // 创建 test_1.jpg 后，应该返回 test_2.jpg
        fs::write(&result, b"test").unwrap();
        let result = find_available_filename(&base).unwrap();
        assert_eq!(result, temp_dir.path().join("test_2.jpg"));
    }

    #[test]
    fn test_find_available_filename_no_extension() {
        let temp_dir = TempDir::new().unwrap();
        let base = temp_dir.path().join("test");

        fs::write(&base, b"test").unwrap();
        let result = find_available_filename(&base).unwrap();
        assert_eq!(result, temp_dir.path().join("test_1"));
    }
}
