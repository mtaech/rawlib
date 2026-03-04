//! Batch processing logic with parallel execution support

use crate::cli::{Config, ProcessMode, OverwritePolicy, Verbosity, ProcessStats};
use crate::error::{Error, Result};
use crate::utils;
use indicatif::{ProgressBar, ProgressStyle, MultiProgress};
use log::{debug, info, warn};
use rayon::prelude::*;
use std::fs;
use std::path::{Path, PathBuf};
use std::sync::Arc;

/// 单个文件的处理结果
#[derive(Debug)]
struct FileResult {
    input: PathBuf,
    output: PathBuf,
    size: usize,
    input_size: u64,
    skipped: bool,
    error: Option<Error>,
}

pub struct BatchProcessor {
    config: Config,
}

impl BatchProcessor {
    pub fn new(config: Config) -> Self {
        Self { config }
    }
    
    /// 执行批处理（自动选择串行或并行）
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
        
        // 开始计时
        stats.start();
        
        // 如果只有一个文件，使用串行处理
        if total == 1 || self.config.jobs == Some(1) {
            debug!("Using sequential processing");
            self.process_sequential(&mut stats)?;
        } else {
            debug!("Using parallel processing");
            self.process_parallel(&mut stats)?;
        }
        
        // 结束计时
        stats.finish();
        
        info!("Batch processing completed: {} success, {} failed, {} skipped", 
              stats.success, stats.failed, stats.skipped);
        
        Ok(stats)
    }
    
    /// 串行处理（单线程）
    fn process_sequential(&self, stats: &mut ProcessStats) -> Result<()> {
        let progress = self.create_progress_bar(stats.total)?;
        
        for input_file in &self.config.input_files {
            if let Some(ref pb) = progress {
                pb.set_message(format!("处理: {}", input_file.display()));
            }
            
            let result = self.process_single_file(input_file);
            self.handle_result(&result, stats);
            
            if let Some(ref pb) = progress {
                pb.inc(1);
            }
        }
        
        if let Some(pb) = progress {
            pb.finish_with_message("处理完成");
        }
        
        Ok(())
    }
    
    /// 并行处理（多线程）
    fn process_parallel(&self, stats: &mut ProcessStats) -> Result<()> {
        // 设置 Rayon 线程池
        let jobs = self.config.jobs.unwrap_or_else(num_cpus::get);
        debug!("Using {} parallel workers", jobs);
        
        // 创建自定义线程池
        let pool = rayon::ThreadPoolBuilder::new()
            .num_threads(jobs)
            .build()
            .map_err(|e| Error::Config { 
                message: format!("Failed to create thread pool: {}", e) 
            })?;
        
        // 使用 MultiProgress 支持多线程进度条
        let multi_progress = if self.config.show_progress && self.config.verbosity != Verbosity::Quiet {
            Some(Arc::new(MultiProgress::new()))
        } else {
            None
        };
        
        let progress = multi_progress.as_ref().map(|mp| {
            let pb = mp.add(ProgressBar::new(stats.total as u64));
            pb.set_style(
                ProgressStyle::default_bar()
                    .template("{spinner:.green} [{elapsed_precise}] [{bar:40.cyan/blue}] {pos}/{len} ({eta})")
                    .unwrap()
                    .progress_chars("#>-")
            );
            pb
        });
        
        // 并行处理文件
        let results: Vec<FileResult> = pool.install(|| {
            self.config.input_files
                .par_iter()
                .map(|input_file| {
                    let result = self.process_single_file(input_file);
                    
                    if let Some(ref pb) = progress {
                        pb.inc(1);
                    }
                    
                    result
                })
                .collect()
        });
        
        if let Some(pb) = progress {
            pb.finish_with_message("处理完成");
        }
        
        // 汇总结果
        for result in results {
            self.handle_result(&result, stats);
        }
        
        Ok(())
    }
    
    /// 处理单个文件并返回结果
    fn process_single_file(&self, input_file: &Path) -> FileResult {
        let output_file = self.determine_output_path(input_file);
        
        // 获取输入文件大小
        let input_size = fs::metadata(input_file)
            .map(|m| m.len())
            .unwrap_or(0);
        
        // 处理覆盖策略
        let final_output = match self.handle_overwrite_policy(&output_file) {
            Ok(Some(path)) => path,
            Ok(None) => {
                return FileResult {
                    input: input_file.to_path_buf(),
                    output: output_file,
                    size: 0,
                    input_size,
                    skipped: true,
                    error: None,
                };
            }
            Err(e) => {
                return FileResult {
                    input: input_file.to_path_buf(),
                    output: output_file,
                    size: 0,
                    input_size,
                    skipped: false,
                    error: Some(e),
                };
            }
        };
        
        // 提取并保存缩略图
        match self.extract_and_save(input_file, &final_output) {
            Ok(size) => FileResult {
                input: input_file.to_path_buf(),
                output: final_output,
                size,
                input_size,
                skipped: false,
                error: None,
            },
            Err(e) => FileResult {
                input: input_file.to_path_buf(),
                output: final_output,
                size: 0,
                input_size,
                skipped: false,
                error: Some(e),
            },
        }
    }
    
    /// 提取并保存缩略图
    fn extract_and_save(&self, input: &Path, output: &Path) -> Result<usize> {
        use rawlib::extract_thumbnail;
        
        debug!("Processing file: {} -> {}", input.display(), output.display());
        
        let thumb_bytes = extract_thumbnail(input)?;
        fs::write(output, &thumb_bytes)?;
        
        Ok(thumb_bytes.len())
    }
    
    /// 处理结果并更新统计
    fn handle_result(&self, result: &FileResult, stats: &mut ProcessStats) {
        if result.skipped {
            stats.skipped += 1;
            if self.config.verbosity >= Verbosity::Normal {
                println!("⊘ 跳过 (已存在): {}", result.output.display());
            }
        } else if let Some(ref e) = result.error {
            stats.failed += 1;
            stats.total_input_bytes += result.input_size;
            warn!("Failed to process {}: {}", result.input.display(), e);
            eprintln!("✗ {}: {}", result.input.display(), e);
        } else {
            stats.success += 1;
            stats.total_input_bytes += result.input_size;
            stats.total_output_bytes += result.size as u64;
            if self.config.verbosity >= Verbosity::Normal {
                println!("✓ {} -> {} ({} 字节)", 
                         result.input.display(), 
                         result.output.display(),
                         result.size);
            }
            if self.config.verbosity == Verbosity::Verbose {
                self.print_thumbnail_info(&result.input);
            }
        }
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
