//! Accent color extracted from a game's artwork (DOIT.md §3, §5).
//!
//! Computed in Rust (not in a canvas, which may be blocked from reading
//! asset:// images) and cached in `games.accent_color`:
//!   NULL  = not computed yet
//!   ''    = computed, no usable color (missing/undecodable art, grayscale)
//!   '#rrggbb' = color to tint the Hero area with
//! Steam media sync resets it to NULL when artwork is re-downloaded.

use image::imageops::FilterType;
use rusqlite::{params, Connection};
use std::path::Path;

/// Return the cached accent color, computing and storing it on first use.
pub fn ensure_accent_color(
    conn: &Connection,
    game_id: i64,
    cached: Option<String>,
    hero_path: Option<&str>,
    cover_path: Option<&str>,
) -> Option<String> {
    if let Some(value) = cached {
        return (!value.is_empty()).then_some(value);
    }

    let computed = [hero_path, cover_path]
        .into_iter()
        .flatten()
        .find_map(|path| extract_accent(Path::new(path)));

    let stored = computed.clone().unwrap_or_default();
    let _ = conn.execute(
        "UPDATE games SET accent_color = ?1 WHERE id = ?2",
        params![stored, game_id],
    );
    computed
}

/// Dominant, reasonably saturated color of an image, normalized to a tone that
/// works as a subtle tint in both light and dark themes.
pub fn extract_accent(path: &Path) -> Option<String> {
    let image = image::open(path).ok()?;
    let thumb = image.resize(48, 48, FilterType::Triangle).to_rgb8();

    // 4-bit-per-channel histogram, weighted toward saturated mid-tones.
    let mut bins = vec![(0.0_f64, [0.0_f64; 3]); 4096];
    for pixel in thumb.pixels() {
        let [r, g, b] = pixel.0;
        let (_, s, l) = rgb_to_hsl(r, g, b);
        if !(0.12..=0.92).contains(&l) {
            continue;
        }
        let weight = 0.25 + s * (1.0 - (l - 0.5).abs() * 1.4).max(0.0);
        let index = ((r as usize >> 4) << 8) | ((g as usize >> 4) << 4) | (b as usize >> 4);
        let bin = &mut bins[index];
        bin.0 += weight;
        bin.1[0] += r as f64 * weight;
        bin.1[1] += g as f64 * weight;
        bin.1[2] += b as f64 * weight;
    }

    let (weight, sums) = bins
        .into_iter()
        .max_by(|a, b| a.0.partial_cmp(&b.0).unwrap_or(std::cmp::Ordering::Equal))?;
    if weight <= 0.0 {
        return None;
    }

    let [r, g, b] = sums.map(|sum| (sum / weight).round().clamp(0.0, 255.0) as u8);
    let (h, s, l) = rgb_to_hsl(r, g, b);
    if s < 0.12 {
        return None; // Near-grayscale art: no tint.
    }

    let (r, g, b) = hsl_to_rgb(h, s.min(0.7), l.clamp(0.38, 0.58));
    Some(format!("#{:02x}{:02x}{:02x}", r, g, b))
}

fn rgb_to_hsl(r: u8, g: u8, b: u8) -> (f64, f64, f64) {
    let (r, g, b) = (r as f64 / 255.0, g as f64 / 255.0, b as f64 / 255.0);
    let max = r.max(g).max(b);
    let min = r.min(g).min(b);
    let l = (max + min) / 2.0;
    if (max - min).abs() < f64::EPSILON {
        return (0.0, 0.0, l);
    }
    let d = max - min;
    let s = if l > 0.5 {
        d / (2.0 - max - min)
    } else {
        d / (max + min)
    };
    let h = if max == r {
        ((g - b) / d + if g < b { 6.0 } else { 0.0 }) / 6.0
    } else if max == g {
        ((b - r) / d + 2.0) / 6.0
    } else {
        ((r - g) / d + 4.0) / 6.0
    };
    (h, s, l)
}

fn hsl_to_rgb(h: f64, s: f64, l: f64) -> (u8, u8, u8) {
    fn hue(p: f64, q: f64, mut t: f64) -> f64 {
        if t < 0.0 {
            t += 1.0;
        }
        if t > 1.0 {
            t -= 1.0;
        }
        if t < 1.0 / 6.0 {
            p + (q - p) * 6.0 * t
        } else if t < 0.5 {
            q
        } else if t < 2.0 / 3.0 {
            p + (q - p) * (2.0 / 3.0 - t) * 6.0
        } else {
            p
        }
    }
    let q = if l < 0.5 {
        l * (1.0 + s)
    } else {
        l + s - l * s
    };
    let p = 2.0 * l - q;
    let to_byte = |v: f64| (v * 255.0).round().clamp(0.0, 255.0) as u8;
    (
        to_byte(hue(p, q, h + 1.0 / 3.0)),
        to_byte(hue(p, q, h)),
        to_byte(hue(p, q, h - 1.0 / 3.0)),
    )
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn extracts_saturated_color_and_skips_grayscale() {
        let dir = std::env::temp_dir();
        let red = dir.join("playarc_accent_red.png");
        let gray = dir.join("playarc_accent_gray.png");
        image::RgbImage::from_pixel(32, 32, image::Rgb([200, 40, 30]))
            .save(&red)
            .unwrap();
        image::RgbImage::from_pixel(32, 32, image::Rgb([128, 128, 128]))
            .save(&gray)
            .unwrap();

        let color = extract_accent(&red).expect("color");
        assert!(color.starts_with('#') && color.len() == 7);
        let r = u8::from_str_radix(&color[1..3], 16).unwrap();
        let g = u8::from_str_radix(&color[3..5], 16).unwrap();
        assert!(r > g, "should stay red-ish: {color}");
        assert_eq!(extract_accent(&gray), None);
    }
}
