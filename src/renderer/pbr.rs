//! ============================================================================
//! JARVIS OS PBR Material System — Physically Based Rendering
//! ============================================================================
//! Implements:
//!   - Cook-Torrance BRDF with GGX (Trowbridge-Reitz) NDF
//!   - Schlick-GGX geometry function (Smith)
//!   - Schlick Fresnel approximation
//!   - Image-Based Lighting via irradiance spherical harmonics
//!   - Multiple light types: directional, point, spot
//! ============================================================================

use super::Vec3;
use crate::vga_buffer::Color;

// ============================================================================
// PBR MATERIAL
// ============================================================================
#[derive(Debug, Clone, Copy)]
pub struct PbrMaterial {
    pub albedo: Vec3,
    pub metallic: f32,
    pub roughness: f32,
    pub ao: f32,          // Ambient occlusion
    pub emissive: f32,    // Emissive intensity (0 = none)
    pub emissive_color: Vec3,
    pub opacity: f32,
}

impl PbrMaterial {
    pub fn new() -> Self {
        Self {
            albedo: Vec3::new(0.5, 0.5, 0.5),
            metallic: 0.0,
            roughness: 0.5,
            ao: 1.0,
            emissive: 0.0,
            emissive_color: Vec3::new(1.0, 1.0, 1.0),
            opacity: 1.0,
        }
    }

    pub fn metal(r: f32, g: f32, b: f32, roughness: f32) -> Self {
        Self {
            albedo: Vec3::new(r, g, b),
            metallic: 1.0,
            roughness,
            ..Self::new()
        }
    }

    pub fn dielectric(r: f32, g: f32, b: f32, roughness: f32) -> Self {
        Self {
            albedo: Vec3::new(r, g, b),
            metallic: 0.0,
            roughness,
            ..Self::new()
        }
    }

    pub fn emissive_mat(r: f32, g: f32, b: f32, intensity: f32) -> Self {
        Self {
            emissive: intensity,
            emissive_color: Vec3::new(r, g, b),
            ..Self::new()
        }
    }
}

// ============================================================================
// LIGHT TYPES
// ============================================================================
#[derive(Debug, Clone, Copy)]
pub enum LightType {
    Directional { direction: Vec3 },
    Point { position: Vec3, radius: f32 },
    Spot { position: Vec3, direction: Vec3, inner_angle: f32, outer_angle: f32, radius: f32 },
}

#[derive(Debug, Clone, Copy)]
pub struct Light {
    pub light_type: LightType,
    pub color: Vec3,
    pub intensity: f32,
}

impl Light {
    pub fn directional(dir: Vec3, r: f32, g: f32, b: f32, intensity: f32) -> Self {
        Self {
            light_type: LightType::Directional { direction: dir.normalize() },
            color: Vec3::new(r, g, b),
            intensity,
        }
    }

    pub fn point(pos: Vec3, r: f32, g: f32, b: f32, intensity: f32, radius: f32) -> Self {
        Self {
            light_type: LightType::Point { position: pos, radius },
            color: Vec3::new(r, g, b),
            intensity,
        }
    }

    pub fn spot(pos: Vec3, dir: Vec3, r: f32, g: f32, b: f32, intensity: f32,
                inner: f32, outer: f32, radius: f32) -> Self {
        Self {
            light_type: LightType::Spot {
                position: pos, direction: dir.normalize(),
                inner_angle: inner, outer_angle: outer, radius,
            },
            color: Vec3::new(r, g, b),
            intensity,
        }
    }
}

// ============================================================================
// SPHERICAL HARMONICS for IBL
// ============================================================================
#[derive(Debug, Clone, Copy)]
pub struct SphericalHarmonics {
    /// 9 coefficients for 3 bands (L00..L22), each is Vec3 (RGB)
    pub c: [Vec3; 9],
}

impl SphericalHarmonics {
    pub fn new() -> Self {
        Self { c: [Vec3::zero(); 9] }
    }

    /// Evaluate SH at given direction
    pub fn evaluate(&self, dir: Vec3) -> Vec3 {
        let x = dir.x;
        let y = dir.y;
        let z = dir.z;

        // Band 0 (L00)
        let mut result = self.c[0].scale(0.282095);

        // Band 1 (L1m)
        result = result.add(self.c[1].scale(0.488603 * y));
        result = result.add(self.c[2].scale(0.488603 * z));
        result = result.add(self.c[3].scale(0.488603 * x));

        // Band 2 (L2m)
        result = result.add(self.c[4].scale(1.092548 * x * y));
        result = result.add(self.c[5].scale(1.092548 * y * z));
        result = result.add(self.c[6].scale(0.315392 * (3.0 * z * z - 1.0)));
        result = result.add(self.c[7].scale(1.092548 * x * z));
        result = result.add(self.c[8].scale(0.546274 * (x * x - y * y)));

        result
    }

    /// Create a simple sky/ambient SH from a color
    pub fn from_ambient_color(r: f32, g: f32, b: f32) -> Self {
        let mut sh = Self::new();
        let c = Vec3::new(r, g, b);
        sh.c[0] = c.scale(1.0 / 0.282095);
        sh
    }

    /// Create SH from hemispherical gradient (top color, bottom color)
    pub fn from_hemisphere(top: Vec3, bottom: Vec3) -> Self {
        let mut sh = Self::new();
        // L00 - average
        sh.c[0] = Vec3::new(
            (top.x + bottom.x) * 0.5,
            (top.y + bottom.y) * 0.5,
            (top.z + bottom.z) * 0.5,
        ).scale(1.0 / 0.282095);

        // L10 - vertical gradient (Y direction)
        let diff = top.sub(bottom);
        sh.c[1] = diff.scale(0.5 / 0.488603);

        sh
    }
}

// ============================================================================
// PBR BRDF EVALUATION
// ============================================================================

/// Normal Distribution Function — GGX/Trowbridge-Reitz
fn ndf_ggx(n_dot_h: f32, roughness: f32) -> f32 {
    let a = roughness * roughness;
    let a2 = a * a;
    let n_dot_h2 = n_dot_h * n_dot_h;
    let denom = n_dot_h2 * (a2 - 1.0) + 1.0;
    a2 / (core::f32::consts::PI * denom * denom)
}

/// Geometry function — Schlick-GGX
fn geometry_schlick_ggx(n_dot_v: f32, roughness: f32) -> f32 {
    let a = roughness * roughness;
    let k = (a + 1.0) / 8.0; // IBL variant: k = a/2 for direct
    n_dot_v / (n_dot_v * (1.0 - k) + k)
}

/// Smith geometry function
fn geometry_smith(n_dot_v: f32, n_dot_l: f32, roughness: f32) -> f32 {
    let ggx_v = geometry_schlick_ggx(n_dot_v, roughness);
    let ggx_l = geometry_schlick_ggx(n_dot_l, roughness);
    ggx_v * ggx_l
}

/// Fresnel — Schlick approximation
fn fresnel_schlick(cos_theta: f32, f0: Vec3) -> Vec3 {
    let one_minus_cos = 1.0 - cos_theta;
    let one_minus_cos2 = one_minus_cos * one_minus_cos;
    let one_minus_cos5 = one_minus_cos2 * one_minus_cos2 * one_minus_cos;
    f0.add(Vec3::new(1.0, 1.0, 1.0).sub(f0).scale(one_minus_cos5))
}

/// Evaluate Cook-Torrance BRDF at a shading point
/// Returns the reflected radiance contribution
pub fn eval_brdf(normal: Vec3, view_dir: Vec3, light_dir: Vec3,
                 material: &PbrMaterial, light: &Light) -> Vec3 {
    let n = normal.normalize();
    let v = view_dir.normalize();
    let l = light_dir.normalize();
    let h = l.add(v).normalize();

    let n_dot_v = (n.dot(v)).max(0.001);
    let n_dot_l = (n.dot(l)).max(0.001);
    let n_dot_h = (n.dot(h)).max(0.001);
    let h_dot_v = (h.dot(v)).max(0.001);

    if n_dot_l <= 0.0 {
        return Vec3::zero();
    }

    let f0_base = Vec3::new(0.04, 0.04, 0.04);
    let f0 = Vec3::lerp(f0_base, material.albedo, material.metallic);

    // BRDF terms
    let d = ndf_ggx(n_dot_h, material.roughness);
    let g = geometry_smith(n_dot_v, n_dot_l, material.roughness);
    let f = fresnel_schlick(h_dot_v, f0);

    // Specular BRDF
    let spec_denom = 4.0 * n_dot_v * n_dot_l;
    let specular = f.scale(d * g / spec_denom.max(0.001));

    // Diffuse BRDF (Lambertian)
    let one = Vec3::new(1.0, 1.0, 1.0);
    let kd = one.sub(f).scale(1.0 - material.metallic);
    let diffuse = material.albedo.scale(kd.x * (1.0 / core::f32::consts::PI));

    // Combine
    let brdf = diffuse.add(specular);
    let radiance = brdf.scale(n_dot_l);

    // Light color & intensity
    let radiance = radiance.scale(light.intensity);
    Vec3::new(radiance.x * light.color.x, radiance.y * light.color.y, radiance.z * light.color.z)
}

/// Evaluate direct lighting from all lights
pub fn eval_direct_lights(position: Vec3, normal: Vec3, view_dir: Vec3,
                          material: &PbrMaterial, lights: &[Light]) -> Vec3 {
    let mut result = Vec3::zero();

    for light in lights {
        let light_dir = match light.light_type {
            LightType::Directional { direction } => direction.normalize(),
            LightType::Point { position: pos, radius: _ } => {
                pos.sub(position).normalize()
            }
            LightType::Spot { position: pos, direction: dir, inner_angle, outer_angle, radius: _ } => {
                let to_light = pos.sub(position).normalize();
                let cos_outer = libm::cosf(outer_angle);
                let cos_inner = libm::cosf(inner_angle);
                let cos_theta = dir.dot(to_light);

                // Spot falloff
                let falloff = ((cos_theta - cos_outer) / (cos_inner - cos_outer)).max(0.0).min(1.0);
                let atten = falloff * falloff;

                result = result.add(eval_brdf(normal, view_dir, to_light, material, light).scale(atten));
                continue;
            }
        };

        result = result.add(eval_brdf(normal, view_dir, light_dir, material, light));
    }

    result
}

/// Evaluate IBL (Image-Based Lighting) contribution
pub fn eval_ibl(normal: Vec3, view_dir: Vec3, material: &PbrMaterial,
                irradiance_sh: &SphericalHarmonics) -> Vec3 {
    let n = normal.normalize();
    let v = view_dir.normalize();

    // Diffuse IBL: sample irradiance in normal direction
    let irradiance = irradiance_sh.evaluate(n);

    // Specular IBL: reflect view vector around normal
    let n_dot_v = n.dot(v);
    let reflection = v.sub(n.scale(2.0 * n_dot_v)).normalize();
    let specular_ibl = irradiance_sh.evaluate(reflection);

    // Fresnel for IBL
    let f0_base2 = Vec3::new(0.04, 0.04, 0.04);
    let f0 = Vec3::lerp(f0_base2, material.albedo, material.metallic);
    let f = fresnel_schlick(n_dot_v.max(0.0), f0);

    let one2 = Vec3::new(1.0, 1.0, 1.0);
    let kd = one2.sub(f).scale(1.0 - material.metallic).scale(1.0 / core::f32::consts::PI);
    let ks = f;

    // Diffuse IBL * albedo * AO
    let diffuse = {
        let s = irradiance.scale(kd.x);
        Vec3::new(s.x * material.albedo.x, s.y * material.albedo.y, s.z * material.albedo.z)
    }.scale(material.ao);

    // Specular IBL (simplified: use irradiance color as rough specular)
    let specular = specular_ibl.scale(ks.x).scale(material.roughness * 0.5);

    // Emissive
    let emissive = material.emissive_color.scale(material.emissive);

    diffuse.add(specular).add(emissive)
}

/// Full PBR evaluation: direct + IBL
pub fn shade_pbr(position: Vec3, normal: Vec3, view_dir: Vec3,
                 material: &PbrMaterial, lights: &[Light],
                 irradiance_sh: &SphericalHarmonics) -> Vec3 {
    let direct = eval_direct_lights(position, normal, view_dir, material, lights);
    let ibl = eval_ibl(normal, view_dir, material, irradiance_sh);
    direct.add(ibl)
}

/// Convert PBR output to Color (no tone mapping)
pub fn pbr_to_color(value: Vec3) -> Color {
    let r = (value.x * 255.0).max(0.0).min(255.0) as u8;
    let g = (value.y * 255.0).max(0.0).min(255.0) as u8;
    let b = (value.z * 255.0).max(0.0).min(255.0) as u8;
    Color { r, g, b }
}

/// Convert PBR output to ARGB u32
pub fn pbr_to_argb(value: Vec3) -> u32 {
    let r = (value.x * 255.0).max(0.0).min(255.0) as u8;
    let g = (value.y * 255.0).max(0.0).min(255.0) as u8;
    let b = (value.z * 255.0).max(0.0).min(255.0) as u8;
    0xFF000000 | (r as u32) << 16 | (g as u32) << 8 | b as u32
}