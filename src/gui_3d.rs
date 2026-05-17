use crate::vga_buffer::{Color, FramebufferWriter};
use alloc::vec::Vec;

#[derive(Debug, Clone, Copy)]
pub struct Point3D {
    pub x: f32,
    pub y: f32,
    pub z: f32,
}

#[derive(Debug, Clone, Copy)]
pub struct Point2D {
    pub x: isize,
    pub y: isize,
}

pub struct Mesh3D {
    pub vertices: Vec<Point3D>,
    pub edges: Vec<(usize, usize)>,
    pub color: Color,
}

impl Mesh3D {
    pub fn new_cube(size: f32, color: Color) -> Self {
        let h = size / 2.0;
        let vertices = alloc::vec![
            Point3D {
                x: -h,
                y: -h,
                z: -h
            }, // 0
            Point3D { x: h, y: -h, z: -h }, // 1
            Point3D { x: h, y: h, z: -h },  // 2
            Point3D { x: -h, y: h, z: -h }, // 3
            Point3D { x: -h, y: -h, z: h }, // 4
            Point3D { x: h, y: -h, z: h },  // 5
            Point3D { x: h, y: h, z: h },   // 6
            Point3D { x: -h, y: h, z: h },  // 7
        ];

        let edges = alloc::vec![
            (0, 1),
            (1, 2),
            (2, 3),
            (3, 0), // Back face
            (4, 5),
            (5, 6),
            (6, 7),
            (7, 4), // Front face
            (0, 4),
            (1, 5),
            (2, 6),
            (3, 7), // Connecting edges
        ];

        Self {
            vertices,
            edges,
            color,
        }
    }

    pub fn new_bar(width: f32, height: f32, depth: f32, color: Color) -> Self {
        let w = width / 2.0;
        let h_base = 0.0; // Bar starts at bottom
        let h_top = -height; // In 3D space, Y is often down, but let's assume standard projection
        let d = depth / 2.0;

        let vertices = alloc::vec![
            Point3D {
                x: -w,
                y: h_base,
                z: -d
            }, // 0: Bottom-back-left
            Point3D {
                x: w,
                y: h_base,
                z: -d
            }, // 1: Bottom-back-right
            Point3D {
                x: w,
                y: h_base,
                z: d
            }, // 2: Bottom-front-right
            Point3D {
                x: -w,
                y: h_base,
                z: d
            }, // 3: Bottom-front-left
            Point3D {
                x: -w,
                y: h_top,
                z: -d
            }, // 4: Top-back-left
            Point3D {
                x: w,
                y: h_top,
                z: -d
            }, // 5: Top-back-right
            Point3D {
                x: w,
                y: h_top,
                z: d
            }, // 6: Top-front-right
            Point3D {
                x: -w,
                y: h_top,
                z: d
            }, // 7: Top-front-left
        ];

        let edges = alloc::vec![
            (0, 1),
            (1, 2),
            (2, 3),
            (3, 0), // Bottom face
            (4, 5),
            (5, 6),
            (6, 7),
            (7, 4), // Top face
            (0, 4),
            (1, 5),
            (2, 6),
            (3, 7), // Verticals
        ];

        Self {
            vertices,
            edges,
            color,
        }
    }

    pub fn new_node_sphere(radius: f32, segments: usize, color: Color) -> Self {
        let mut vertices = Vec::new();
        let mut edges = Vec::new();

        // Simple latitude/longitude wireframe
        for i in 0..segments {
            let lat = (core::f32::consts::PI * i as f32) / (segments as f32 - 1.0)
                - core::f32::consts::PI / 2.0;
            let sin_lat = libm::sinf(lat);
            let cos_lat = libm::cosf(lat);

            for j in 0..segments {
                let lon = (2.0 * core::f32::consts::PI * j as f32) / (segments as f32);
                let sin_lon = libm::sinf(lon);
                let cos_lon = libm::cosf(lon);

                vertices.push(Point3D {
                    x: radius * cos_lat * cos_lon,
                    y: radius * sin_lat,
                    z: radius * cos_lat * sin_lon,
                });

                let current = i * segments + j;
                let next_lon = i * segments + (j + 1) % segments;
                edges.push((current, next_lon));

                if i < segments - 1 {
                    let next_lat = (i + 1) * segments + j;
                    edges.push((current, next_lat));
                }
            }
        }

        Self {
            vertices,
            edges,
            color,
        }
    }
}

pub struct HologramRenderer {
    width: usize,
    height: usize,
    fov: f32,
    viewer_distance: f32,
}

impl HologramRenderer {
    pub fn new(width: usize, height: usize) -> Self {
        Self {
            width,
            height,
            fov: 256.0,
            viewer_distance: 4.0,
        }
    }

    fn project(&self, p: Point3D) -> Point2D {
        // Simple perspective projection
        let z = p.z + self.viewer_distance;
        let factor = if z > 0.1 {
            self.fov / z
        } else {
            self.fov / 0.1
        };

        Point2D {
            x: (p.x * factor) as isize + (self.width as isize / 2),
            y: (p.y * factor) as isize + (self.height as isize / 2),
        }
    }

    fn rotate_y(p: Point3D, angle: f32) -> Point3D {
        let s = libm::sinf(angle);
        let c = libm::cosf(angle);
        Point3D {
            x: p.x * c - p.z * s,
            y: p.y,
            z: p.x * s + p.z * c,
        }
    }

    fn rotate_x(p: Point3D, angle: f32) -> Point3D {
        let s = libm::sinf(angle);
        let c = libm::cosf(angle);
        Point3D {
            x: p.x,
            y: p.y * c - p.z * s,
            z: p.y * s + p.z * c,
        }
    }

    pub fn render_mesh(
        &self,
        writer: &mut FramebufferWriter,
        mesh: &Mesh3D,
        angle_x: f32,
        angle_y: f32,
        offset_x: isize,
        offset_y: isize,
    ) {
        let mut projected = alloc::vec![];

        // Transform and project vertices
        for v in &mesh.vertices {
            let rotated = Self::rotate_x(*v, angle_x);
            let rotated = Self::rotate_y(rotated, angle_y);
            let mut p2d = self.project(rotated);
            p2d.x += offset_x;
            p2d.y += offset_y;
            projected.push(p2d);
        }

        // Draw edges
        for (idx1, idx2) in &mesh.edges {
            if *idx1 < projected.len() && *idx2 < projected.len() {
                let p1 = projected[*idx1];
                let p2 = projected[*idx2];
                writer.draw_line(p1.x, p1.y, p2.x, p2.y, mesh.color);
            }
        }
    }
}
