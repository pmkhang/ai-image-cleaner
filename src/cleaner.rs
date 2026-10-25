use crate::cli::Strength;
use crate::error::CleanerError;
use crate::perturb::perturb_image;
use image::{
    ImageEncoder, ImageFormat, codecs::jpeg::JpegEncoder, codecs::png::PngEncoder,
    codecs::webp::WebPEncoder,
};
use std::fs::{self, File};
use std::io::{BufReader, BufWriter, Read};
use std::path::Path;

#[derive(Debug, Clone, Default)]
pub struct MetadataSummary {
    pub has_c2pa: bool,
    pub has_exif: bool,
    pub has_xmp: bool,
    pub has_ai_prompt: bool,
    pub detected_tags: Vec<String>,
}

impl MetadataSummary {
    pub fn has_any(&self) -> bool {
        self.has_c2pa
            || self.has_exif
            || self.has_xmp
            || self.has_ai_prompt
            || !self.detected_tags.is_empty()
    }
}

#[derive(Debug, Clone)]
pub struct CleanResult {
    #[allow(dead_code)]
    pub format: ImageFormat,
    #[allow(dead_code)]
    pub width: u32,
    #[allow(dead_code)]
    pub height: u32,
    pub original_size: u64,
    pub cleaned_size: u64,
    pub metadata_found: MetadataSummary,
    pub deep_applied: bool,
}

/// Inspects an image file to check for AI metadata (C2PA, EXIF, XMP, PNG text chunks)
pub fn inspect_metadata(path: &Path) -> Result<(ImageFormat, MetadataSummary), CleanerError> {
    let file = File::open(path).map_err(|e| CleanerError::Io {
        path: path.to_path_buf(),
        source: e,
    })?;
    let mut reader = BufReader::new(file);

    let mut header = [0u8; 16];
    let n = reader.read(&mut header).unwrap_or(0);
    let format = ImageFormat::from_path(path).unwrap_or_else(|_| {
        if n >= 8 && &header[0..8] == b"\x89PNG\r\n\x1a\n" {
            ImageFormat::Png
        } else if n >= 2 && header[0] == 0xFF && header[1] == 0xD8 {
            ImageFormat::Jpeg
        } else if n >= 12 && &header[0..4] == b"RIFF" && &header[8..12] == b"WEBP" {
            ImageFormat::WebP
        } else {
            ImageFormat::Png
        }
    });

    let mut summary = MetadataSummary::default();

    // Read full file bytes for inspection
    let bytes = fs::read(path).map_err(|e| CleanerError::Io {
        path: path.to_path_buf(),
        source: e,
    })?;

    match format {
        ImageFormat::Png => inspect_png(&bytes, &mut summary),
        ImageFormat::Jpeg => inspect_jpeg(&bytes, &mut summary),
        ImageFormat::WebP => inspect_webp(&bytes, &mut summary),
        _ => {}
    }

    Ok((format, summary))
}

fn inspect_png(bytes: &[u8], summary: &mut MetadataSummary) {
    if bytes.len() < 8 || &bytes[0..8] != b"\x89PNG\r\n\x1a\n" {
        return;
    }
    let mut pos = 8;
    while pos + 8 <= bytes.len() {
        let len = u32::from_be_bytes([bytes[pos], bytes[pos + 1], bytes[pos + 2], bytes[pos + 3]])
            as usize;
        let chunk_type = &bytes[pos + 4..pos + 8];
        let chunk_name = String::from_utf8_lossy(chunk_type).to_string();

        match chunk_type {
            b"c2pa" | b"c2PA" | b"caBX" => {
                summary.has_c2pa = true;
                summary
                    .detected_tags
                    .push(format!("C2PA Content Credentials ({chunk_name})"));
            }
            b"eXIf" => {
                summary.has_exif = true;
                summary.detected_tags.push("EXIF metadata".into());
            }
            b"tEXt" | b"zTXt" | b"iTXt" => {
                let chunk_data = if pos + 8 + len <= bytes.len() {
                    &bytes[pos + 8..pos + 8 + len]
                } else {
                    &[]
                };
                if chunk_data.starts_with(b"XML:com.adobe.xmp") {
                    summary.has_xmp = true;
                    summary.detected_tags.push("XMP metadata (iTXt)".into());
                } else {
                    summary.has_ai_prompt = true;
                    let text = String::from_utf8_lossy(chunk_data).to_string();
                    let preview = if text.len() > 30 {
                        format!("{}...", text[..30].replace('\n', " "))
                    } else {
                        text.replace('\n', " ")
                    };
                    summary
                        .detected_tags
                        .push(format!("{chunk_name}: {preview}"));
                }
            }
            _ => {}
        }

        pos += 12 + len; // 4 len + 4 type + len data + 4 crc
    }
}

fn inspect_jpeg(bytes: &[u8], summary: &mut MetadataSummary) {
    let mut pos = 2;
    while pos + 4 <= bytes.len() {
        if bytes[pos] != 0xFF {
            break;
        }
        let marker = bytes[pos + 1];
        if marker == 0xDA || marker == 0xD9 {
            // SOS or EOI
            break;
        }
        if marker == 0x00 || (0xD0..=0xD7).contains(&marker) {
            pos += 2;
            continue;
        }
        let len = u16::from_be_bytes([bytes[pos + 2], bytes[pos + 3]]) as usize;
        let payload_start = pos + 4;
        let payload_end = (pos + 2 + len).min(bytes.len());
        let payload = if payload_start < payload_end {
            &bytes[payload_start..payload_end]
        } else {
            &[]
        };

        match marker {
            0xE1 => {
                // APP1: EXIF or XMP
                if payload.starts_with(b"Exif\0\0") {
                    summary.has_exif = true;
                    summary.detected_tags.push("EXIF (APP1)".into());
                } else if payload.starts_with(b"http://ns.adobe.com/xap/1.0/\0") {
                    summary.has_xmp = true;
                    summary.detected_tags.push("XMP (APP1)".into());
                }
            }
            0xEB => {
                // APP11: C2PA JUMBF Manifest
                summary.has_c2pa = true;
                summary
                    .detected_tags
                    .push("C2PA JUMBF Manifest (APP11)".into());
            }
            0xED => {
                // APP13: IPTC / Photoshop
                summary.detected_tags.push("IPTC/Photoshop (APP13)".into());
            }
            0xFE => {
                // COM: Comment
                summary.has_ai_prompt = true;
                summary.detected_tags.push("JPEG Comment (COM)".into());
            }
            _ => {}
        }

        pos += 2 + len;
    }
}

fn inspect_webp(bytes: &[u8], summary: &mut MetadataSummary) {
    if bytes.len() < 12 || &bytes[0..4] != b"RIFF" || &bytes[8..12] != b"WEBP" {
        return;
    }
    let mut pos = 12;
    while pos + 8 <= bytes.len() {
        let chunk_type = &bytes[pos..pos + 4];
        let len = u32::from_le_bytes([
            bytes[pos + 4],
            bytes[pos + 5],
            bytes[pos + 6],
            bytes[pos + 7],
        ]) as usize;
        match chunk_type {
            b"EXIF" => {
                summary.has_exif = true;
                summary.detected_tags.push("WebP EXIF".into());
            }
            b"XMP " => {
                summary.has_xmp = true;
                summary.detected_tags.push("WebP XMP".into());
            }
            b"C2PA" => {
                summary.has_c2pa = true;
                summary.detected_tags.push("WebP C2PA".into());
            }
            _ => {}
        }
        // WebP chunks are padded to 2-byte boundary
        let padded_len = if !len.is_multiple_of(2) { len + 1 } else { len };
        pos += 8 + padded_len;
    }
}

pub fn erase_corner_logo(img: &mut image::DynamicImage) {
    let (w, h) = image::GenericImageView::dimensions(img);
    if w < 200 || h < 200 {
        return;
    }

    let mut rgba = img.to_rgba8();

    // Standard Gemini watermark position: centered around (w - 112, h - 112)
    let center_x = (w - 112) as f32;
    let center_y = (h - 112) as f32;
    let star_radius = 70.0f32;
    let p = 0.50f32;

    let min_x = (w - 200).max(0);
    let max_x = w;
    let min_y = (h - 200).max(0);
    let max_y = h;

    for y in min_y..max_y {
        for x in min_x..max_x {
            let dx = (x as f32 - center_x).abs();
            let dy = (y as f32 - center_y).abs();

            if dx.powf(p) + dy.powf(p) <= (star_radius + 4.0).powf(p) {
                let mut samples = Vec::new();
                for r in (32..=52).step_by(2) {
                    for deg in (0..360).step_by(10) {
                        let rad = (deg as f32).to_radians();
                        let sx = (x as f32 + rad.cos() * r as f32).round() as i32;
                        let sy = (y as f32 + rad.sin() * r as f32).round() as i32;
                        if sx >= 0 && sx < w as i32 && sy >= 0 && sy < h as i32 {
                            let s_dx = (sx as f32 - center_x).abs();
                            let s_dy = (sy as f32 - center_y).abs();
                            if s_dx.powf(p) + s_dy.powf(p) > (star_radius + 6.0).powf(p) {
                                samples.push(rgba.get_pixel(sx as u32, sy as u32));
                            }
                        }
                    }
                }

                if !samples.is_empty() {
                    let mut r_vals: Vec<u8> = samples.iter().map(|p| p[0]).collect();
                    let mut g_vals: Vec<u8> = samples.iter().map(|p| p[1]).collect();
                    let mut b_vals: Vec<u8> = samples.iter().map(|p| p[2]).collect();
                    r_vals.sort_unstable();
                    g_vals.sort_unstable();
                    b_vals.sort_unstable();

                    let dither: i16 = if (x ^ y) & 1 == 0 { 2 } else { -2 };
                    let med_r = (r_vals[r_vals.len() / 2] as i16 + dither).clamp(0, 255) as u8;
                    let med_g = (g_vals[g_vals.len() / 2] as i16 + dither).clamp(0, 255) as u8;
                    let med_b = (b_vals[b_vals.len() / 2] as i16 + dither).clamp(0, 255) as u8;

                    rgba.put_pixel(x, y, image::Rgba([med_r, med_g, med_b, 255]));
                }
            }
        }
    }

    *img = image::DynamicImage::ImageRgba8(rgba);
}

/// Cleans an image file by stripping all metadata, optionally perturbing pixels,
/// and optionally erasing visible corner AI watermarks (e.g. Gemini sparkle).
pub fn clean_image_file(
    input_path: &Path,
    output_path: &Path,
    deep: bool,
    strength: Strength,
    erase_logo: bool,
    dry_run: bool,
) -> Result<CleanResult, CleanerError> {
    let original_size = fs::metadata(input_path)
        .map_err(|e| CleanerError::Io {
            path: input_path.to_path_buf(),
            source: e,
        })?
        .len();

    let (format, metadata_found) = inspect_metadata(input_path)?;

    if dry_run {
        return Ok(CleanResult {
            format,
            width: 0,
            height: 0,
            original_size,
            cleaned_size: original_size,
            metadata_found,
            deep_applied: false,
        });
    }

    // Decode original image pixels
    let mut img = image::open(input_path).map_err(|e| CleanerError::Image {
        path: input_path.to_path_buf(),
        source: e,
    })?;

    let width = img.width();
    let height = img.height();

    // If erase_logo is requested, erase visible corner watermarks (Gemini star)
    if erase_logo {
        erase_corner_logo(&mut img);
    }

    // If deep mode is requested, apply subtle watermark disruption
    let mut deep_applied = false;
    if deep {
        img = perturb_image(&img, strength);
        deep_applied = true;
    }

    // Write to a temporary file in the target directory, then atomic rename
    let output_dir = output_path.parent().unwrap_or_else(|| Path::new("."));
    fs::create_dir_all(output_dir).map_err(|e| CleanerError::Io {
        path: output_dir.to_path_buf(),
        source: e,
    })?;

    let temp_path = output_dir.join(format!(
        ".tmp_{}_{}",
        std::process::id(),
        output_path
            .file_name()
            .unwrap_or_default()
            .to_string_lossy()
    ));

    {
        let file = File::create(&temp_path).map_err(|e| CleanerError::Io {
            path: temp_path.clone(),
            source: e,
        })?;
        let mut writer = BufWriter::new(file);

        match format {
            ImageFormat::Jpeg => {
                let rgb_img = img.to_rgb8();
                let mut encoder = JpegEncoder::new_with_quality(&mut writer, 95);
                encoder
                    .encode_image(&rgb_img)
                    .map_err(|e| CleanerError::Image {
                        path: temp_path.clone(),
                        source: e,
                    })?;
            }
            ImageFormat::Png => {
                let encoder = PngEncoder::new(&mut writer);
                encoder
                    .write_image(
                        img.as_bytes(),
                        img.width(),
                        img.height(),
                        img.color().into(),
                    )
                    .map_err(|e| CleanerError::Image {
                        path: temp_path.clone(),
                        source: e,
                    })?;
            }
            ImageFormat::WebP => {
                let encoder = WebPEncoder::new_lossless(&mut writer);
                encoder
                    .write_image(
                        img.as_bytes(),
                        img.width(),
                        img.height(),
                        img.color().into(),
                    )
                    .map_err(|e| CleanerError::Image {
                        path: temp_path.clone(),
                        source: e,
                    })?;
            }
            _ => {
                // For other supported formats (e.g. BMP/TIFF), write clean format
                img.write_to(&mut writer, format)
                    .map_err(|e| CleanerError::Image {
                        path: temp_path.clone(),
                        source: e,
                    })?;
            }
        }
    }

    // Atomic replace
    fs::rename(&temp_path, output_path).map_err(|e| CleanerError::Io {
        path: output_path.to_path_buf(),
        source: e,
    })?;

    let cleaned_size = fs::metadata(output_path)
        .map_err(|e| CleanerError::Io {
            path: output_path.to_path_buf(),
            source: e,
        })?
        .len();

    Ok(CleanResult {
        format,
        width,
        height,
        original_size,
        cleaned_size,
        metadata_found,
        deep_applied,
    })
}
