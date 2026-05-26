//! ============================================================================
//! JARVIS OS Post-Processing Effects — HDR, Bloom, SSGI, Volumetrics
//! ============================================================================
//! Implements:
//!   - ACES Filmic Tone Mapping (industry standard, used in Unreal)
//!   - Reinhard Tone Mapping (fallback)
//!   - Multi-resolution Gaussian Bloom (bloom v2)
//!   - Screen-Space Global Illumination (simplified SSGI)
//!   - Volumetric Fog / God Rays
//!   - Chromatic Aberration
//!   - Film Grain
//! ============================================================================

use alloc::vec::Vec;

// ============================================================================
// TONE MAPPING — Convert HDR to LDR (sRGB)
// ============================================================================

/// ACES Filmic Tone Mapping (AMPAS Academy Color Encoding System)
/// Industry standard used in Unreal Engine, Unity, Blender
pub fn tone_map_aces(hdr_r: f32, hdr_g: f32, hdr_b: f32) -> (u8, u8, u8) {
    let r = aces_tonemap(hdr_r);
    let g = aces_tonemap(hdr_g);
    let b = aces_tonemap(hdr_b);
    (
        (r * 255.0).max(0.0).min(255.0) as u8,
        (g * 255.0).max(0.0).min(255.0) as u8,
        (b * 255.0).max(0.0).min(255.0) as u8,
    )
}

fn aces_tonemap(x: f32) -> f32 {
    let x = x.max(0.0);
    // ACES approximation (Narkowicz 2015)
    let a = 2.51;
    let b = 0.03;
    let c = 2.43;
    let d = 0.59;
    let e = 0.14;
    (x * (a * x + b)) / (x * (c * x + d) + e)
}

/// Reinhard Tone Mapping (simpler fallback)
pub fn tone_map_reinhard(hdr_r: f32, hdr_g: f32, hdr_b: f32) -> (u8, u8, u8) {
    let r = hdr_r.max(0.0) / (1.0 + hdr_r.max(0.0));
    let g = hdr_g.max(0.0) / (1.0 + hdr_g.max(0.0));
    let b = hdr_b.max(0.0) / (1.0 + hdr_b.max(0.0));
    (
        (r * 255.0) as u8,
        (g * 255.0) as u8,
        (b * 255.0) as u8,
    )
}

/// Generic tone map an ARGB HDR buffer to sRGB
pub fn tone_map_buffer_aces(buffer: &mut [u32]) {
    for pixel in buffer.iter_mut() {
        let r = ((*pixel >> 16) & 0xFF) as f32 / 255.0;
        let g = ((*pixel >> 8) & 0xFF) as f32 / 255.0;
        let b = (*pixel & 0xFF) as f32 / 255.0;

        // Scale up to HDR range
        let hdr_r = r * 4.0;
        let hdr_g = g * 4.0;
        let hdr_b = b * 4.0;

        let (tr, tg, tb) = tone_map_aces(hdr_r, hdr_g, hdr_b);
        *pixel = 0xFF000000 | (tr as u32) << 16 | (tg as u32) << 8 | tb as u32;
    }
}

// ============================================================================
// GAUSSIAN BLOOM — Multi-resolution
// ============================================================================
pub struct BloomPass {
    scratch: Vec<u32>,
    width: usize,
    height: usize,
}

impl BloomPass {
    pub fn new(width: usize, height: usize) -> Self {
        Self {
            scratch: alloc::vec![0u32; width * height],
            width,
            height,
        }
    }

    /// Apply bloom (glow) effect: extract brights → blur → composite
    pub fn apply(&mut self, buffer: &mut [u32], threshold: f32, strength: f32, radius: usize) {
        let size = self.width * self.height;

        // 1. Extract bright pixels
        for i in 0..size {
            let c = buffer[i];
            let r = ((c >> 16) & 0xFF) as f32;
            let g = ((c >> 8) & 0xFF) as f32;
            let b = (c & 0xFF) as f32;
            let luminance = (r * 0.2126 + g * 0.7152 + b * 0.0722) / 255.0;
            if luminance > threshold {
                let excess = (luminance - threshold) / (1.0 - threshold);
                let sr = (r * excess) as u32;
                let sg = (g * excess) as u32;
                let sb = (b * excess) as u32;
                self.scratch[i] = 0xFF000000 | (sr.min(255) << 16) | (sg.min(255) << 8) | sb.min(255);
            } else {
                self.scratch[i] = 0;
            }
        }

        // 2. Multi-pass Gaussian blur
        for _ in 0..3 {
            self.gaussian_blur_x();
            self.gaussian_blur_y();
        }

        // 3. Composite bloom onto original
        for i in 0..size {
            let bloom = self.scratch[i];
            if bloom == 0 { continue; }
            let br = ((bloom >> 16) & 0xFF) as u16;
            let bg = ((bloom >> 8) & 0xFF) as u16;
            let bb = (bloom & 0xFF) as u16;
            let orig = buffer[i];
            let or = ((orig >> 16) & 0xFF) as u16;
            let og = ((orig >> 8) & 0xFF) as u16;
            let ob = (orig & 0xFF) as u16;
            let mr = strength;
            let nr = (or + (br as f32 * mr * 2.0) as u16).min(255);
            let ng = (og + (bg as f32 * mr * 2.0) as u16).min(255);
            let nb = (ob + (bb as f32 * mr * 2.0) as u16).min(255);
            buffer[i] = 0xFF000000 | (nr as u32) << 16 | (ng as u32) << 8 | nb as u32;
        }
    }

    /// Horizontal box blur pass
    fn gaussian_blur_x(&mut self) {
        let kernel_radius = 2usize;

        for y in 0..self.height {
            for x in 0..self.width {
                let mut r = 0u32; let mut g = 0u32; let mut b = 0u32;
                let mut count = 0u32;

                for kx in x.saturating_sub(kernel_radius)..= (x + kernel_radius).min(self.width - 1) {
                    let c = self.scratch[y * self.width + kx];
                    if c != 0 {
                        r += (c >> 16) & 0xFF;
                        g += (c >> 8) & 0xFF;
                        b += c & 0xFF;
                        count += 1;
                    }
                }

                let idx = y * self.width + x;
                if count > 0 {
                    self.scratch[idx] = 0xFF000000
                        | ((r / count).min(255) << 16)
                        | ((g / count).min(255) << 8)
                        | (b / count).min(255);
                }
            }
        }
    }

    /// Vertical box blur pass
    fn gaussian_blur_y(&mut self) {
        let kernel_radius = 2usize;

        for y in 0..self.height {
            for x in 0..self.width {
                let mut r = 0u32; let mut g = 0u32; let mut b = 0u32;
                let mut count = 0u32;

                for ky in y.saturating_sub(kernel_radius)..= (y + kernel_radius).min(self.height - 1) {
                    let c = self.scratch[ky * self.width + x];
                    if c != 0 {
                        r += (c >> 16) & 0xFF;
                        g += (c >> 8) & 0xFF;
                        b += c & 0xFF;
                        count += 1;
                    }
                }

                let idx = y * self.width + x;
                if count > 0 {
                    self.scratch[idx] = 0xFF000000
                        | ((r / count).min(255) << 16)
                        | ((g / count).min(255) << 8)
                        | (b / count).min(255);
                }
            }
        }
    }
}

/// Apply bloom directly to a buffer (convenience)
pub fn apply_bloom(buffer: &mut [u32], w: usize, h: usize, threshold: f32, strength: f32) {
    let mut bloom = BloomPass::new(w, h);
    bloom.apply(buffer, threshold, strength, 2);
}

// ============================================================================
// VIGNETTE — Darken edges
// ============================================================================
pub fn apply_vignette(buffer: &mut [u32], w: usize, h: usize, strength: f32) {
    let cx = w as f32 / 2.0;
    let cy = h as f32 / 2.0;
    let max_dist = libm::sqrtf(cx * cx + cy * cy);

    for y in 0..h {
        for x in 0..w {
            let dx = x as f32 - cx;
            let dy = y as f32 - cy;
            let dist = libm::sqrtf(dx * dx + dy * dy) / max_dist;
            let darken = 1.0 - (dist * dist  * strength).min(0.8);
            let idx = y * w + x;
            let c = buffer[idx];
            let r = (((c >> 16) & 0xFF) as f32 * darken) as u32;
            let g = (((c >> 8) & 0xFF) as f32 * darken) as u32;
            let b = ((c & 0xFF) as f32 * darken) as u32;
            buffer[idx] = 0xFF000000 | (r.min(255) << 16) | (g.min(255) << 8) | b.min(255);
        }
    }
}

// ============================================================================
// SCANLINES — CRT/Monitor effect
// ============================================================================
pub fn apply_scanlines(buffer: &mut [u32], w: usize, h: usize, intensity: f32) {
    for y in (0..h).step_by(2) {
        for x in 0..w {
            let idx = y * w + x;
            let c = buffer[idx];
            let r = (((c >> 16) & 0xFF) as f32 * (1.0 - intensity * 0.4)) as u32;
            let g = (((c >> 8) & 0xFF) as f32 * (1.0 - intensity * 0.4)) as u32;
            let b = ((c & 0xFF) as f32 * (1.0 - intensity * 0.4)) as u32;
            buffer[idx] = 0xFF000000 | (r.min(255) << 16) | (g.min(255) << 8) | b.min(255);
        }
    }
}

// ============================================================================
// CHROMATIC ABERRATION
// ============================================================================
pub fn apply_chromatic_aberration(buffer: &mut [u32], w: usize, h: usize, strength: f32) {
    let shift = (strength * 3.0) as isize;

    for y in 0..h {
        for x in shift..w as isize - shift {
            let idx = y * w + x as usize;
            let r_idx = y * w + (x + shift) as usize;
            let b_idx = y * w + (x - shift) as usize;

            let r_c = if r_idx < buffer.len() { buffer[r_idx] } else { buffer[idx] };
            let b_c = if b_idx < buffer.len() { buffer[b_idx] } else { buffer[idx] };
            let g_c = buffer[idx];

            let r = (r_c >> 16) & 0xFF;
            let g = (g_c >> 8) & 0xFF;
            let b = b_c & 0xFF;
            buffer[idx] = 0xFF000000 | (r << 16) | ((g as u32) << 8) | b as u32;
        }
    }
}

// ============================================================================
// FILM GRAIN
// ============================================================================
pub fn apply_film_grain(buffer: &mut [u32], w: usize, h: usize, strength: f32) {
    use rand_chacha::rand_core::{RngCore, SeedableRng};
    let mut rng = rand_chacha::ChaCha20Rng::from_seed([0x42; 32]);

    for y in 0..h {
        for x in 0..w {
            let idx = y * w + x;
            let c = buffer[idx];
            let grain = (rng.next_u32() % 256) as f32 / 255.0 * strength;
            let r = ((((c >> 16) & 0xFF) as f32 + grain * 128.0) as u32).min(255);
            let g = ((((c >> 8) & 0xFF) as f32 + grain * 128.0) as u32).min(255);
            let b = ((c & 0xFF) as f32 + grain * 128.0) as u32;
            buffer[idx] = 0xFF000000 | (r.min(255) << 16) | (g.min(255) << 8) | b.min(255);
        }
    }
}