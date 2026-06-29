//! Example of parallel processing with rawlib

use rawlib::parallel::{ParallelProcessor, ParallelConfig, process_files_parallel};
use std::path::PathBuf;

fn main() {
    println!("RawLib Parallel Processing Example\n");

    // Example 1: Simple parallel processing with default settings
    println!("=== Example 1: Simple Parallel Processing ===");
    
    let files = vec![
        PathBuf::from("photo1.cr2"),
        PathBuf::from("photo2.nef"),
        PathBuf::from("photo3.arw"),
    ];

    // This will automatically use all CPU cores
    let results = process_files_parallel(&files);

    for result in &results {
        match &result.thumbnail {
            Ok(thumb) => {
                println!("✓ {}: {} bytes in {:?}", 
                    result.path.display(), 
                    thumb.data.len(),
                    result.elapsed
                );
            }
            Err(e) => {
                println!("✗ {}: {}", result.path.display(), e);
            }
        }
    }

    // Example 2: Parallel processing with custom configuration
    println!("\n=== Example 2: Custom Configuration ===");

    let config = ParallelConfig {
        jobs: Some(4),      // Use exactly 4 threads
        verbose: true,      // Enable verbose output
        on_progress: None,
    };

    let results = ParallelProcessor::process_files(&files, &config);

    // Count successes and failures
    let success_count = results.iter().filter(|r| r.is_success()).count();
    let fail_count = results.iter().filter(|r| r.is_error()).count();

    println!("Success: {}, Failed: {}", success_count, fail_count);

    // Example 3: Process with statistics
    println!("\n=== Example 3: Processing with Statistics ===");

    let (results, stats) = ParallelProcessor::process_with_stats(&files, &config);

    println!("\nProcessing Statistics:");
    println!("  Total files: {}", stats.total);
    println!("  Successful: {}", stats.success);
    println!("  Failed: {}", stats.failed);
    println!("  Total time: {:?}", stats.total_elapsed);
    println!("  Speed: {:.1} files/sec", stats.files_per_second());
    println!("  Average: {:.1} ms/file", stats.ms_per_file());
    println!("  Input size: {} bytes", stats.total_input_bytes);
    println!("  Output size: {} bytes", stats.total_output_bytes);
    println!("  Compression ratio: {:.1}%", stats.compression_ratio());

    // Example 4: Process a single file
    println!("\n=== Example 4: Single File Processing ===");

    let result = ParallelProcessor::process_single("photo.cr2");
    
    println!("File: {}", result.path.display());
    println!("Input size: {} bytes", result.input_size);
    println!("Processing time: {:?}", result.elapsed);
    
    match result.thumbnail {
        Ok(thumb) => {
            println!("Thumbnail size: {}x{}", thumb.width, thumb.height);
            println!("Data size: {} bytes", thumb.data.len());
        }
        Err(e) => {
            println!("Error: {}", e);
        }
    }

    // Example 5: Filter and process only specific files
    println!("\n=== Example 5: Filter and Process ===");

    let all_files: Vec<PathBuf> = std::fs::read_dir("./photos")
        .map(|entries| {
            entries
                .filter_map(|e| e.ok())
                .map(|e| e.path())
                .filter(|p| {
                    p.extension()
                        .and_then(|e| e.to_str())
                        .map(|e| matches!(e.to_lowercase().as_str(), "cr2" | "nef" | "arw" | "rw2"))
                        .unwrap_or(false)
                })
                .collect()
        })
        .unwrap_or_default();

    if !all_files.is_empty() {
        println!("Found {} RAW files", all_files.len());
        
        let config = ParallelConfig {
            jobs: Some(8),
            verbose: true,
            on_progress: None,
        };
        
        let (_results, stats) = ParallelProcessor::process_with_stats(&all_files, &config);
        
        println!("\nProcessed at {:.1} files/sec", stats.files_per_second());
    } else {
        println!("No RAW files found in ./photos directory");
    }

    println!("\nExamples completed!");
}
