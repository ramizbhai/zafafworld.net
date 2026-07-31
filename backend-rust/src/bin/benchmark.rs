//! Native Image Processing Benchmark CLI.
//!
//! Decodes a sample image and measures encode duration, compressed size,
//! and compression ratio for all WebP and AVIF variants.

use std::time::Instant;
use image::imageops::FilterType;

fn encode_to_webp(img: &image::DynamicImage, quality: f32) -> Vec<u8> {
    let encoder = webp::Encoder::from_image(img).unwrap();
    encoder.encode(quality).to_vec()
}

fn encode_to_avif(img: &image::DynamicImage, quality: u32) -> Vec<u8> {
    let rgba8 = img.to_rgba8();
    let (width, height) = rgba8.dimensions();
    let pixels: &[ravif::RGBA8] = unsafe {
        std::slice::from_raw_parts(
            rgba8.as_ptr() as *const ravif::RGBA8,
            rgba8.len() / 4,
        )
    };
    let img_ref = ravif::Img::new(pixels, width as usize, height as usize);
    let res = ravif::Encoder::new()
        .with_quality(quality as f32)
        .with_speed(6)
        .encode_rgba(img_ref)
        .unwrap();
    res.avif_file
}

fn encode_with_ceil_webp(img: &image::DynamicImage, max_bytes: usize) -> Vec<u8> {
    let mut quality = 80.0;
    let min_quality = 50.0;
    loop {
        let bytes = encode_to_webp(img, quality);
        if bytes.len() <= max_bytes || quality <= min_quality {
            return bytes;
        }
        quality -= 5.0;
    }
}

fn encode_with_ceil_avif(img: &image::DynamicImage, max_bytes: usize) -> Vec<u8> {
    let mut quality = 80;
    let min_quality = 50;
    loop {
        let bytes = encode_to_avif(img, quality);
        if bytes.len() <= max_bytes || quality <= min_quality {
            return bytes;
        }
        quality -= 5;
    }
}

fn main() {
    println!("==========================================================");
    println!("PR-5 ENTERPRISE MEDIA PROCESSING PIPELINE BENCHMARKS");
    println!("==========================================================");

    // Read test image
    let img_path = "../client-web/build/client/logo.webp";
    let raw_bytes = std::fs::read(img_path).expect("Failed to read logo.webp");
    let raw_size = raw_bytes.len();
    println!("Source Image: {} (Raw WebP Size: {} bytes)", img_path, raw_size);

    let img = image::ImageReader::open(img_path)
        .unwrap()
        .with_guessed_format()
        .unwrap()
        .decode()
        .unwrap();

    let sizes = [
        ("thumb", 150, 15 * 1024, 10 * 1024),
        ("card", 400, 40 * 1024, 30 * 1024),
        ("medium", 800, 80 * 1024, 60 * 1024),
        ("large", 1200, 150 * 1024, 100 * 1024),
        ("original", 1920, 300 * 1024, 200 * 1024),
    ];

    println!("\n| Variant | Format | Width | Size (Bytes) | Encode Time (ms) | Comp. Ratio |");
    println!("|---|---|---|---|---|---|");

    for (name, limit, max_webp, max_avif) in sizes {
        let resized = if name == "original" && img.width() <= limit {
            img.clone()
        } else if img.width() > limit {
            img.resize(limit, 99999, FilterType::Lanczos3)
        } else {
            img.clone()
        };

        // WebP Benchmark
        let t0 = Instant::now();
        let webp_bytes = encode_with_ceil_webp(&resized, max_webp);
        let webp_dur = t0.elapsed().as_millis();
        let webp_ratio = (raw_size as f64) / (webp_bytes.len() as f64);

        println!(
            "| {:<8} | WebP   | {:<5} | {:<12} | {:<16} | {:.2}x |",
            name,
            resized.width(),
            webp_bytes.len(),
            webp_dur,
            webp_ratio
        );

        // AVIF Benchmark
        let t0 = Instant::now();
        let avif_bytes = encode_with_ceil_avif(&resized, max_avif);
        let avif_dur = t0.elapsed().as_millis();
        let avif_ratio = (raw_size as f64) / (avif_bytes.len() as f64);

        println!(
            "| {:<8} | AVIF   | {:<5} | {:<12} | {:<16} | {:.2}x |",
            name,
            resized.width(),
            avif_bytes.len(),
            avif_dur,
            avif_ratio
        );
    }
    println!("==========================================================");
}
