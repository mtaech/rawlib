//! Batch processing logic

use crate::cli::{Config, ProcessMode, OverwritePolicy, Verbosity, ProcessStats};
use crate::error::{Error, Result};
use crate::utils;
use indicatif::{ProgressBar, ProgressStyle};
use log::{debug, info, warn};
use std::fs;
use std::path::{Path, PathBuf};

pub struct BatchProcessor {
    config: Config,
}

impl BatchProcessor {
    pub fn new(config: Config) -> Self {
        Self { config }
    }
    
    /// 执行批处理
    pub fn process(&self) -> Result<ProcessStats> {
        info!("Starting batch processing");
        
        // 确保输出目录存在
        self.ensure_output_directory()?;
        
        let total = self.config.input_files.len();
        let mut stats = ProcessStats {
            total,
            quiet: self.config.verbosity == Verbosity::Quiet,
            ..Default::default()
        };
        
        // 创建进度条
        let progress = self.create_progress_bar(total)?;
        
        // 处理每个文件
        for input_file in &self.config.input_files {
            if let Some(ref pb) = progress {
                pb.set_message(format!("处理: {}", input_file.display()));
            }
            
            let output_file = self.determine_output_path(input_file);
            
            // 处理覆盖策略
            let final_output = match self.handle_overwrite_policy(&output_file)? {
                Some(path) => path,
                None => {
                    stats.skipped += 1;
                    if self.config.verbosity >= Verbosity::Normal {
                        println!("⊘ 跳过 (已存在): {}", output_file.display());
                    }
                    if let Some(ref pb) = progress {
                        pb.inc(1);
                    }
                    continue;
                }
            };
            
            // 处理文件
            match self.process_single_file(input_file, &final_output) {
                Ok(size) => {
                    stats.success += 1;
                    if self.config.verbosity >= Verbosity::Normal {
                        println!("✓ {} -> {} ({} 字节)", 
                                 input_file.display(), 
                                 final_output.display(),
                                 size);
                    }
                    if self.config.verbosity == Verbosity::Verbose {
                        self.print_thumbnail_info(input_file);
                    }
                }
                Err(e) => {
                    stats.failed += 1;
                    warn!("Failed to process {}: {}", input_file.display(), e);
                    eprintln!("✗ {}: {}", input_file.display(), e);
                }
            }
            
            if let Some(ref pb) = progress {
                pb.inc(1);
            }
        }
        
        if let Some(pb) = progress {
            pb.finish_with_message("处理完成");
        }
        
        info!("Batch processing completed: {} success, {} failed, {} skipped", 
              stats.success, stats.failed, stats.skipped);
        
        Ok(stats)
    }
    
    /// 确保输出目录存在
    fn ensure_output_directory(&self) -> Result<()> {
        match self.config.mode {
            ProcessMode::SingleFile => {
                if let Some(parent) = self.config.output.parent() {
                    if !parent.exists() {
                        debug!("Creating parent directory: {}", parent.display());
                        fs::create_dir_all(parent)?;
                        if self.config.verbosity >= Verbosity::Verbose {
                            println!("创建目录: {}", parent.display());
                        }
                    }
                }
            }
            ProcessMode::Batch => {
                if !self.config.output.exists() {
                    debug!("Creating output directory: {}", self.config.output.display());
                    fs::create_dir_all(&self.config.output)?;
                    if self.config.verbosity >= Verbosity::Verbose {
                        println!("创建输出目录: {}", self.config.output.display());
                    }
                }
            }
        }
        Ok(())
    }
    
    /// 创建进度条
    fn create_progress_bar(&self, total: usize) -> Result<Option<ProgressBar>> {
        if self.config.show_progress && self.config.verbosity != Verbosity::Quiet {
            let pb = ProgressBar::new(total as u64);
            pb.set_style(
                ProgressStyle::default_bar()
                    .template("{spinner:.green} [{elapsed_precise}] [{bar:40.cyan/blue}] {pos}/{len} ({eta})\n{msg}")
                    .map_err(|e| Error::Config { 
                        message: format!("Failed to set progress style: {}", e) 
                    })?
                    .progress_chars("#>-")
            );
            Ok(Some(pb))
        } else {
            Ok(None)
        }
    }
    
    /// 确定输出路径
    fn determine_output_path(&self, input_file: &Path) -> PathBuf {
        match self.config.mode {
            ProcessMode::SingleFile => self.config.output.clone(),
            ProcessMode::Batch => {
                let file_stem = input_file.file_stem().unwrap();
                let extension = self.config.format.extension();
                self.config.output
                    .join(file_stem)
                    .with_extension(extension)
            }
        }
    }
    
    /// 处理覆盖策略
    fn handle_overwrite_policy(&self, output_file: &Path) -> Result<Option<PathBuf>> {
        if !output_file.exists() {
            return Ok(Some(output_file.to_path_buf()));
        }
        
        match self.config.overwrite_policy {
            OverwritePolicy::Skip => {
                debug!("Skipping existing file: {}", output_file.display());
                Ok(None)
            }
            OverwritePolicy::Overwrite => {
                debug!("Overwriting existing file: {}", output_file.display());
                Ok(Some(output_file.to_path_buf()))
            }
            OverwritePolicy::Rename => {
                let renamed = utils::find_available_filename(output_file)?;
                debug!("Renaming to: {}", renamed.display());
                Ok(Some(renamed))
            }
        }
    }
    
    /// 处理单个文件
    fn process_single_file(&self, input: &Path, output: &Path) -> Result<usize> {
        use rawlib::extract_thumbnail;
        
        debug!("Processing file: {} -> {}", input.display(), output.display());
        
        let thumb_bytes = extract_thumbnail(input)?;
        fs::write(output, &thumb_bytes)?;
        
        Ok(thumb_bytes.len())
    }
    
    /// 打印缩略图详细信息
    fn print_thumbnail_info(&self, input: &Path) {
        use rawlib::extract_thumbnail_with_info;
        
        if let Ok(thumb_info) = extract_thumbnail_with_info(input) {
            println!("    格式: {:?}, 尺寸: {}x{}, 颜色: {}, 位深: {}",
                     thumb_info.format,
                     thumb_info.width,
                     thumb_info.height,
                     thumb_info.colors,
                     thumb_info.bits);
        }
    }
}
