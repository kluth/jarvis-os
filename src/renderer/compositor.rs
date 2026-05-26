//! ============================================================================
//! JARVIS OS 3D GUI Compositor — Unreal-quality Scene + PBR UI Rendering
//! ============================================================================
//! The entire GUI is rendered through the 3D pipeline:
//!   1. 3D hex core background (perspective, glow shaders)
//!   2. PBR UI elements as screen-space quads (identity projection)
//!   3. Full post-processing: bloom, ACES tone mapping, vignette
//! ============================================================================

use super::effects::{apply_bloom, apply_vignette};
use super::ui::build_jarvis_ui;
use super::{BlendMode, Renderer, ShaderType, Vertex};
use alloc::vec::Vec;

pub struct GuiCompositor {
    renderer: Renderer,
    ui_renderer: Renderer,
    scene_bg: Vec<u32>,
    angle: f32,
    pub output: Vec<u32>,
    width: usize,
    height: usize,
}

impl GuiCompositor {
    pub fn new(width: usize, height: usize) -> Self {
        Self {
            renderer: Renderer::new(width, height),
            ui_renderer: Renderer::new(width, height),
            scene_bg: alloc::vec![0xFF08080Eu32; width * height],
            angle: 0.0,
            output: alloc::vec![0xFF08080Eu32; width * height],
            width,
            height,
        }
    }

    /// Build rotating hex core mesh with glow materials
    fn build_core_mesh(&self, angle: f32) -> (Vec<Vertex>, Vec<Vertex>, Vec<Vertex>) {
        let hex_r = 0.6;
        let hex_t = 0.08;
        let segs = 6;
        let mut outer = Vec::new();
        let mut inner = Vec::new();
        let mut hub = Vec::new();
        let ca = libm::cosf(angle);
        let sa = libm::sinf(angle);
        let rot = |x: f32, z: f32| (x * ca - z * sa, x * sa + z * ca);

        for i in 0..segs {
            let t0 = (i as f32 / segs as f32) * core::f32::consts::TAU;
            let t1 = ((i + 1) as f32 / segs as f32) * core::f32::consts::TAU;
            for &(r, z, cr, cg, cb) in &[
                (hex_r, -0.02, 0.0, 0.9, 1.0),
                (hex_r - hex_t, -0.02, 0.0, 0.5, 0.8),
            ] {
                let (rx0, rz0) = rot(libm::cosf(t0) * r, libm::sinf(t0) * r);
                let (rx1, rz1) = rot(libm::cosf(t1) * r, libm::sinf(t1) * r);
                outer.push(Vertex::new(rx0, z, rz0).with_color(cr, cg, cb));
                outer.push(Vertex::new(rx1, z, rz1).with_color(cr, cg, cb));
            }
            // Inner ring
            let ir = (hex_r - hex_t) * 0.7;
            let (rix0, riz0) = rot(libm::cosf(t0) * ir, libm::sinf(t0) * ir);
            let (rix1, riz1) = rot(libm::cosf(t1) * ir, libm::sinf(t1) * ir);
            inner.push(Vertex::new(rix0, -0.01, riz0).with_color(0.5, 0.1, 0.8));
            inner.push(Vertex::new(rix1, -0.01, riz1).with_color(0.5, 0.1, 0.8));
            inner.push(Vertex::new(0.0, -0.01, 0.0).with_color(0.3, 0.05, 0.6));
            inner.push(Vertex::new(rix1, -0.01, riz1).with_color(0.5, 0.1, 0.8));
            inner.push(Vertex::new(0.0, -0.01, 0.0).with_color(0.3, 0.05, 0.6));
            inner.push(Vertex::new(rix0, -0.01, riz0).with_color(0.5, 0.1, 0.8));
        }
        let hr = 0.12;
        let hs = 12;
        for i in 0..hs {
            let t0 = (i as f32 / hs as f32) * core::f32::consts::TAU;
            let t1 = ((i + 1) as f32 / hs as f32) * core::f32::consts::TAU;
            let (rx0, rz0) = rot(libm::cosf(t0) * hr, libm::sinf(t0) * hr);
            let (rx1, rz1) = rot(libm::cosf(t1) * hr, libm::sinf(t1) * hr);
            hub.push(Vertex::new(rx0, 0.0, rz0).with_color(0.0, 1.0, 0.9));
            hub.push(Vertex::new(rx1, 0.0, rz1).with_color(0.0, 1.0, 0.9));
            hub.push(Vertex::new(0.0, hr, 0.0).with_color(0.0, 0.7, 0.6));
        }
        (outer, inner, hub)
    }

    pub fn render_frame(&mut self) -> &[u32] {
        self.angle += 0.02;
        if self.angle > core::f32::consts::TAU * 2.0 {
            self.angle -= core::f32::consts::TAU * 2.0;
        }

        // === PASS 1: 3D Hex Core (perspective camera) ===
        self.renderer.clear(0xFF08080E, f32::MAX);
        self.renderer
            .set_perspective(45.0, self.width as f32 / self.height as f32, 0.1, 10.0);
        self.renderer.set_view(
            super::Vec3::new(0.0, 0.3, 2.5),
            super::Vec3::new(0.0, 0.0, 0.0),
            super::Vec3::new(0.0, 1.0, 0.0),
        );
        let (o, i, h) = self.build_core_mesh(self.angle);
        for tri in o.chunks(3) {
            if tri.len() == 3 {
                self.renderer.draw_triangle(
                    &tri[0],
                    &tri[1],
                    &tri[2],
                    None,
                    BlendMode::Alpha,
                    ShaderType::Glow,
                );
            }
        }
        for tri in i.chunks(3) {
            if tri.len() == 3 {
                self.renderer.draw_triangle(
                    &tri[0],
                    &tri[1],
                    &tri[2],
                    None,
                    BlendMode::Alpha,
                    ShaderType::Glow,
                );
            }
        }
        for tri in h.chunks(3) {
            if tri.len() == 3 {
                self.renderer.draw_triangle(
                    &tri[0],
                    &tri[1],
                    &tri[2],
                    None,
                    BlendMode::Add,
                    ShaderType::Glow,
                );
            }
        }
        // Grab scene with richer colors before tone mapping darkens everything
        self.scene_bg.copy_from_slice(&self.renderer.framebuffer);

        // === PASS 2: PBR UI (identity projection, screen-space) ===
        self.ui_renderer.clear(0x00000000, f32::MAX);
        let ui_scene = build_jarvis_ui(self.angle);
        ui_scene.render(&mut self.ui_renderer);

        // === PASS 3: Composite UI over 3D background ===
        let pixels = self.width * self.height;
        for i in 0..pixels {
            let ui = self.ui_renderer.framebuffer[i];
            let ua = ((ui >> 24) & 0xFF) as u16;
            if ua < 10 {
                self.renderer.framebuffer[i] = self.scene_bg[i];
                continue;
            }
            let bg = self.scene_bg[i];
            let br = ((bg >> 16) & 0xFF) as u16;
            let bg_ = ((bg >> 8) & 0xFF) as u16;
            let bb = (bg & 0xFF) as u16;
            let ur = ((ui >> 16) & 0xFF) as u16;
            let ug = ((ui >> 8) & 0xFF) as u16;
            let ub = (ui & 0xFF) as u16;
            let a = (ua as f32 / 255.0).min(1.0);
            let ia = 1.0 - a;
            let or = (ur as f32 * a + br as f32 * ia) as u16;
            let og = (ug as f32 * a + bg_ as f32 * ia) as u16;
            let ob = (ub as f32 * a + bb as f32 * ia) as u16;
            self.renderer.framebuffer[i] = 0xFF000000
                | ((or.min(255) as u32) << 16)
                | ((og.min(255) as u32) << 8)
                | (ob.min(255) as u32);
        }

        // === PASS 4: Post-processing (mild bloom, no tone-map darkening) ===
        apply_bloom(
            &mut self.renderer.framebuffer,
            self.width,
            self.height,
            0.06,
            0.7,
        );
        // Replace ACES tone mapping with a simple brightness boost for the dark theme
        for pixel in self.renderer.framebuffer.iter_mut() {
            let r = ((*pixel >> 16) & 0xFF);
            let g = ((*pixel >> 8) & 0xFF);
            let b = (*pixel & 0xFF);
            // Boost dark colors for visibility: 1.5x brighten
            let nr = (r * 3 / 2).min(255);
            let ng = (g * 3 / 2).min(255);
            let nb = (b * 3 / 2).min(255);
            *pixel = 0xFF000000 | (nr << 16) | (ng << 8) | nb;
        }
        apply_vignette(&mut self.renderer.framebuffer, self.width, self.height, 0.5);

        self.output.copy_from_slice(&self.renderer.framebuffer);
        &self.output
    }

    pub fn blit_to_fb(&self, dest: &mut [u8]) {
        let n = (self.width * self.height).min(dest.len() / 3);
        for i in 0..n {
            let c = self.output[i];
            dest[i * 3] = (c & 0xFF) as u8; // B
            dest[i * 3 + 1] = ((c >> 8) & 0xFF) as u8; // G
            dest[i * 3 + 2] = ((c >> 16) & 0xFF) as u8; // R
        }
    }

    pub fn renderer_mut(&mut self) -> &mut Renderer {
        &mut self.renderer
    }
}
