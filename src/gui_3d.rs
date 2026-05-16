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
