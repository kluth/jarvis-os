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

/// A triangle face with average depth for painter's algorithm
#[derive(Clone)]
pub struct Face {
    pub v0: Point3D,
    pub v1: Point3D,
    pub v2: Point3D,
    pub color: Color,
    pub avg_z: f32,
}

impl Face {
    fn new(v0: Point3D, v1: Point3D, v2: Point3D, color: Color) -> Self {
        let avg_z = (v0.z + v1.z + v2.z) / 3.0;
        Face { v0, v1, v2, color, avg_z }
    }
}

pub struct Mesh3D {
    pub vertices: Vec<Point3D>,
    pub faces: Vec<Face>,
}

impl Mesh3D {
    /// Create a solid shaded bar with different colors for each face
    pub fn new_bar(width: f32, height: f32, depth: f32, base_color: Color) -> Self {
        let w = width / 2.0;
        let h_top = -height;
        let h_bot = 0.0;
        let d = depth / 2.0;

        // Vertices: [bottom-back-left, bottom-back-right, bottom-front-right, bottom-front-left,
        //             top-back-left, top-back-right, top-front-right, top-front-left]
        let verts = alloc::vec![
            Point3D { x: -w, y: h_bot, z: -d },  // 0
            Point3D { x:  w, y: h_bot, z: -d },  // 1
            Point3D { x:  w, y: h_bot, z:  d },  // 2
            Point3D { x: -w, y: h_bot, z:  d },  // 3
            Point3D { x: -w, y: h_top, z: -d },  // 4
            Point3D { x:  w, y: h_top, z: -d },  // 5
            Point3D { x:  w, y: h_top, z:  d },  // 6
            Point3D { x: -w, y: h_top, z:  d },  // 7
        ];

        // Shading factors: front=1.0 (brightest), top=0.7, side=0.5
        let front_light = scale_color(base_color, 1.0);
        let top_light = scale_color(base_color, 0.7);
        let side_light = scale_color(base_color, 0.5);
        let bottom_light = scale_color(base_color, 0.3);

        // 6 faces × 2 triangles = 12 faces total
        // Each face defined as 2 triangles
        let mut faces = Vec::new();

        // Front face (z=+d): vertices 3,2,6,7
        faces.push(Face::new(verts[3], verts[2], verts[6], front_light));
        faces.push(Face::new(verts[3], verts[6], verts[7], front_light));

        // Back face (z=-d): vertices 0,1,5,4
        faces.push(Face::new(verts[0], verts[1], verts[5], side_light));
        faces.push(Face::new(verts[0], verts[5], verts[4], side_light));

        // Right face (x=+w): vertices 1,2,6,5
        faces.push(Face::new(verts[1], verts[2], verts[6], side_light));
        faces.push(Face::new(verts[1], verts[6], verts[5], side_light));

        // Left face (x=-w): vertices 0,3,7,4
        faces.push(Face::new(verts[0], verts[3], verts[7], side_light));
        faces.push(Face::new(verts[0], verts[7], verts[4], side_light));

        // Top face (y=h_top): vertices 4,5,6,7
        faces.push(Face::new(verts[4], verts[5], verts[6], top_light));
        faces.push(Face::new(verts[4], verts[6], verts[7], top_light));

        // Bottom face (y=h_bot): vertices 0,1,2,3
        faces.push(Face::new(verts[0], verts[1], verts[2], bottom_light));
        faces.push(Face::new(verts[0], verts[2], verts[3], bottom_light));

        Mesh3D { vertices: verts, faces }
    }

    /// Create a solid shaded sphere using triangle strips
    pub fn new_node_sphere(radius: f32, segments: usize, base_color: Color) -> Self {
        let mut verts = Vec::new();
        // Generate vertices via latitude/longitude
        for i in 0..=segments {
            let lat = (core::f32::consts::PI * i as f32) / segments as f32
                - core::f32::consts::PI / 2.0;
            let sin_lat = libm::sinf(lat);
            let cos_lat = libm::cosf(lat);
            for j in 0..segments {
                let lon = (2.0 * core::f32::consts::PI * j as f32) / segments as f32;
                let sin_lon = libm::sinf(lon);
                let cos_lon = libm::cosf(lon);
                verts.push(Point3D {
                    x: radius * cos_lat * cos_lon,
                    y: radius * sin_lat,
                    z: radius * cos_lat * sin_lon,
                });
            }
        }

        // Generate triangle faces
        let mut faces = Vec::new();
        for i in 0..segments {
            for j in 0..segments {
                let a = i * segments + j;
                let b = a + segments;
                let c = b + 1;
                let d = a + 1;

                // Shade based on vertex normal (use Y component for simple top-down light)
                let shade_a = 0.3 + (verts[a].y / radius).max(-1.0).min(1.0).abs() * 0.7;
                let shade_b = 0.3 + (verts[b].y / radius).max(-1.0).min(1.0).abs() * 0.7;
                let shade_c = 0.3 + (verts[c].y / radius).max(-1.0).min(1.0).abs() * 0.7;
                let shade_d = 0.3 + (verts[d].y / radius).max(-1.0).min(1.0).abs() * 0.7;

                let avg = (shade_a + shade_b + shade_c) / 3.0;
                let color = scale_color(base_color, avg);
                faces.push(Face::new(verts[a], verts[b], verts[c], color));

                let avg2 = (shade_a + shade_c + shade_d) / 3.0;
                let color2 = scale_color(base_color, avg2);
                faces.push(Face::new(verts[a], verts[c], verts[d], color2));
            }
        }

        Mesh3D { vertices: verts, faces }
    }
}

fn scale_color(c: Color, factor: f32) -> Color {
    Color {
        r: (c.r as f32 * factor) as u8,
        g: (c.g as f32 * factor) as u8,
        b: (c.b as f32 * factor) as u8,
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
        let z = p.z + self.viewer_distance;
        let factor = if z > 0.1 { self.fov / z } else { self.fov / 0.1 };
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

    /// Render mesh with solid filled faces using painter's algorithm
    pub fn render_mesh(
        &self,
        writer: &mut FramebufferWriter,
        mesh: &Mesh3D,
        angle_x: f32,
        angle_y: f32,
        offset_x: isize,
        offset_y: isize,
    ) {
        // Transform and project all vertices
        let mut projected: Vec<Point2D> = Vec::new();
        for v in &mesh.vertices {
            let rotated = Self::rotate_x(*v, angle_x);
            let rotated = Self::rotate_y(rotated, angle_y);
            let mut p2d = self.project(rotated);
            p2d.x += offset_x;
            p2d.y += offset_y;
            projected.push(p2d);
        }

        // Build projected faces with updated avg_z and 2D positions
        let mut projected_faces: Vec<(Face, Point2D, Point2D, Point2D)> = Vec::new();
        for face in &mesh.faces {
            // Find indices in the mesh's vertices list
            let idx0 = mesh.vertices.iter().position(|v| v.x == face.v0.x && v.y == face.v0.y && v.z == face.v0.z).unwrap_or(0);
            let idx1 = mesh.vertices.iter().position(|v| v.x == face.v1.x && v.y == face.v1.y && v.z == face.v1.z).unwrap_or(0);
            let idx2 = mesh.vertices.iter().position(|v| v.x == face.v2.x && v.y == face.v2.y && v.z == face.v2.z).unwrap_or(0);
            
            if idx0 >= projected.len() || idx1 >= projected.len() || idx2 >= projected.len() {
                continue;
            }

            let p0 = projected[idx0];
            let p1 = projected[idx1];
            let p2 = projected[idx2];

            // Compute 2D area to cull backfaces (CW = visible)
            let area = ((p1.x - p0.x) * (p2.y - p0.y) - (p2.x - p0.x) * (p1.y - p0.y)) as f32;
            if area <= 0.0 {
                continue; // Backface culling
            }

            // Recompute avg_z from projected vertices (use z from depth buffer equivalent)
            // Actually use the face's stored avg_z (3D z before projection)
            projected_faces.push((face.clone(), p0, p1, p2));
        }

        // Sort by depth (painter's algorithm: back to front = farthest first)
        projected_faces.sort_by(|a, b| {
            b.0.avg_z.partial_cmp(&a.0.avg_z).unwrap_or(core::cmp::Ordering::Equal)
        });

        // Draw filled triangles
        for (face, p0, p1, p2) in &projected_faces {
            fill_triangle(writer, *p0, *p1, *p2, face.color);
        }
    }
}

/// Scanline triangle fill algorithm
fn fill_triangle(writer: &mut FramebufferWriter, v0: Point2D, v1: Point2D, v2: Point2D, color: Color) {
    let mut pts = [v0, v1, v2];
    // Sort by y ascending
    if pts[0].y > pts[1].y { pts.swap(0, 1); }
    if pts[1].y > pts[2].y { pts.swap(1, 2); }
    if pts[0].y > pts[1].y { pts.swap(0, 1); }

    let [p0, p1, p2] = pts;

    let total_height = p2.y - p0.y;
    if total_height == 0 { return; }

    let bounds = writer.get_info();
    let fb_width = bounds.width as isize;
    let fb_height = bounds.height as isize;

    // Fill the upper half (p0 -> p1)
    for y in p0.y..=p1.y {
        let dy1 = y - p0.y;
        let dy2 = y - p0.y;
        let segment_height = p1.y - p0.y;

        // Interpolate x along edges p0-p2 and p0-p1
        let x1 = interpolate_x(p0.x, p2.x, dy1, total_height);
        let x2 = if segment_height > 0 {
            interpolate_x(p0.x, p1.x, dy2, segment_height)
        } else {
            x1
        };

        let (mut x_start, mut x_end) = if x1 < x2 { (x1, x2) } else { (x2, x1) };
        if x_start < 0 { x_start = 0; }
        if x_end >= fb_width { x_end = fb_width - 1; }
        if y >= 0 && y < fb_height {
            for x in x_start..=x_end {
                writer.write_pixel(x as usize, y as usize, color);
            }
        }
    }

    // Fill the lower half (p1 -> p2)
    for y in p1.y..=p2.y {
        let dy1 = y - p1.y;
        let dy2 = y - p1.y;
        let segment_height = p2.y - p1.y;

        let x1 = interpolate_x(p0.x, p2.x, y - p0.y, total_height);
        let x2 = if segment_height > 0 {
            interpolate_x(p1.x, p2.x, dy1, segment_height)
        } else {
            x1
        };

        let (mut x_start, mut x_end) = if x1 < x2 { (x1, x2) } else { (x2, x1) };
        if x_start < 0 { x_start = 0; }
        if x_end >= fb_width { x_end = fb_width - 1; }
        if y >= 0 && y < fb_height {
            for x in x_start..=x_end {
                writer.write_pixel(x as usize, y as usize, color);
            }
        }
    }
}

fn interpolate_x(x0: isize, x1: isize, dy: isize, total_dy: isize) -> isize {
    if total_dy == 0 { return x0; }
    x0 + (x1 - x0) * dy / total_dy
}
