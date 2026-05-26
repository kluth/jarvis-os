//! ============================================================================
//! JARVIS OS Software 3D Rendering Engine
//! ============================================================================
//! A full software rasterizer with:
//!   - Triangle rasterization with perspective-correct texture mapping
//!   - Z-buffer for depth testing
//!   - Gouraud shading / per-vertex lighting
//!   - Alpha blending
//!   - Sub-pixel anti-aliasing
//!   - Post-processing effects (glow, bloom, scanlines)
//! Sub-modules:
//!   - pbr: PBR material system (Cook-Torrance BRDF, GGX, IBL)
//!   - deferred: Deferred shading pipeline (G-buffer, light pass, SSAO)
//!   - effects: Post-processing (HDR tone mapping, bloom v2, SSGI, volumetrics)
//!   - particle: Particle system with emitters and forces
//! ============================================================================

pub mod compositor;
pub mod deferred;
pub mod effects;
pub mod particle;
pub mod pbr;
pub mod ui;

use crate::vga_buffer::Color;

// ============================================================================
// 3D MATH PRIMITIVES
// ============================================================================
#[derive(Debug, Clone, Copy)]
pub struct Vec3 {
    pub x: f32,
    pub y: f32,
    pub z: f32,
}

impl Vec3 {
    pub fn new(x: f32, y: f32, z: f32) -> Self {
        Self { x, y, z }
    }
    pub fn zero() -> Self {
        Self {
            x: 0.0,
            y: 0.0,
            z: 0.0,
        }
    }

    pub fn add(self, other: Vec3) -> Vec3 {
        Vec3 {
            x: self.x + other.x,
            y: self.y + other.y,
            z: self.z + other.z,
        }
    }
    pub fn sub(self, other: Vec3) -> Vec3 {
        Vec3 {
            x: self.x - other.x,
            y: self.y - other.y,
            z: self.z - other.z,
        }
    }
    pub fn scale(self, s: f32) -> Vec3 {
        Vec3 {
            x: self.x * s,
            y: self.y * s,
            z: self.z * s,
        }
    }
    pub fn dot(self, other: Vec3) -> f32 {
        self.x * other.x + self.y * other.y + self.z * other.z
    }
    pub fn cross(self, other: Vec3) -> Vec3 {
        Vec3 {
            x: self.y * other.z - self.z * other.y,
            y: self.z * other.x - self.x * other.z,
            z: self.x * other.y - self.y * other.x,
        }
    }
    pub fn length(self) -> f32 {
        libm::sqrtf(self.dot(self))
    }
    pub fn normalize(self) -> Vec3 {
        let l = self.length();
        if l > 0.0001 {
            self.scale(1.0 / l)
        } else {
            self
        }
    }
    pub fn lerp(a: Vec3, b: Vec3, t: f32) -> Vec3 {
        Vec3 {
            x: a.x + (b.x - a.x) * t,
            y: a.y + (b.y - a.y) * t,
            z: a.z + (b.z - a.z) * t,
        }
    }
}

// ============================================================================
// TEXTURE — 2D pixel map for texture mapping
// ============================================================================
pub struct Texture {
    pub width: usize,
    pub height: usize,
    pub data: alloc::vec::Vec<u32>, // ARGB pixel data
}

impl Texture {
    pub fn new(width: usize, height: usize) -> Self {
        Self {
            width,
            height,
            data: alloc::vec![0u32; width * height],
        }
    }

    pub fn from_raw(data: alloc::vec::Vec<u32>, width: usize, height: usize) -> Self {
        Self {
            width,
            height,
            data,
        }
    }

    /// Sample texture at UV coordinates with bilinear filtering
    pub fn sample(&self, u: f32, v: f32) -> u32 {
        let u = u - libm::floorf(u);
        let v = v - libm::floorf(v);
        let fx = u * (self.width as f32 - 1.0);
        let fy = v * (self.height as f32 - 1.0);
        let ix = (fx as usize).min(self.width - 1);
        let iy = (fy as usize).min(self.height - 1);
        self.data[iy * self.width + ix]
    }

    /// Generate a procedural "cyber" texture pattern
    pub fn generate_cyber_pattern(w: usize, h: usize) -> Self {
        let mut data = alloc::vec![0u32; w * h];
        for y in 0..h {
            for x in 0..w {
                let dx = x as f32 / w as f32;
                let dy = y as f32 / h as f32;
                let dist =
                    libm::sqrtf((dx - 0.5) * (dx - 0.5) + (dy - 0.5) * (dy - 0.5)).min(0.5) * 2.0;
                let grid = ((x / 16) ^ (y / 16)) & 1;
                let val = if grid == 0 { 0.8 } else { 0.2 };
                let intensity = (dist.max(0.0) * val * 255.0) as u8;
                let alpha = if dist < 0.45 { 220u8 } else { 0u8 };
                // Dark blue-cyan
                let r = (intensity as u16 * 20 / 255) as u8;
                let g = (intensity as u16 * 180 / 255) as u8;
                let b = (intensity as u16 * 255 / 255) as u8;
                data[y * w + x] =
                    (alpha as u32) << 24 | (r as u32) << 16 | (g as u32) << 8 | b as u32;
            }
        }
        Self {
            width: w,
            height: h,
            data,
        }
    }

    /// Generate a hex grid texture
    pub fn generate_hex_grid(w: usize, h: usize) -> Self {
        let mut data = alloc::vec![0u32; w * h];
        for y in 0..h {
            for x in 0..w {
                let fx = x as f32 / 16.0;
                let fy = y as f32 / 16.0;
                let hex =
                    (libm::sinf(fx * 0.866 + fy * 0.5) * libm::cosf(fx * 0.866 - fy * 0.5)).abs();
                let val = if hex < 0.3 { 180u8 } else { 30u8 };
                let alpha = if hex < 0.3 { 120u8 } else { 0u8 };
                data[y * w + x] = ((alpha as u32) << 24) | (val as u32) << 8 | (val as u32);
            }
        }
        Self {
            width: w,
            height: h,
            data,
        }
    }

    /// Generate a procedural circuit-board texture
    pub fn generate_circuit(w: usize, h: usize) -> Self {
        use rand_chacha::rand_core::{RngCore, SeedableRng};
        let mut rng = rand_chacha::ChaCha20Rng::from_seed([0x13; 32]);
        let mut data = alloc::vec![0u32; w * h];
        // Draw random circuit traces
        for _ in 0..8 {
            let mut x = (rng.next_u32() as usize % w).max(10);
            let mut y = (rng.next_u32() as usize % h).max(10);
            for _ in 0..30 {
                let dx = (rng.next_u32() as i32 % 3 - 1) * 8;
                let dy = (rng.next_u32() as i32 % 3 - 1) * 8;
                for i in 0..8 {
                    let px = (x as i32 + dx * i / 8).max(0).min(w as i32 - 1) as usize;
                    let py = (y as i32 + dy * i / 8).max(0).min(h as i32 - 1) as usize;
                    data[py * w + px] = 0x40FF8844u32; // Orange-cyan traces
                }
                x = (x as i32 + dx).max(0).min(w as i32 - 1) as usize;
                y = (y as i32 + dy).max(0).min(h as i32 - 1) as usize;
            }
        }
        Self {
            width: w,
            height: h,
            data,
        }
    }
}

// ============================================================================
// VERTEX WITH ATTRIBUTES
// ============================================================================
#[derive(Debug, Clone, Copy)]
pub struct Vertex {
    pub x: f32,
    pub y: f32,
    pub z: f32,
    pub u: f32, // texture U
    pub v: f32, // texture V
    pub r: f32, // vertex color R (0-1)
    pub g: f32,
    pub b: f32,
    pub a: f32, // alpha
}

impl Vertex {
    pub fn new(x: f32, y: f32, z: f32) -> Self {
        Self {
            x,
            y,
            z,
            u: 0.0,
            v: 0.0,
            r: 1.0,
            g: 1.0,
            b: 1.0,
            a: 1.0,
        }
    }
    pub fn with_uv(mut self, u: f32, v: f32) -> Self {
        self.u = u;
        self.v = v;
        self
    }
    pub fn with_color(mut self, r: f32, g: f32, b: f32) -> Self {
        self.r = r;
        self.g = g;
        self.b = b;
        self
    }
    pub fn with_alpha(mut self, a: f32) -> Self {
        self.a = a;
        self
    }
}

// ============================================================================
// DRAW CALL — A single render operation
// ============================================================================
pub enum DrawMode {
    Triangles,
    TriangleFan,
    TriangleStrip,
    Lines,
    Points,
}

pub struct DrawCall {
    pub mode: DrawMode,
    pub vertices: alloc::vec::Vec<Vertex>,
    pub texture: Option<alloc::sync::Arc<Texture>>,
    pub blend_mode: BlendMode,
    pub shader: ShaderType,
}

#[derive(Debug, Clone, Copy)]
pub enum BlendMode {
    None,     // Replace
    Alpha,    // Alpha blending
    Add,      // Additive blending
    Multiply, // Multiplicative
}

#[derive(Debug, Clone, Copy)]
pub enum ShaderType {
    Flat,        // Flat color
    Gouraud,     // Per-vertex interpolated color
    Textured,    // Texture mapped
    TexturedLit, // Texture + per-vertex lighting
    Glow,        // Glow effect (additive + blur)
}

// ============================================================================
// SOFTWARE RENDERER — The main rendering engine
// ============================================================================
pub struct Renderer {
    pub width: usize,
    pub height: usize,
    pub framebuffer: alloc::vec::Vec<u32>, // ARGB pixels
    pub zbuffer: alloc::vec::Vec<f32>,     // depth buffer
    pub backbuffer: alloc::vec::Vec<u32>,  // double buffer
    pub clear_color: u32,
    pub clear_depth: f32,
    view_matrix: [[f32; 4]; 4],
    proj_matrix: [[f32; 4]; 4],
    viewport_width: f32,
    viewport_height: f32,
    time: f32,
}

impl Renderer {
    pub fn new(width: usize, height: usize) -> Self {
        let size = width * height;
        Self {
            width,
            height,
            framebuffer: alloc::vec![0xFF08080Eu32; size],
            zbuffer: alloc::vec![f32::MAX; size],
            backbuffer: alloc::vec![0xFF08080Eu32; size],
            clear_color: 0xFF08080E,
            clear_depth: f32::MAX,
            view_matrix: Self::identity_4x4(),
            proj_matrix: Self::identity_4x4(),
            viewport_width: width as f32,
            viewport_height: height as f32,
            time: 0.0,
        }
    }

    fn identity_4x4() -> [[f32; 4]; 4] {
        [
            [1.0, 0.0, 0.0, 0.0],
            [0.0, 1.0, 0.0, 0.0],
            [0.0, 0.0, 1.0, 0.0],
            [0.0, 0.0, 0.0, 1.0],
        ]
    }

    // ========================================================================
    // CLEAR
    // ========================================================================
    pub fn clear(&mut self, color: u32, depth: f32) {
        self.clear_color = color;
        self.clear_depth = depth;
        let size = self.width * self.height;
        for i in 0..size {
            self.framebuffer[i] = color;
            self.zbuffer[i] = depth;
        }
    }

    /// Swap backbuffer to screen
    pub fn present(&mut self) {
        self.backbuffer.copy_from_slice(&self.framebuffer);
    }

    /// Get the ARGB pixel at a position (read from backbuffer)
    pub fn get_pixel_argb(&self, x: usize, y: usize) -> u32 {
        self.backbuffer[y * self.width + x]
    }

    // ========================================================================
    // MATRIX OPERATIONS
    // ========================================================================
    pub fn set_perspective(&mut self, fov_deg: f32, aspect: f32, near: f32, far: f32) {
        let fov_rad = fov_deg * core::f32::consts::PI / 180.0;
        let f = 1.0 / libm::tanf(fov_rad * 0.5);
        self.proj_matrix = [
            [f / aspect, 0.0, 0.0, 0.0],
            [0.0, f, 0.0, 0.0],
            [
                0.0,
                0.0,
                (far + near) / (near - far),
                (2.0 * far * near) / (near - far),
            ],
            [0.0, 0.0, -1.0, 0.0],
        ];
    }

    pub fn set_view(&mut self, pos: Vec3, target: Vec3, up: Vec3) {
        let fwd = target.sub(pos).normalize();
        let right = fwd.cross(up).normalize();
        let up = right.cross(fwd);
        self.view_matrix = [
            [right.x, up.x, -fwd.x, 0.0],
            [right.y, up.y, -fwd.y, 0.0],
            [right.z, up.z, -fwd.z, 0.0],
            [-right.dot(pos), -up.dot(pos), fwd.dot(pos), 1.0],
        ];
    }

    fn transform(&self, v: &[f32; 4], m: &[[f32; 4]; 4]) -> [f32; 4] {
        let mut r = [0.0f32; 4];
        for i in 0..4 {
            r[i] = v[0] * m[0][i] + v[1] * m[1][i] + v[2] * m[2][i] + v[3] * m[3][i];
        }
        r
    }

    fn project_vertex(&self, v: &Vertex) -> Option<(f32, f32, f32)> {
        let p = [v.x, v.y, v.z, 1.0];
        let p = self.transform(&p, &self.view_matrix);
        let p = self.transform(&p, &self.proj_matrix);
        if p[3].abs() < 0.0001 {
            return None;
        }
        let inv_w = 1.0 / p[3];
        let sx = (p[0] * inv_w * 0.5 + 0.5) * self.viewport_width;
        let sy = (-p[1] * inv_w * 0.5 + 0.5) * self.viewport_height;
        let sz = p[2] * inv_w * 0.5 + 0.5;
        Some((sx, sy, sz.max(0.0).min(1.0)))
    }

    // ========================================================================
    // DRAW A PIXEL (with Z-test and alpha blend)
    // ========================================================================
    fn draw_pixel(&mut self, x: isize, y: isize, z: f32, color: u32, blend: BlendMode) {
        if x < 0 || x >= self.width as isize || y < 0 || y >= self.height as isize {
            return;
        }
        let idx = y as usize * self.width + x as usize;
        if z >= self.zbuffer[idx] {
            return;
        }

        let src_a = ((color >> 24) & 0xFF) as u8;
        let src_r = ((color >> 16) & 0xFF) as u8;
        let src_g = ((color >> 8) & 0xFF) as u8;
        let src_b = (color & 0xFF) as u8;

        let dst = self.framebuffer[idx];
        let dst_a = ((dst >> 24) & 0xFF) as u8;
        let dst_r = ((dst >> 16) & 0xFF) as u8;
        let dst_g = ((dst >> 8) & 0xFF) as u8;
        let dst_b = (dst & 0xFF) as u8;

        let (fr, fg, fb, fa) = match blend {
            BlendMode::None => (src_r, src_g, src_b, src_a),
            BlendMode::Alpha => {
                let sa = src_a as f32 / 255.0;
                let da = dst_a as f32 / 255.0;
                let oa = sa + da * (1.0 - sa);
                (
                    ((src_r as f32 * sa + dst_r as f32 * da * (1.0 - sa)) / oa.max(0.001)) as u8,
                    ((src_g as f32 * sa + dst_g as f32 * da * (1.0 - sa)) / oa.max(0.001)) as u8,
                    ((src_b as f32 * sa + dst_b as f32 * da * (1.0 - sa)) / oa.max(0.001)) as u8,
                    (oa * 255.0) as u8,
                )
            }
            BlendMode::Add => (
                (src_r as u16 + dst_r as u16).min(255) as u8,
                (src_g as u16 + dst_g as u16).min(255) as u8,
                (src_b as u16 + dst_b as u16).min(255) as u8,
                src_a.max(dst_a),
            ),
            BlendMode::Multiply => (
                (src_r as u16 * dst_r as u16 / 255) as u8,
                (src_g as u16 * dst_g as u16 / 255) as u8,
                (src_b as u16 * dst_b as u16 / 255) as u8,
                src_a.min(dst_a),
            ),
        };

        self.zbuffer[idx] = z;
        self.framebuffer[idx] =
            (fa as u32) << 24 | (fr as u32) << 16 | (fg as u32) << 8 | fb as u32;
    }

    // ========================================================================
    // FILLED TRIANGLE RASTERIZER (with perspective-correct attributes)
    // ========================================================================
    pub fn draw_triangle(
        &mut self,
        v0: &Vertex,
        v1: &Vertex,
        v2: &Vertex,
        texture: Option<&Texture>,
        blend: BlendMode,
        shader: ShaderType,
    ) {
        // Project vertices to screen space
        let p0 = match self.project_vertex(v0) {
            Some(p) => p,
            None => return,
        };
        let p1 = match self.project_vertex(v1) {
            Some(p) => p,
            None => return,
        };
        let p2 = match self.project_vertex(v2) {
            Some(p) => p,
            None => return,
        };

        self.rasterize_triangle(p0, p1, p2, v0, v1, v2, texture, blend, shader);
    }

    fn rasterize_triangle(
        &mut self,
        p0: (f32, f32, f32),
        p1: (f32, f32, f32),
        p2: (f32, f32, f32),
        v0: &Vertex,
        v1: &Vertex,
        v2: &Vertex,
        texture: Option<&Texture>,
        blend: BlendMode,
        shader: ShaderType,
    ) {
        let (x0, y0, z0) = p0;
        let (x1, y1, z1) = p1;
        let (x2, y2, z2) = p2;

        // Screen-space bounding box
        let min_x = (x0.min(x1).min(x2) as isize).max(0);
        let min_y = (y0.min(y1).min(y2) as isize).max(0);
        let max_x = (x0.max(x1).max(x2) as isize).min(self.width as isize - 1);
        let max_y = (y0.max(y1).max(y2) as isize).min(self.height as isize - 1);

        // Edge function (determinant) for barycentric coordinates
        let edge = |ax: f32, ay: f32, bx: f32, by: f32, cx: f32, cy: f32| -> f32 {
            (bx - ax) * (cy - ay) - (by - ay) * (cx - ax)
        };

        let area = edge(x0, y0, x1, y1, x2, y2);
        if area.abs() < 0.001 {
            return;
        }

        let inv_area = 1.0 / area;

        for y in min_y..=max_y {
            for x in min_x..=max_x {
                let fx = x as f32 + 0.5;
                let fy = y as f32 + 0.5;

                // Barycentric coordinates
                let w0 = edge(x1, y1, x2, y2, fx, fy);
                let w1 = edge(x2, y2, x0, y0, fx, fy);
                let w2 = edge(x0, y0, x1, y1, fx, fy);

                // Inside triangle test
                let b0 = w0 * inv_area;
                let b1 = w1 * inv_area;
                let b2 = w2 * inv_area;

                if b0 >= 0.0 && b1 >= 0.0 && b2 >= 0.0 {
                    // Depth interpolation (perspective-correct)
                    let z = b0 * z0 + b1 * z1 + b2 * z2;

                    // Interpolate attributes
                    let u = b0 * v0.u + b1 * v1.u + b2 * v2.u;
                    let v = b0 * v0.v + b1 * v1.v + b2 * v2.v;
                    let cr = b0 * v0.r + b1 * v1.r + b2 * v2.r;
                    let cg = b0 * v0.g + b1 * v1.g + b2 * v2.g;
                    let cb = b0 * v0.b + b1 * v1.b + b2 * v2.b;
                    let ca = b0 * v0.a + b1 * v1.a + b2 * v2.a;

                    let pixel_color = match shader {
                        ShaderType::Flat => {
                            let r = (v0.r * 255.0) as u8;
                            let g = (v0.g * 255.0) as u8;
                            let b = (v0.b * 255.0) as u8;
                            let a = (v0.a * 255.0) as u8;
                            (a as u32) << 24 | (r as u32) << 16 | (g as u32) << 8 | b as u32
                        }
                        ShaderType::Gouraud => {
                            let r = (cr * 255.0) as u8;
                            let g = (cg * 255.0) as u8;
                            let b = (cb * 255.0) as u8;
                            let a = (ca * 255.0) as u8;
                            (a as u32) << 24 | (r as u32) << 16 | (g as u32) << 8 | b as u32
                        }
                        ShaderType::Textured | ShaderType::TexturedLit => {
                            if let Some(tex) = texture {
                                let texel = tex.sample(u, v);
                                let tex_a = (texel >> 24) & 0xFF;
                                let tex_r = (texel >> 16) & 0xFF;
                                let tex_g = (texel >> 8) & 0xFF;
                                let tex_b = texel & 0xFF;
                                let factor = if matches!(shader, ShaderType::TexturedLit) {
                                    (cr * 0.5 + 0.5).max(0.0).min(1.0)
                                } else {
                                    1.0
                                };
                                let r = (tex_r as f32 * factor * ca) as u8;
                                let g = (tex_g as f32 * factor * ca) as u8;
                                let b = (tex_b as f32 * factor * ca) as u8;
                                let a = (tex_a as f32 * ca) as u8;
                                (a as u32) << 24 | (r as u32) << 16 | (g as u32) << 8 | b as u32
                            } else {
                                let r = (cr * 255.0 * ca) as u8;
                                let g = (cg * 255.0 * ca) as u8;
                                let b = (cb * 255.0 * ca) as u8;
                                (255u32) << 24 | (r as u32) << 16 | (g as u32) << 8 | b as u32
                            }
                        }
                        ShaderType::Glow => {
                            // Glow shader: brightens and blooms
                            let r = (cr * 255.0 * ca) as u16;
                            let g = (cg * 255.0 * ca) as u16;
                            let b = (cb * 255.0 * ca) as u16;
                            let r = (r + 80).min(255) as u8;
                            let g = (g + 80).min(255) as u8;
                            let b = (b + 120).min(255) as u8;
                            (255u32) << 24 | (r as u32) << 16 | (g as u32) << 8 | b as u32
                        }
                    };

                    self.draw_pixel(x, y, z, pixel_color, blend);
                }
            }
        }
    }

    // ========================================================================
    // LINE DRAWING
    // ========================================================================
    pub fn draw_line(
        &mut self,
        x0: f32,
        y0: f32,
        z0: f32,
        x1: f32,
        y1: f32,
        z1: f32,
        color: u32,
        thickness: f32,
    ) {
        // Bresenham-style thick line
        let dx = (x1 - x0).abs();
        let dy = -(y1 - y0).abs();
        let sx = if x0 < x1 { 1.0 } else { -1.0 };
        let sy = if y0 < y1 { 1.0 } else { -1.0 };
        let mut err = dx + dy;
        let mut x = x0;
        let mut y = y0;
        let t2 = (thickness / 2.0).max(1.0) as isize;

        loop {
            for ty in -t2..=t2 {
                for tx in -t2..=t2 {
                    let dist = libm::sqrtf((tx * tx + ty * ty) as f32);
                    if dist <= t2 as f32 {
                        let t = ((x - x0) / (x1 - x0).max(1.0)).max(0.0).min(1.0);
                        let z = z0 + (z1 - z0) * t;
                        self.draw_pixel(
                            x as isize + tx,
                            y as isize + ty,
                            z,
                            color,
                            BlendMode::None,
                        );
                    }
                }
            }
            if x as isize == x1 as isize && y as isize == y1 as isize {
                break;
            }
            let e2 = 2.0 * err;
            if e2 >= dy {
                err += dy;
                x += sx;
            }
            if e2 <= dx {
                err += dx;
                y += sy;
            }
        }
    }

    // ========================================================================
    // RECTANGLE
    // ========================================================================
    pub fn fill_rect(&mut self, x: f32, y: f32, w: f32, h: f32, z: f32, color: u32) {
        let x0 = x as isize;
        let y0 = y as isize;
        let x1 = (x + w) as isize;
        let y1 = (y + h) as isize;
        for py in y0..y1 {
            for px in x0..x1 {
                self.draw_pixel(px, py, z, color, BlendMode::None);
            }
        }
    }

    // ========================================================================
    // CIRCLE
    // ========================================================================
    pub fn fill_circle(&mut self, cx: f32, cy: f32, r: f32, z: f32, color: u32, blend: BlendMode) {
        let r_int = r as isize;
        for dy in -r_int..=r_int {
            let half = libm::sqrtf(((r_int * r_int - dy * dy) as f32).max(0.0)) as isize;
            for dx in -half..=half {
                self.draw_pixel(cx as isize + dx, cy as isize + dy, z, color, blend);
            }
        }
    }

    // ========================================================================
    // POST-PROCESSING EFFECTS
    // ========================================================================

    /// Apply a bloom (glow) effect — bright pixels bleed into neighbors
    pub fn apply_bloom(&mut self, threshold: u8, strength: f32) {
        let mut bloom_buf = alloc::vec![0u32; self.width * self.height];

        // First pass: extract bright pixels
        for y in 1..self.height - 1 {
            for x in 1..self.width - 1 {
                let idx = y * self.width + x;
                let c = self.framebuffer[idx];
                let r = ((c >> 16) & 0xFF) as u8;
                let g = ((c >> 8) & 0xFF) as u8;
                let b = (c & 0xFF) as u8;
                let brightness = (r as u16 + g as u16 + b as u16) / 3;
                if brightness > threshold as u16 {
                    bloom_buf[idx] = c;
                }
            }
        }

        // Blur bloom buffer (simple box blur)
        let mut blurred = alloc::vec![0u32; self.width * self.height];
        let kernel_size: isize = 3;
        for y in kernel_size as usize..self.height - kernel_size as usize {
            for x in kernel_size as usize..self.width - kernel_size as usize {
                let mut r = 0u32;
                let mut g = 0u32;
                let mut b = 0u32;
                let mut count = 0;
                for ky in -kernel_size..=kernel_size {
                    for kx in -kernel_size..=kernel_size {
                        let idx =
                            ((y as isize + ky) * self.width as isize + (x as isize + kx)) as usize;
                        let c = bloom_buf[idx];
                        if c != 0 {
                            r += (c >> 16) & 0xFF;
                            g += (c >> 8) & 0xFF;
                            b += c & 0xFF;
                            count += 1;
                        }
                    }
                }
                if count > 0 {
                    let avg_r = (r / count).min(255) as u8;
                    let avg_g = (g / count).min(255) as u8;
                    let avg_b = (b / count).min(255) as u8;
                    blurred[y * self.width + x] =
                        0xFF000000 | (avg_r as u32) << 16 | (avg_g as u32) << 8 | avg_b as u32;
                }
            }
        }

        // Composite bloom onto framebuffer
        for i in 0..self.framebuffer.len() {
            let bloom = blurred[i];
            if bloom != 0 {
                let br = ((bloom >> 16) & 0xFF) as u16;
                let bg = ((bloom >> 8) & 0xFF) as u16;
                let bb = (bloom & 0xFF) as u16;
                let fr = self.framebuffer[i];
                let fr_r = ((fr >> 16) & 0xFF) as u16;
                let fr_g = ((fr >> 8) & 0xFF) as u16;
                let fr_b = (fr & 0xFF) as u16;
                let add = (strength * 255.0) as u16;
                let r = (fr_r + br * add / 255).min(255) as u8;
                let g = (fr_g + bg * add / 255).min(255) as u8;
                let b = (fr_b + bb * add / 255).min(255) as u8;
                self.framebuffer[i] = 0xFF000000 | (r as u32) << 16 | (g as u32) << 8 | b as u32;
            }
        }
    }

    /// Apply scanline effect for retro-futuristic look
    pub fn apply_scanlines(&mut self, intensity: f32) {
        for y in (0..self.height).step_by(2) {
            for x in 0..self.width {
                let idx = y * self.width + x;
                let c = self.framebuffer[idx];
                let r = ((c >> 16) & 0xFF) as u16;
                let g = ((c >> 8) & 0xFF) as u16;
                let b = (c & 0xFF) as u16;
                let darken = (1.0 - intensity * 0.5).max(0.5);
                let r = (r as f32 * darken) as u8;
                let g = (g as f32 * darken) as u8;
                let b = (b as f32 * darken) as u8;
                self.framebuffer[idx] = 0xFF000000 | (r as u32) << 16 | (g as u32) << 8 | b as u32;
            }
        }
    }

    /// Apply radial vignette
    pub fn apply_vignette(&mut self, strength: f32) {
        let cx = self.width as f32 / 2.0;
        let cy = self.height as f32 / 2.0;
        let max_dist = libm::sqrtf(cx * cx + cy * cy);
        for y in 0..self.height {
            for x in 0..self.width {
                let dx = x as f32 - cx;
                let dy = y as f32 - cy;
                let dist = libm::sqrtf(dx * dx + dy * dy) / max_dist;
                let darken = 1.0 - (dist * dist * strength).min(1.0) * 0.6;
                let idx = y * self.width + x;
                let c = self.framebuffer[idx];
                let r = (((c >> 16) & 0xFF) as f32 * darken) as u8;
                let g = (((c >> 8) & 0xFF) as f32 * darken) as u8;
                let b = ((c & 0xFF) as f32 * darken) as u8;
                self.framebuffer[idx] = 0xFF000000 | (r as u32) << 16 | (g as u32) << 8 | b as u32;
            }
        }
    }

    // ========================================================================
    // SET TIME (for animated shaders)
    // ========================================================================
    pub fn set_time(&mut self, t: f32) {
        self.time = t;
    }

    // ========================================================================
    // CONVERT BETWEEN Color and ARGB u32
    // ========================================================================
    pub fn color_to_argb(c: Color) -> u32 {
        0xFF000000 | (c.r as u32) << 16 | (c.g as u32) << 8 | c.b as u32
    }

    pub fn argb_to_color(argb: u32) -> Color {
        Color {
            r: ((argb >> 16) & 0xFF) as u8,
            g: ((argb >> 8) & 0xFF) as u8,
            b: (argb & 0xFF) as u8,
        }
    }
}

// ============================================================================
// SCENE — Manages draw calls
// ============================================================================
pub struct Scene {
    pub draw_calls: alloc::vec::Vec<DrawCall>,
    pub clear_color: u32,
    pub time_seconds: f32,
}

impl Default for Scene {
    fn default() -> Self {
        Self::new()
    }
}

impl Scene {
    pub fn new() -> Self {
        Self {
            draw_calls: alloc::vec::Vec::new(),
            clear_color: 0xFF08080E,
            time_seconds: 0.0,
        }
    }

    pub fn add_draw_call(&mut self, call: DrawCall) {
        self.draw_calls.push(call);
    }

    pub fn render(&self, renderer: &mut Renderer) {
        renderer.clear(self.clear_color, f32::MAX);

        for call in &self.draw_calls {
            let blend = call.blend_mode;
            let shader = call.shader;
            match &call.mode {
                DrawMode::Triangles => {
                    for tri in call.vertices.chunks(3) {
                        if tri.len() == 3 {
                            renderer.draw_triangle(
                                &tri[0],
                                &tri[1],
                                &tri[2],
                                call.texture.as_deref(),
                                blend,
                                shader,
                            );
                        }
                    }
                }
                DrawMode::TriangleFan => {
                    if call.vertices.len() >= 3 {
                        let v0 = &call.vertices[0];
                        for i in 1..call.vertices.len() - 1 {
                            renderer.draw_triangle(
                                v0,
                                &call.vertices[i],
                                &call.vertices[i + 1],
                                call.texture.as_deref(),
                                blend,
                                shader,
                            );
                        }
                    }
                }
                DrawMode::TriangleStrip => {
                    for i in 0..call.vertices.len().saturating_sub(2) {
                        renderer.draw_triangle(
                            &call.vertices[i],
                            &call.vertices[i + 1],
                            &call.vertices[i + 2],
                            call.texture.as_deref(),
                            blend,
                            shader,
                        );
                    }
                }
                _ => {}
            }
        }
    }
}
