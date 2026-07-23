//! Examples of using rawlib to extract thumbnails from RAW files

use rawlib::{extract_thumbnail, extract_thumbnail_with_info, RawProcessor};
use std::path::Path;

/// Example 1: Simple thumbnail extraction
fn example_simple_extraction() -> Result<(), Box<dyn std::error::Error>> {
    println!("=== Example 1: Simple Extraction ===\n");

    let raw_file = "photo.cr2";

    // Extract thumbnail in one line
    let thumb_bytes = extract_thumbnail(raw_file)?;

    println!("✓ Extracted {} bytes", thumb_bytes.len());

    // Save to file
    std::fs::write("thumb_simple.jpg", &thumb_bytes)?;
    println!("✓ Saved to thumb_simple.jpg\n");

    Ok(())
}

/// Example 2: Extract with detailed information
fn example_with_info() -> Result<(), Box<dyn std::error::Error>> {
    println!("=== Example 2: Extract with Information ===\n");

    let raw_file = "photo.nef";

    // Extract with metadata
    let thumb = extract_thumbnail_with_info(raw_file)?;

    println!("Format: {:?}", thumb.format);
    println!("MIME Type: {}", thumb.format.mime_type());
    println!("Dimensions: {}x{} pixels", thumb.width, thumb.height);
    println!("Colors: {}", thumb.colors);
    println!("Bits per sample: {}", thumb.bits);
    println!("Data size: {} bytes", thumb.data.len());

    // Save to file
    std::fs::write("thumb_info.jpg", &thumb.data)?;
    println!("\n✓ Saved to thumb_info.jpg\n");

    Ok(())
}

/// Example 3: Using RawProcessor directly for more control
fn example_with_processor() -> Result<(), Box<dyn std::error::Error>> {
    println!("=== Example 3: Using RawProcessor ===\n");

    let raw_file = "image.arw";

    // Create processor
    let mut processor = RawProcessor::new()?;
    println!("✓ Processor created");

    // Open file
    processor.open_file(raw_file)?;
    println!("✓ File opened: {}", raw_file);

    // Extract thumbnail
    processor.unpack_thumb()?;
    println!("✓ Thumbnail unpacked");

    let thumb = processor.get_thumbnail()?;
    println!("✓ Thumbnail extracted: {}x{}", thumb.width, thumb.height);

    // Save
    std::fs::write("thumb_processor.jpg", &thumb.data)?;
    println!("✓ Saved to thumb_processor.jpg\n");

    Ok(())
}

/// Example 4: Batch processing multiple files
fn example_batch_processing() -> Result<(), Box<dyn std::error::Error>> {
    println!("=== Example 4: Batch Processing ===\n");

    let files = vec!["img1.cr2", "img2.nef", "img3.arw"];

    for (i, file) in files.iter().enumerate() {
        match extract_thumbnail(file) {
            Ok(thumb) => {
                let output = format!("thumb_{}.jpg", i + 1);
                std::fs::write(&output, &thumb)?;
                println!("✓ {} -> {} ({} bytes)", file, output, thumb.len());
            }
            Err(e) => {
                eprintln!("✗ Failed to process {}: {}", file, e);
            }
        }
    }

    println!();
    Ok(())
}

/// Example 5: Error handling
fn example_error_handling() {
    println!("=== Example 5: Error Handling ===\n");

    let non_existent = "does_not_exist.cr2";

    match extract_thumbnail(non_existent) {
        Ok(_) => println!("Unexpected success"),
        Err(e) => {
            println!("Expected error caught:");
            println!("  Error: {}", e);
        }
    }

    println!();
}

/// Example 6: Check if file has thumbnail
fn example_check_thumbnail(path: &str) -> Result<bool, Box<dyn std::error::Error>> {
    println!("=== Example 6: Check Thumbnail ===\n");

    match extract_thumbnail_with_info(path) {
        Ok(thumb) => {
            println!("✓ {} has a thumbnail:", path);
            println!("  Format: {:?}", thumb.format);
            println!("  Size: {}x{}", thumb.width, thumb.height);
            Ok(true)
        }
        Err(e) => {
            println!("✗ {} - {}", path, e);
            Ok(false)
        }
    }
}

fn main() {
    println!("RawLib Examples\n");
    println!("LibRaw Version: {}", RawProcessor::version());
    println!("Version Number: {}\n", RawProcessor::version_number());
    println!("{}", "=".repeat(50));
    println!();

    // Note: These examples require actual RAW files to run
    // Uncomment the examples you want to try

    // example_simple_extraction().ok();
    // example_with_info().ok();
    // example_with_processor().ok();
    // example_batch_processing().ok();
    // example_error_handling();
    // example_check_thumbnail("photo.cr2").ok();

    println!("Examples completed!");
    println!("\nTo run these examples:");
    println!("1. Uncomment the example functions above");
    println!("2. Make sure you have RAW files in the current directory");
    println!("3. Run: cargo run --example usage");
}
