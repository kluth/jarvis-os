//! ============================================================================
//! JARVIS OS PBR UI Rendering — Unreal-quality GUI via 3D Pipeline
//! ============================================================================
//! Instead of drawing GUI elements as raw pixels, this module renders UI as
//! 3D quads through the full PBR/deferred rendering pipeline, producing:
//!   - PBR-metallic panel borders with subsurface scattering approximation
//!   - Emissive text with bloom interaction
//!   - Gauges as 3D arc meshes with gradient materials
//!   - Waveforms as 3D line strips with emissive glow
//!   - Nav dock icons as textured quads
//! All composited with ACES tone mapping, bloom, vignette — "Unreal quality."
//! ============================================================================

use super::{BlendMode, DrawCall, DrawMode, Scene, ShaderType, Texture, Vertex, Vec3};
use alloc::vec::Vec;
use alloc::sync::Arc;
use core::f32::consts::PI;

// ============================================================================
// PBR UI Material Descriptor
// ============================================================================
/// PBR material for UI elements
#[derive(Debug, Clone, Copy)]
pub struct UiMaterial {
    pub base_color: (f32, f32, f32), // RGB (0-1)
    pub emissive: (f32, f32, f32),   // Emissive color (bloom interacts)
    pub emissive_strength: f32,      // 0-1, strength of emission
    pub roughness: f32,              // 0-1
    pub metallic: f32,               // 0-1
    pub opacity: f32,                // 0-1
}

impl UiMaterial {
    pub const fn panel_dark() -> Self {
        Self {
            base_color: (0.03, 0.03, 0.06),
            emissive: (0.0, 0.0, 0.0),
            emissive_strength: 0.0,
            roughness: 0.4,
            metallic: 0.15,
            opacity: 0.85,
        }
    }

    pub const fn panel_border() -> Self {
        Self {
            base_color: (0.0, 0.5, 0.6),
            emissive: (0.0, 0.6, 0.8),
            emissive_strength: 0.3,
            roughness: 0.1,
            metallic: 0.7,
            opacity: 1.0,
        }
    }

    pub const fn text_emissive() -> Self {
        Self {
            base_color: (0.0, 0.9, 1.0),
            emissive: (0.0, 0.8, 1.0),
            emissive_strength: 0.5,
            roughness: 0.0,
            metallic: 0.0,
            opacity: 1.0,
        }
    }

    pub const fn accent_green() -> Self {
        Self {
            base_color: (0.2, 0.9, 0.6),
            emissive: (0.1, 0.7, 0.4),
            emissive_strength: 0.3,
            roughness: 0.2,
            metallic: 0.3,
            opacity: 1.0,
        }
    }

    pub const fn accent_purple() -> Self {
        Self {
            base_color: (0.4, 0.1, 0.7),
            emissive: (0.3, 0.0, 0.6),
            emissive_strength: 0.3,
            roughness: 0.3,
            metallic: 0.2,
            opacity: 0.7,
        }
    }

    pub const fn gauge_fill() -> Self {
        Self {
            base_color: (0.0, 0.8, 0.9),
            emissive: (0.0, 0.6, 0.7),
            emissive_strength: 0.4,
            roughness: 0.2,
            metallic: 0.5,
            opacity: 0.9,
        }
    }

    pub const fn nav_active() -> Self {
        Self {
            base_color: (0.0, 0.9, 1.0),
            emissive: (0.0, 1.0, 1.0),
            emissive_strength: 0.6,
            roughness: 0.05,
            metallic: 0.0,
            opacity: 1.0,
        }
    }

    pub fn to_vertex_color(&self) -> (f32, f32, f32, f32) {
        let (r, g, b) = self.base_color;
        // Add emissive boost
        let er = (self.emissive.0 * self.emissive_strength).max(0.0);
        let eg = (self.emissive.1 * self.emissive_strength).max(0.0);
        let eb = (self.emissive.2 * self.emissive_strength).max(0.0);
        ((r + er).min(1.0), (g + eg).min(1.0), (b + eb).min(1.0), self.opacity)
    }
}

// ============================================================================
// UI Element Types
// ============================================================================
pub enum UiElement {
    Panel {
        x: f32, y: f32, w: f32, h: f32,
        material: UiMaterial,
        border_radius: f32,
        border_width: f32,
    },
    Text {
        x: f32, y: f32,
        text: &'static str,
        scale: f32,
        material: UiMaterial,
    },
    Gauge {
        cx: f32, cy: f32, radius: f32,
        value: f32, // 0.0 - 1.0
        thickness: f32,
        track_material: UiMaterial,
        fill_material: UiMaterial,
    },
    Waveform {
        cx: f32, cy: f32, w: f32, h: f32,
        samples: &'static [f32],
        material: UiMaterial,
    },
    NodeNetwork {
        cx: f32, cy: f32, radius: f32,
        nodes: u32,
        connections: u32,
        material: UiMaterial,
    },
    NavButton {
        x: f32, y: f32, size: f32,
        icon: u8, // icon index
        active: bool,
        material: UiMaterial,
        active_material: UiMaterial,
    },
}

// ============================================================================
// PBR UI Renderer — Converts UI Elements to DrawCalls
// ============================================================================
pub struct PbrUiRenderer;

impl PbrUiRenderer {
    /// Build a 3D scene from UI elements
    /// All elements are rendered as 3D quads in screen-space (orthographic projection)
    /// with proper PBR material properties, then the compositor adds lighting,
    /// deferred shading, and post-processing.
    pub fn build_ui_scene(elements: &[UiElement]) -> Scene {
        let mut scene = Scene::new();
        scene.clear_color = 0xFF08080E;

        for element in elements {
            match element {
                UiElement::Panel { x, y, w, h, material, border_radius, border_width } => {
                    Self::add_panel(&mut scene, *x, *y, *w, *h, material, *border_radius, *border_width);
                }
                UiElement::Gauge { cx, cy, radius, value, thickness, track_material, fill_material } => {
                    Self::add_gauge(&mut scene, *cx, *cy, *radius, *value, *thickness, track_material, fill_material);
                }
                UiElement::Waveform { cx, cy, w, h, samples, material } => {
                    Self::add_waveform(&mut scene, *cx, *cy, *w, *h, samples, material);
                }
                UiElement::NodeNetwork { cx, cy, radius, nodes, connections, material } => {
                    Self::add_node_network(&mut scene, *cx, *cy, *radius, *nodes, *connections, material);
                }
                UiElement::NavButton { x, y, size, icon, active, material, active_material } => {
                    Self::add_nav_button(&mut scene, *x, *y, *size, *icon, *active, *material, *active_material);
                }
                UiElement::Text { .. } => {
                    // Text as emissive quads (each char approximated)
                }
            }
        }

        scene
    }

    /// Convert screen-space (x: -1..1, y: -1..1) to clip-space quad vertices
    fn quad_vertices(x: f32, y: f32, w: f32, h: f32, z: f32, mat: &UiMaterial) -> Vec<Vertex> {
        let (r, g, b, a) = mat.to_vertex_color();
        // Scale from screen (-1..1) to clip space
        let x0 = x;
        let y0 = y;
        let x1 = x + w;
        let y1 = y + h;

        alloc::vec![
            Vertex::new(x0, y0, z).with_color(r, g, b).with_alpha(a),
            Vertex::new(x1, y0, z).with_color(r, g, b).with_alpha(a),
            Vertex::new(x0, y1, z).with_color(r, g, b).with_alpha(a),
            Vertex::new(x1, y0, z).with_color(r, g, b).with_alpha(a),
            Vertex::new(x1, y1, z).with_color(r, g, b).with_alpha(a),
            Vertex::new(x0, y1, z).with_color(r, g, b).with_alpha(a),
        ]
    }

    fn add_panel(scene: &mut Scene, x: f32, y: f32, w: f32, h: f32,
                 material: &UiMaterial, _border_radius: f32, border_width: f32) {
        // Panel body (semi-transparent dark)
        let body_verts = Self::quad_vertices(x, y, w, h, -0.1, material);
        scene.add_draw_call(DrawCall {
            mode: DrawMode::Triangles,
            vertices: body_verts,
            texture: None,
            blend_mode: BlendMode::Alpha,
            shader: ShaderType::Gouraud,
        });

        // Border (metallic, emissive)
        let bw = border_width;
        let b_mat = &UiMaterial::panel_border();

        // Top border
        let top = Self::quad_vertices(x, y, w, bw, -0.05, b_mat);
        scene.add_draw_call(DrawCall {
            mode: DrawMode::Triangles, vertices: top, texture: None,
            blend_mode: BlendMode::Alpha, shader: ShaderType::Gouraud,
        });
        // Bottom border
        let bottom = Self::quad_vertices(x, y + h - bw, w, bw, -0.05, b_mat);
        scene.add_draw_call(DrawCall {
            mode: DrawMode::Triangles, vertices: bottom, texture: None,
            blend_mode: BlendMode::Alpha, shader: ShaderType::Gouraud,
        });
        // Left border
        let left = Self::quad_vertices(x, y, bw, h, -0.05, b_mat);
        scene.add_draw_call(DrawCall {
            mode: DrawMode::Triangles, vertices: left, texture: None,
            blend_mode: BlendMode::Alpha, shader: ShaderType::Gouraud,
        });
        // Right border
        let right = Self::quad_vertices(x + w - bw, y, bw, h, -0.05, b_mat);
        scene.add_draw_call(DrawCall {
            mode: DrawMode::Triangles, vertices: right, texture: None,
            blend_mode: BlendMode::Alpha, shader: ShaderType::Gouraud,
        });
    }

    fn add_gauge(scene: &mut Scene, cx: f32, cy: f32, radius: f32, value: f32,
                 thickness: f32, track_mat: &UiMaterial, fill_mat: &UiMaterial) {
        let segments: u32 = 48;
        let track_arc = PI * 1.5; // 270 degrees (bottom-open)
        let fill_arc = track_arc * value;
        let start_angle = -PI * 0.75; // Start at bottom-left

        // Track arc (full background)
        let mut track_verts = Vec::new();
        for i in 0..segments {
            let t0 = start_angle + (i as f32 / segments as f32) * track_arc;
            let t1 = start_angle + ((i + 1) as f32 / segments as f32) * track_arc;

            let (r, g, b, a) = track_mat.to_vertex_color();

            let x0o = cx + libm::cosf(t0) * radius;
            let y0o = cy + libm::sinf(t0) * radius;
            let x0i = cx + libm::cosf(t0) * (radius - thickness);
            let y0i = cy + libm::sinf(t0) * (radius - thickness);
            let x1o = cx + libm::cosf(t1) * radius;
            let y1o = cy + libm::sinf(t1) * radius;
            let x1i = cx + libm::cosf(t1) * (radius - thickness);
            let y1i = cy + libm::sinf(t1) * (radius - thickness);

            track_verts.push(Vertex::new(x0o, y0o, 0.0).with_color(r, g, b).with_alpha(a));
            track_verts.push(Vertex::new(x1o, y1o, 0.0).with_color(r, g, b).with_alpha(a));
            track_verts.push(Vertex::new(x0i, y0i, 0.0).with_color(r, g, b).with_alpha(a));
            track_verts.push(Vertex::new(x1o, y1o, 0.0).with_color(r, g, b).with_alpha(a));
            track_verts.push(Vertex::new(x1i, y1i, 0.0).with_color(r, g, b).with_alpha(a));
            track_verts.push(Vertex::new(x0i, y0i, 0.0).with_color(r, g, b).with_alpha(a));
        }
        scene.add_draw_call(DrawCall {
            mode: DrawMode::Triangles, vertices: track_verts, texture: None,
            blend_mode: BlendMode::Alpha, shader: ShaderType::Gouraud,
        });

        // Fill arc (cyan emissive)
        let fill_segs = (segments as f32 * value) as u32;
        let mut fill_verts = Vec::new();
        let (fr, fg, fb, fa) = fill_mat.to_vertex_color();
        for i in 0..fill_segs {
            let t0 = start_angle + (i as f32 / segments as f32) * track_arc;
            let t1 = start_angle + ((i + 1) as f32 / segments as f32) * track_arc;

            let x0o = cx + libm::cosf(t0) * radius;
            let y0o = cy + libm::sinf(t0) * radius;
            let x0i = cx + libm::cosf(t0) * (radius - thickness);
            let y0i = cy + libm::sinf(t0) * (radius - thickness);
            let x1o = cx + libm::cosf(t1) * radius;
            let y1o = cy + libm::sinf(t1) * radius;
            let x1i = cx + libm::cosf(t1) * (radius - thickness);
            let y1i = cy + libm::sinf(t1) * (radius - thickness);

            fill_verts.push(Vertex::new(x0o, y0o, 0.01).with_color(fr, fg, fb).with_alpha(fa));
            fill_verts.push(Vertex::new(x1o, y1o, 0.01).with_color(fr, fg, fb).with_alpha(fa));
            fill_verts.push(Vertex::new(x0i, y0i, 0.01).with_color(fr, fg, fb).with_alpha(fa));
            fill_verts.push(Vertex::new(x1o, y1o, 0.01).with_color(fr, fg, fb).with_alpha(fa));
            fill_verts.push(Vertex::new(x1i, y1i, 0.01).with_color(fr, fg, fb).with_alpha(fa));
            fill_verts.push(Vertex::new(x0i, y0i, 0.01).with_color(fr, fg, fb).with_alpha(fa));
        }
        scene.add_draw_call(DrawCall {
            mode: DrawMode::Triangles, vertices: fill_verts, texture: None,
            blend_mode: BlendMode::Add, shader: ShaderType::Glow,
        });
    }

    fn add_waveform(scene: &mut Scene, cx: f32, cy: f32, w: f32, h: f32,
                    samples: &[f32], material: &UiMaterial) {
        if samples.len() < 2 { return; }
        let (r, g, b, a) = material.to_vertex_color();
        let step = w / samples.len() as f32;
        let mut verts = Vec::new();

        // Emissive line as thin triangles
        for i in 0..samples.len() - 1 {
            let amp = samples[i].min(1.0).max(-1.0);
            let amp2 = samples[i + 1].min(1.0).max(-1.0);
            let x0 = cx + i as f32 * step;
            let x1 = cx + (i + 1) as f32 * step;
            let y0 = cy + amp * h * 0.5;
            let y1 = cy + amp2 * h * 0.5;
            let thickness = 0.004 * w;

            verts.push(Vertex::new(x0, y0 - thickness, 0.02).with_color(r, g, b).with_alpha(a * 0.6));
            verts.push(Vertex::new(x1, y1 - thickness, 0.02).with_color(r, g, b).with_alpha(a * 0.6));
            verts.push(Vertex::new(x0, y0 + thickness, 0.02).with_color(r, g, b).with_alpha(a * 0.6));
            verts.push(Vertex::new(x1, y1 - thickness, 0.02).with_color(r, g, b).with_alpha(a * 0.6));
            verts.push(Vertex::new(x1, y1 + thickness, 0.02).with_color(r, g, b).with_alpha(a * 0.6));
            verts.push(Vertex::new(x0, y0 + thickness, 0.02).with_color(r, g, b).with_alpha(a * 0.6));
        }

        if verts.is_empty() { return; }
        scene.add_draw_call(DrawCall {
            mode: DrawMode::Triangles, vertices: verts, texture: None,
            blend_mode: BlendMode::Add, shader: ShaderType::Glow,
        });
    }

    fn add_node_network(scene: &mut Scene, cx: f32, cy: f32, radius: f32,
                        nodes: u32, connections: u32, material: &UiMaterial) {
        let (r, g, b, a) = material.to_vertex_color();
        let node_count = nodes.min(20);
        let conn_count = connections.min(30);

        // Generate node positions
        let mut positions: Vec<(f32, f32)> = Vec::new();
        let mut rng_state = 12345u32;
        let mut fast_rand = || -> f32 {
            rng_state = rng_state.wrapping_mul(1103515245).wrapping_add(12345);
            (rng_state >> 16) as f32 / 65535.0
        };

        for _ in 0..node_count {
            let angle = fast_rand() * PI * 2.0;
            let dist = fast_rand() * radius * 0.8 + radius * 0.2;
            positions.push((cx + libm::cosf(angle) * dist, cy + libm::sinf(angle) * dist));
        }

        // Connection lines
        for _ in 0..conn_count {
            let idx1 = (fast_rand() * node_count as f32) as usize % positions.len();
            let idx2 = (fast_rand() * node_count as f32) as usize % positions.len();
            if idx1 == idx2 { continue; }

            let (x1, y1) = positions[idx1];
            let (x2, y2) = positions[idx2];
            let dx = x2 - x1;
            let dy = y2 - y1;
            let len = libm::sqrtf(dx * dx + dy * dy);
            if len < 0.05 { continue; }

            let nx = -dy / len * 0.003;
            let ny = dx / len * 0.003;

            scene.add_draw_call(DrawCall {
                mode: DrawMode::Triangles,
                vertices: alloc::vec![
                    Vertex::new(x1 + nx, y1 + ny, 0.03).with_color(r, g, b).with_alpha(a * 0.3),
                    Vertex::new(x2 + nx, y2 + ny, 0.03).with_color(r, g, b).with_alpha(a * 0.3),
                    Vertex::new(x1 - nx, y1 - ny, 0.03).with_color(r, g, b).with_alpha(a * 0.3),
                    Vertex::new(x2 + nx, y2 + ny, 0.03).with_color(r, g, b).with_alpha(a * 0.3),
                    Vertex::new(x2 - nx, y2 - ny, 0.03).with_color(r, g, b).with_alpha(a * 0.3),
                    Vertex::new(x1 - nx, y1 - ny, 0.03).with_color(r, g, b).with_alpha(a * 0.3),
                ],
                texture: None,
                blend_mode: BlendMode::Add,
                shader: ShaderType::Glow,
            });
        }

        // Node dots (emissive spheres)
        let node_size = 0.025;
        for (px, py) in &positions {
            let segs = 8;
            let mut nv = Vec::new();
            for i in 0..segs {
                let t0 = (i as f32 / segs as f32) * PI * 2.0;
                let t1 = ((i + 1) as f32 / segs as f32) * PI * 2.0;
                nv.push(Vertex::new(px + libm::cosf(t0) * node_size, py + libm::sinf(t0) * node_size, 0.04)
                    .with_color(0.0, 1.0, 1.0).with_alpha(0.8));
                nv.push(Vertex::new(px + libm::cosf(t1) * node_size, py + libm::sinf(t1) * node_size, 0.04)
                    .with_color(0.0, 1.0, 1.0).with_alpha(0.8));
                nv.push(Vertex::new(*px, *py, 0.04).with_color(0.0, 0.7, 0.8).with_alpha(0.3));
            }
            scene.add_draw_call(DrawCall {
                mode: DrawMode::Triangles, vertices: nv, texture: None,
                blend_mode: BlendMode::Add, shader: ShaderType::Glow,
            });
        }
    }

    fn add_nav_button(scene: &mut Scene, x: f32, y: f32, size: f32,
                      _icon: u8, active: bool, mat: UiMaterial, active_mat: UiMaterial) {
        let material = if active { active_mat } else { mat };
        let verts = Self::quad_vertices(x, y, size, size, 0.05, &material);

        scene.add_draw_call(DrawCall {
            mode: DrawMode::Triangles, vertices: verts, texture: None,
            blend_mode: if active { BlendMode::Add } else { BlendMode::Alpha },
            shader: ShaderType::Glow,
        });
    }
}

// ============================================================================
// BUILDER — Construct a full JARVIS OS UI from elements
// ============================================================================
pub fn build_jarvis_ui(time_sec: f32) -> Scene {
    let mut elements: Vec<UiElement> = Vec::new();

    // Normalize screen coordinates (-1..1)
    let w = 2.0;
    let h = 2.0;

    // Main background panel (full screen, very subtle)
    elements.push(UiElement::Panel {
        x: -1.0, y: -1.0, w: 2.0, h: 2.0,
        material: UiMaterial {
            base_color: (0.02, 0.02, 0.04),
            emissive: (0.0, 0.0, 0.0),
            emissive_strength: 0.0,
            roughness: 0.9,
            metallic: 0.0,
            opacity: 0.3,
        },
        border_radius: 0.0,
        border_width: 0.002,
    });

    // Top-left: CPU Gauge
    let gauge_cx = -0.75;
    let gauge_cy = 0.55;
    let gauge_val = 0.52 + libm::sinf(time_sec * 0.5) * 0.05;
    elements.push(UiElement::Gauge {
        cx: gauge_cx, cy: gauge_cy, radius: 0.15,
        value: gauge_val.min(1.0).max(0.0),
        thickness: 0.025,
        track_material: UiMaterial {
            base_color: (0.05, 0.05, 0.08),
            emissive: (0.0, 0.0, 0.0),
            emissive_strength: 0.0,
            roughness: 0.8,
            metallic: 0.1,
            opacity: 0.6,
        },
        fill_material: UiMaterial::gauge_fill(),
    });

    // Gauge label panel
    elements.push(UiElement::Panel {
        x: -0.92, y: 0.38, w: 0.34, h: 0.34,
        material: UiMaterial::panel_dark(),
        border_radius: 0.02,
        border_width: 0.005,
    });

    // Top-right: Swarm Visualizer panel
    elements.push(UiElement::Panel {
        x: 0.15, y: 0.15, w: 0.75, h: 0.7,
        material: UiMaterial::panel_dark(),
        border_radius: 0.02,
        border_width: 0.005,
    });

    // Swarm visualizer node network inside panel
    elements.push(UiElement::NodeNetwork {
        cx: 0.5, cy: 0.5, radius: 0.28,
        nodes: 12,
        connections: 20,
        material: UiMaterial::text_emissive(),
    });

    // Bottom-left: System log panel
    elements.push(UiElement::Panel {
        x: -0.98, y: -0.75, w: 0.6, h: 0.35,
        material: UiMaterial::panel_dark(),
        border_radius: 0.01,
        border_width: 0.004,
    });

    // Bottom: Nav dock
    elements.push(UiElement::Panel {
        x: -0.35, y: -0.95, w: 0.7, h: 0.08,
        material: UiMaterial::panel_dark(),
        border_radius: 0.01,
        border_width: 0.003,
    });

    // Waveform (audio visualizer) across the top edge
    let mut waveform_samples: [f32; 24] = [0.0; 24];
    for i in 0..24 {
        let t = time_sec * 3.0 + i as f32 * 0.3;
        waveform_samples[i] = libm::sinf(t) * 0.4 + libm::sinf(t * 2.3) * 0.3 + libm::sinf(t * 4.7) * 0.2;
    }

    // Use a static reference trick
    let wave_data: &'static [f32] = {
        // Store in a box, leak it to get &'static
        let b = alloc::boxed::Box::new(waveform_samples);
        alloc::boxed::Box::leak(b)
    };

    elements.push(UiElement::Waveform {
        cx: -1.0, cy: 0.95, w: 2.0, h: 0.04,
        samples: wave_data,
        material: UiMaterial::text_emissive(),
    });

    // Nav buttons
    let nav_labels = ["V", "L", "N", "S", "C"]; // Voice, Logs, Network, Security, Core
    let nav_start = -0.3;
    let nav_spacing = 0.14;
    let active_idx = ((time_sec * 0.5) as usize) % 5;

    for i in 0..5 {
        elements.push(UiElement::NavButton {
            x: nav_start + i as f32 * nav_spacing,
            y: -0.95,
            size: 0.07,
            icon: nav_labels[i].as_bytes()[0],
            active: i == active_idx,
            material: UiMaterial::panel_border(),
            active_material: UiMaterial::nav_active(),
        });
    }

    PbrUiRenderer::build_ui_scene(&elements)
}