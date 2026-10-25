use ai_image_cleaner::batch::run_batch;
use ai_image_cleaner::cleaner::{clean_image_file, inspect_metadata};
use ai_image_cleaner::cli::{Cli, Strength};
use crc32fast::Hasher;
use image::{Rgb, RgbImage, Rgba, RgbaImage};
use std::fs;
use std::path::Path;

/// Creates a minimal PNG with a custom tEXt chunk containing AI prompt parameters
fn create_test_ai_png(path: &Path, prompt: &str) {
    let mut img = RgbaImage::new(32, 32);
    for pixel in img.pixels_mut() {
        *pixel = Rgba([100, 150, 200, 255]);
    }

    let mut png_bytes = Vec::new();
    img.write_to(
        &mut std::io::Cursor::new(&mut png_bytes),
        image::ImageFormat::Png,
    )
    .expect("Failed to encode base PNG");

    // Locate IEND chunk and insert tEXt chunk right before it
    let iend_pos = png_bytes
        .windows(4)
        .position(|w| w == b"IEND")
        .expect("IEND chunk not found")
        - 4;

    let mut chunk_data = Vec::new();
    chunk_data.extend_from_slice(b"parameters\0");
    chunk_data.extend_from_slice(prompt.as_bytes());

    let mut text_chunk = Vec::new();
    text_chunk.extend_from_slice(&(chunk_data.len() as u32).to_be_bytes());
    text_chunk.extend_from_slice(b"tEXt");
    text_chunk.extend_from_slice(&chunk_data);

    let mut hasher = Hasher::new();
    hasher.update(b"tEXt");
    hasher.update(&chunk_data);
    let crc = hasher.finalize();
    text_chunk.extend_from_slice(&crc.to_be_bytes());

    let mut full_png = Vec::new();
    full_png.extend_from_slice(&png_bytes[..iend_pos]);
    full_png.extend_from_slice(&text_chunk);
    full_png.extend_from_slice(&png_bytes[iend_pos..]);

    fs::write(path, full_png).expect("Failed to write test PNG file");
}

/// Creates a standard JPEG image
fn create_test_jpeg(path: &Path) {
    let mut img = RgbImage::new(32, 32);
    for pixel in img.pixels_mut() {
        *pixel = Rgb([200, 100, 50]);
    }
    img.save_with_format(path, image::ImageFormat::Jpeg)
        .expect("Failed to save test JPEG");
}

#[test]
fn test_png_ai_metadata_detection_and_stripping() {
    let temp_dir = tempfile::tempdir().expect("Failed to create tempdir");
    let input_path = temp_dir.path().join("ai_generated.png");
    let output_path = temp_dir.path().join("cleaned.png");

    let prompt = "A photorealistic portrait of an astronaut on Mars, 8k, unreal engine 5";
    create_test_ai_png(&input_path, prompt);

    // 1. Inspect original file
    let (_, summary) = inspect_metadata(&input_path).expect("Failed to inspect");
    assert!(summary.has_ai_prompt, "Expected AI prompt to be detected");
    assert!(
        summary.detected_tags.iter().any(|t| t.contains("tEXt")),
        "Expected tEXt chunk in tags"
    );

    // 2. Clean the file (Standard mode)
    let result = clean_image_file(&input_path, &output_path, false, Strength::Medium, false)
        .expect("Failed to clean file");

    assert_eq!(result.width, 32);
    assert_eq!(result.height, 32);
    assert!(output_path.exists());

    // 3. Inspect cleaned file
    let (_, cleaned_summary) = inspect_metadata(&output_path).expect("Failed to inspect cleaned");
    assert!(
        !cleaned_summary.has_any(),
        "Expected zero AI metadata in cleaned image, found: {:?}",
        cleaned_summary.detected_tags
    );
}

#[test]
fn test_deep_cleaning_mode() {
    let temp_dir = tempfile::tempdir().expect("Failed to create tempdir");
    let input_path = temp_dir.path().join("test_deep.png");
    let output_path = temp_dir.path().join("cleaned_deep.png");

    create_test_ai_png(
        &input_path,
        "Cyberpunk city at night, synthwave neon lighting",
    );

    // Clean with deep mode enabled
    let result = clean_image_file(&input_path, &output_path, true, Strength::High, false)
        .expect("Deep clean failed");

    assert!(result.deep_applied);
    assert!(output_path.exists());

    // Verify cleaned image decodes and preserves original dimensions
    let decoded = image::open(&output_path).expect("Failed to open cleaned deep image");
    assert_eq!(decoded.width(), 32);
    assert_eq!(decoded.height(), 32);

    // Verify metadata was completely stripped
    let (_, cleaned_summary) = inspect_metadata(&output_path).expect("Failed to inspect");
    assert!(!cleaned_summary.has_any());
}

#[test]
fn test_batch_processing_and_directory_hierarchy() {
    let temp_dir = tempfile::tempdir().expect("Failed to create tempdir");
    let src_dir = temp_dir.path().join("raw_images");
    let nested_dir = src_dir.join("subfolder");
    fs::create_dir_all(&nested_dir).expect("Failed to create dirs");

    let img1 = src_dir.join("img1.png");
    let img2 = nested_dir.join("img2.jpg");

    create_test_ai_png(&img1, "Midjourney v6 prompt --ar 16:9");
    create_test_jpeg(&img2);

    let out_dir = temp_dir.path().join("output_images");

    let cli = Cli {
        input: src_dir.clone(),
        output: Some(out_dir.clone()),
        in_place: false,
        deep: true,
        strength: Strength::Medium,
        recursive: true,
        threads: Some(2),
        dry_run: false,
        verbose: true,
    };

    let stats = run_batch(&cli).expect("Batch processing failed");
    assert_eq!(stats.total_scanned, 2);
    assert_eq!(stats.successfully_cleaned, 2);
    assert_eq!(stats.metadata_detected, 1);

    // Verify output hierarchy
    let cleaned_img1 = out_dir.join("img1.png");
    let cleaned_img2 = out_dir.join("subfolder").join("img2.jpg");
    assert!(cleaned_img1.exists());
    assert!(cleaned_img2.exists());
}

#[test]
fn test_dry_run_does_not_modify_or_create_files() {
    let temp_dir = tempfile::tempdir().expect("Failed to create tempdir");
    let input_path = temp_dir.path().join("dry_run_test.png");
    let output_path = temp_dir.path().join("should_not_exist.png");

    create_test_ai_png(&input_path, "DALL-E 3 generated prompt");

    let res = clean_image_file(&input_path, &output_path, false, Strength::Medium, true)
        .expect("Dry run failed");

    assert!(res.metadata_found.has_ai_prompt);
    assert!(
        !output_path.exists(),
        "Dry-run should not create output files"
    );
}
