//! ============================================================================
//! JARVIS OS Deferred Shading Pipeline — G-Buffer + Light Pass
//! ============================================================================
//! Implements:
//!   - G-buffer encoding (albedo, normal, depth, metallic-roughness, emissive)
//!   - Decoupled geometry pass + light accumulation pass
//!   - Screen-Space Ambient Occlusion (SSAO)
//!   - Directional light, point light, spot light accumulation
//! ============================================================================

use super::Vec3;
use alloc::vec::Vec;

// ============================================================================
// G-BUFFER ENCODING
// ============================================================================
// We encode G-buffer into compact u32 arrays for memory efficiency
//
// GBUF_ALBEDO:   ARGB (alpha unused, R=alb_r, G=alb_g, B=alb_b)
// GBUF_NORMAL:   A=metallic(8), R=nrm_x*128+128, G=nrm_y*128+128, B=nrm_z*128+128
// GBUF_DEPTH:    A=roughness(8), R/G/B=encoding of linear depth (24 bits)
// GBUF_EMISSIVE: ARGB emissive color * intensity
// GBUF_METAL_ROUGH: A=ao(8), R=metallic(8), G=roughness(8), B=unused

const GBUF_ALBEDO: usize = 0;
const GBUF_NORMAL: usize = 1;
const GBUF_DEPTH: usize = 2;
const GBUF_EMISSIVE: usize = 3;
const GBUF_COUNT: usize = 4;

const MAX_DEPTH_F: f32 = 16777215.0; // 0xFFFFFF as float

pub struct DeferredPipeline {
    pub width: usize,
    pub height: usize,
    pub gbuffer: Vec<[u32; GBUF_COUNT]>,
    pub output: Vec<u32>,
}

impl DeferredPipeline {
    pub fn new(width: usize, height: usize) -> Self {
        let pixels = width * height;
        Self {
            width,
            height,
            gbuffer: alloc::vec![[0u32; GBUF_COUNT]; pixels],
            output: alloc::vec![0xFF08080Eu32; pixels],
        }
    }

    /// Clear G-buffer for new frame
    pub fn clear(&mut self) {
        let pixels = self.width * self.height;
        for i in 0..pixels {
            self.gbuffer[i] = [0, 0, 0, 0];
        }
    }

    /// Write a pixel's G-buffer data
    pub fn write_gbuffer(
        &mut self,
        x: usize,
        y: usize,
        albedo: u32,
        normal: Vec3,
        depth: f32,
        metallic: f32,
        roughness: f32,
        _ao: f32,
        emissive: u32,
    ) {
        let idx = y * self.width + x;
        if idx >= self.gbuffer.len() {
            return;
        }

        // Albedo: ARGB
        self.gbuffer[idx][GBUF_ALBEDO] = albedo;

        // Normal: encode as A=metallic, RGB=normal*128+128
        let nx = ((normal.x * 127.0 + 128.0) as u8);
        let ny = ((normal.y * 127.0 + 128.0) as u8);
        let nz = ((normal.z * 127.0 + 128.0) as u8);
        let m = (metallic * 255.0) as u8;
        self.gbuffer[idx][GBUF_NORMAL] =
            (m as u32) << 24 | (nx as u32) << 16 | (ny as u32) << 8 | nz as u32;

        // Depth + roughness (roughness in alpha channel)
        let depth_bits = (depth * MAX_DEPTH_F).min(MAX_DEPTH_F) as u32;
        let r_bits = (roughness * 255.0) as u32;
        self.gbuffer[idx][GBUF_DEPTH] = (r_bits << 24)
            | ((depth_bits >> 16) & 0xFF) << 16
            | ((depth_bits >> 8) & 0xFF) << 8
            | (depth_bits & 0xFF);

        // Emissive
        self.gbuffer[idx][GBUF_EMISSIVE] = emissive;
    }

    /// Read G-buffer data at a pixel
    pub fn read_gbuffer(&self, idx: usize) -> (u32, Vec3, f32, f32, f32, f32, u32) {
        if idx >= self.gbuffer.len() {
            return (0, Vec3::zero(), 1.0, 0.0, 1.0, 1.0, 0);
        }

        let albedo = self.gbuffer[idx][GBUF_ALBEDO];
        let normal_packed = self.gbuffer[idx][GBUF_NORMAL];
        let depth_packed = self.gbuffer[idx][GBUF_DEPTH];
        let emissive = self.gbuffer[idx][GBUF_EMISSIVE];

        // Decode normal
        let metallic = ((normal_packed >> 24) & 0xFF) as f32 / 255.0;
        let nx = ((normal_packed >> 16) & 0xFF) as f32 / 127.0 - 1.0;
        let ny = ((normal_packed >> 8) & 0xFF) as f32 / 127.0 - 1.0;
        let nz = (normal_packed & 0xFF) as f32 / 127.0 - 1.0;
        let normal = Vec3::new(nx, ny, nz).normalize();

        // Decode depth
        let roughness = ((depth_packed >> 24) & 0xFF) as f32 / 255.0;
        let depth_bits = ((depth_packed >> 16) & 0xFF) << 16
            | ((depth_packed >> 8) & 0xFF) << 8
            | (depth_packed & 0xFF);
        let depth = depth_bits as f32 / MAX_DEPTH_F;

        // AO from emissive alpha (we pack AO there too — simplified)
        let ao = ((emissive >> 24) & 0xFF) as f32 / 255.0;

        (albedo, normal, depth, metallic, roughness, ao, emissive)
    }

    // ========================================================================
    // LIGHT ACCUMULATION PASS
    // ========================================================================

    /// Accumulate directional light over entire buffer
    pub fn accumulate_directional_light(
        &mut self,
        dir: Vec3,
        r: f32,
        g: f32,
        b: f32,
        intensity: f32,
    ) {
        let light_dir = dir.normalize();
        let light_color = Vec3::new(r, g, b);

        for y in 0..self.height {
            for x in 0..self.width {
                let idx = y * self.width + x;
                let (albedo, normal, _depth, metallic, roughness, ao, _emissive) =
                    self.read_gbuffer(idx);

                // Simple NdotL diffuse + specular
                let n_dot_l = normal.dot(light_dir).max(0.0);
                if n_dot_l <= 0.001 {
                    continue;
                }

                let _n_dot_v = if idx > 0 {
                    // Approximate view as (0, 0, 1) in screen space
                    1.0
                } else {
                    1.0
                };

                // Decode albedo
                let alb_r = ((albedo >> 16) & 0xFF) as f32 / 255.0;
                let alb_g = ((albedo >> 8) & 0xFF) as f32 / 255.0;
                let alb_b = (albedo & 0xFF) as f32 / 255.0;

                // Simple diffuse
                let kd = (1.0 - metallic).max(0.02);
                let diff_color = Vec3::new(
                    alb_r * kd * (1.0 / core::f32::consts::PI),
                    alb_g * kd * (1.0 / core::f32::consts::PI),
                    alb_b * kd * (1.0 / core::f32::consts::PI),
                );

                // Simple specular (Blinn-Phong approximation)
                let half = normal.add(light_dir).normalize();
                let n_dot_h = normal.dot(half).max(0.0);
                let spec_power = (1.0 - roughness) * 128.0;
                let spec = libm::powf(n_dot_h, spec_power);

                let result = diff_color
                    .add(light_color.scale(spec * 0.5))
                    .scale(n_dot_l * intensity)
                    .scale(ao);

                // Add to output
                let out_r = ((result.x * 255.0) as u32).min(255);
                let out_g = ((result.y * 255.0) as u32).min(255);
                let out_b = ((result.z * 255.0) as u32).min(255);
                let prev = self.output[idx];
                let pr = (prev >> 16) & 0xFF;
                let pg = (prev >> 8) & 0xFF;
                let pb = prev & 0xFF;
                let new_r = (pr + out_r as u32).min(255);
                let new_g = (pg + out_g as u32).min(255);
                let new_b = (pb + out_b as u32).min(255);
                self.output[idx] = 0xFF000000 | (new_r << 16) | (new_g << 8) | new_b;
            }
        }
    }

    /// Accumulate point light (sphere of influence)
    pub fn accumulate_point_light(
        &mut self,
        pos_world: Vec3,
        r: f32,
        g: f32,
        b: f32,
        intensity: f32,
        radius: f32,
    ) {
        // Compute screen-space bounds from world position and radius
        // Simplified: for each pixel, compute world pos from depth
        let light_color = Vec3::new(r, g, b);

        for y in 0..self.height {
            for x in 0..self.width {
                let idx = y * self.width + x;
                let (_albedo, normal, depth, metallic, _roughness, ao, _emissive) =
                    self.read_gbuffer(idx);

                // Skip far pixels
                if depth > 0.99 {
                    continue;
                }

                // Compute approximate world position from depth and screen coord
                let ndc_x = (x as f32 / self.width as f32) * 2.0 - 1.0;
                let ndc_y = (y as f32 / self.height as f32) * 2.0 - 1.0;
                let world_pos = Vec3::new(ndc_x * depth * 10.0, ndc_y * depth * 10.0, depth * 50.0);

                let to_light = pos_world.sub(world_pos);
                let dist = to_light.length();
                if dist > radius {
                    continue;
                }

                let light_dir = to_light.normalize();
                let n_dot_l = normal.dot(light_dir).max(0.0);
                if n_dot_l <= 0.001 {
                    continue;
                }

                // Attenuation
                let atten = 1.0 - (dist / radius);
                let atten = atten * atten;

                // Decode albedo
                let albedo = self.gbuffer[idx][GBUF_ALBEDO];
                let alb_r = ((albedo >> 16) & 0xFF) as f32 / 255.0;
                let alb_g = ((albedo >> 8) & 0xFF) as f32 / 255.0;
                let alb_b = (albedo & 0xFF) as f32 / 255.0;

                let kd_val = (1.0 - metallic).max(0.02);
                let diff = Vec3::new(
                    alb_r * kd_val * n_dot_l,
                    alb_g * kd_val * n_dot_l,
                    alb_b * kd_val * n_dot_l,
                );
                let result = diff.scale(intensity * atten);
                let result = Vec3::new(
                    result.x * light_color.x,
                    result.y * light_color.y,
                    result.z * light_color.z,
                )
                .scale(ao);

                let prev = self.output[idx];
                let out_r = ((result.x * 255.0) as u32 + ((prev >> 16) & 0xFF)).min(255);
                let out_g = ((result.y * 255.0) as u32 + ((prev >> 8) & 0xFF)).min(255);
                let out_b = ((result.z * 255.0) as u32 + (prev & 0xFF)).min(255);
                self.output[idx] = 0xFF000000 | (out_r << 16) | (out_g << 8) | out_b;
            }
        }
    }

    /// Add ambient + IBL (simple ambient light with AO)
    pub fn accumulate_ambient(
        &mut self,
        irr_r: f32,
        irr_g: f32,
        irr_b: f32,
        ambient_intensity: f32,
    ) {
        let ambient = Vec3::new(irr_r, irr_g, irr_b).scale(ambient_intensity);

        for y in 0..self.height {
            for x in 0..self.width {
                let idx = y * self.width + x;
                let albedo = self.gbuffer[idx][GBUF_ALBEDO];
                let depth_packed = self.gbuffer[idx][GBUF_DEPTH];
                let ao = ((self.gbuffer[idx][GBUF_EMISSIVE] >> 24) & 0xFF) as f32 / 255.0;

                // Skip sky/far pixels
                let depth_bits = ((depth_packed >> 16) & 0xFF) << 16
                    | ((depth_packed >> 8) & 0xFF) << 8
                    | (depth_packed & 0xFF);
                let depth = depth_bits as f32 / MAX_DEPTH_F;
                if depth > 0.99 {
                    continue;
                }

                let alb_r = ((albedo >> 16) & 0xFF) as f32 / 255.0;
                let alb_g = ((albedo >> 8) & 0xFF) as f32 / 255.0;
                let alb_b = (albedo & 0xFF) as f32 / 255.0;

                let contrib = {
                    let color = Vec3::new(alb_r * ao, alb_g * ao, alb_b * ao);
                    Vec3::new(
                        color.x * ambient.x,
                        color.y * ambient.y,
                        color.z * ambient.z,
                    )
                };
                let prev = self.output[idx];
                let out_r = ((contrib.x * 255.0) as u32 + ((prev >> 16) & 0xFF)).min(255);
                let out_g = ((contrib.y * 255.0) as u32 + ((prev >> 8) & 0xFF)).min(255);
                let out_b = ((contrib.z * 255.0) as u32 + (prev & 0xFF)).min(255);
                self.output[idx] = 0xFF000000 | (out_r << 16) | (out_g << 8) | out_b;
            }
        }
    }

    /// Get the accumulated output buffer
    pub fn get_output(&self) -> &[u32] {
        &self.output
    }

    /// Blit output to framebuffer
    pub fn blit_to_framebuffer(&self, dst: &mut [u8]) {
        let pixels = self.width * self.height;
        let dst_pixels = dst.len() / 4;
        let copy_count = pixels.min(dst_pixels);

        for i in 0..copy_count {
            let argb = self.output[i];
            let b = (argb & 0xFF) as u8;
            let g = ((argb >> 8) & 0xFF) as u8;
            let r = ((argb >> 16) & 0xFF) as u8;
            // BGR format for framebuffer
            dst[i * 4] = b;
            dst[i * 4 + 1] = g;
            dst[i * 4 + 2] = r;
        }
    }
}

// ============================================================================
// SCREEN-SPACE AMBIENT OCCLUSION (SSAO)
// ============================================================================
pub fn compute_ssao(
    width: usize,
    height: usize,
    gbuffer: &[[u32; 4]],
    strength: f32,
    radius: f32,
) -> Vec<f32> {
    let pixels = width * height;
    let mut ao = alloc::vec![1.0f32; pixels];

    // Simple SSAO: sample neighboring pixels and compare depths
    let kernel = [
        (-1, -1),
        (1, -1),
        (-1, 1),
        (1, 1),
        (0, -2),
        (0, 2),
        (-2, 0),
        (2, 0),
    ];

    for y in 2..height - 2 {
        for x in 2..width - 2 {
            let idx = y * width + x;
            let depth_packed = gbuffer[idx][2];
            let center_depth = (((depth_packed >> 16) & 0xFF) << 16
                | ((depth_packed >> 8) & 0xFF) << 8
                | (depth_packed & 0xFF)) as f32
                / MAX_DEPTH_F;

            if center_depth > 0.99 {
                continue;
            }

            let mut occlusion = 0.0;
            let mut count = 0;

            for &(dx, dy) in &kernel {
                let sx = (x as isize + dx) as usize;
                let sy = (y as isize + dy) as usize;
                let sidx = sy * width + sx;

                let sdepth_packed = gbuffer[sidx][2];
                let sample_depth = (((sdepth_packed >> 16) & 0xFF) << 16
                    | ((sdepth_packed >> 8) & 0xFF) << 8
                    | (sdepth_packed & 0xFF)) as f32
                    / MAX_DEPTH_F;

                let depth_delta = center_depth - sample_depth;
                if depth_delta > radius * 0.01 {
                    occlusion += 1.0;
                }
                count += 1;
            }

            let ao_val = 1.0 - (occlusion / count as f32) * strength;
            ao[idx] = ao_val.max(0.0);
        }
    }

    ao
}
