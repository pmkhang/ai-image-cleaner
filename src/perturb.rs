use crate::cli::Strength;
use image::{DynamicImage, GenericImageView};

/// Applies imperceptible micro-perturbations to disrupt frequency-domain and
/// steganographic invisible watermarks (e.g., SynthID, Stable Signature).
pub fn perturb_image(img: &DynamicImage, strength: Strength) -> DynamicImage {
    let (width, height) = img.dimensions();
    if width < 8 || height < 8 {
        return img.clone();
    }

    // Step 1: Subtle fractional border crop & resample to break fixed frequency/DCT alignment
    let crop_pixels = match strength {
        Strength::Low => 1.min(width.min(height) / 100),
        Strength::Medium => 2.min(width.min(height) / 50),
        Strength::High => 4.min(width.min(height) / 25),
    };

    let cropped = if width > crop_pixels * 2 + 2 && height > crop_pixels * 2 + 2 && crop_pixels > 0
    {
        img.crop_imm(
            crop_pixels,
            crop_pixels,
            width - crop_pixels * 2,
            height - crop_pixels * 2,
        )
        .resize_exact(width, height, image::imageops::FilterType::CatmullRom)
    } else {
        img.clone()
    };

    let mut rgba_img = cropped.to_rgba8();

    // Step 2: High-frequency micro-dithering (delta is +/- 1 or 2, imperceptible to human eye)
    let max_delta: i16 = match strength {
        Strength::Low => 1,
        Strength::Medium => 1,
        Strength::High => 2,
    };

    let sample_rate: u32 = match strength {
        Strength::Low => 3,    // 1 in 3 pixels
        Strength::Medium => 2, // 1 in 2 pixels
        Strength::High => 1,   // every pixel
    };

    for y in 0..height {
        for x in 0..width {
            if (x.wrapping_add(y * 3)) % sample_rate != 0 {
                continue;
            }
            let pixel = rgba_img.get_pixel_mut(x, y);
            // Alternate sign based on spatial parity to break steganographic latent correlation
            let sign: i16 = if (x ^ y) & 1 == 0 { 1 } else { -1 };
            let delta = sign * max_delta;

            for c in 0..3 {
                let val = pixel.0[c] as i16 + delta;
                pixel.0[c] = val.clamp(0, 255) as u8;
            }
        }
    }

    DynamicImage::ImageRgba8(rgba_img)
}

#[cfg(test)]
mod tests {
    use super::*;
    use image::{Rgba, RgbaImage};

    #[test]
    fn test_perturb_dimensions_preserved() {
        let mut img = RgbaImage::new(100, 100);
        for pixel in img.pixels_mut() {
            *pixel = Rgba([128, 128, 128, 255]);
        }
        let dynamic_img = DynamicImage::ImageRgba8(img);
        let perturbed = perturb_image(&dynamic_img, Strength::Medium);
        assert_eq!(perturbed.width(), 100);
        assert_eq!(perturbed.height(), 100);
    }

    #[test]
    fn test_perturb_small_image_handled_safely() {
        let img = RgbaImage::new(4, 4);
        let dynamic_img = DynamicImage::ImageRgba8(img);
        let perturbed = perturb_image(&dynamic_img, Strength::High);
        assert_eq!(perturbed.width(), 4);
        assert_eq!(perturbed.height(), 4);
    }
}
