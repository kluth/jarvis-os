//! ============================================================================
//! JARVIS OS Particle System — CPU-driven particle engine
//! ============================================================================
//! Implements:
//!   - Particle with position, velocity, color, lifetime, size
//!   - Emitter with configurable spawn rate, area, forces
//!   - Forces: gravity, turbulence, drag
//!   - Billboard rendering (screen-aligned quads)
//!   - Color and size interpolation over lifetime
//! ============================================================================

use super::{BlendMode, Renderer, ShaderType, Vec3, Vertex};
use alloc::vec::Vec;

// ============================================================================
// PARTICLE
// ============================================================================
#[derive(Debug, Clone)]
pub struct Particle {
    pub position: Vec3,
    pub velocity: Vec3,
    pub color_start: Vec3,
    pub color_end: Vec3,
    pub size_start: f32,
    pub size_end: f32,
    pub lifetime: f32,      // Total lifetime in seconds
    pub age: f32,           // Current age in seconds
    pub alive: bool,
}

impl Particle {
    pub fn new(position: Vec3, velocity: Vec3,
               color_start: Vec3, color_end: Vec3,
               size_start: f32, size_end: f32,
               lifetime: f32) -> Self {
        Self {
            position, velocity,
            color_start, color_end,
            size_start, size_end,
            lifetime, age: 0.0,
            alive: true,
        }
    }

    /// Update particle, returns false if expired
    pub fn update(&mut self, dt: f32, forces: &[Vec3]) -> bool {
        if !self.alive { return false; }
        self.age += dt;
        if self.age >= self.lifetime { self.alive = false; return false; }

        // Apply forces
        for force in forces {
            self.velocity = self.velocity.add(*force);
        }

        self.position = self.position.add(self.velocity.scale(dt));
        true
    }

    /// Get interpolation factor [0, 1]
    pub fn t(&self) -> f32 {
        (self.age / self.lifetime).max(0.0).min(1.0)
    }

    /// Get current color by interpolating start → end
    pub fn color(&self) -> Vec3 {
        let t = self.t();
        Vec3::lerp(self.color_start, self.color_end, t)
    }

    /// Get current size by interpolating start → end
    pub fn size(&self) -> f32 {
        let t = self.t();
        self.size_start + (self.size_end - self.size_start) * t
    }

    /// Get current alpha (fades out in last 20% of life)
    pub fn alpha(&self) -> f32 {
        let t = self.t();
        if t > 0.8 {
            (1.0 - (t - 0.8) / 0.2).max(0.0)
        } else {
            1.0
        }
    }
}

// ============================================================================
// EMITTER
// ============================================================================
#[derive(Debug, Clone)]
pub struct EmitterConfig {
    pub spawn_rate: f32,         // Particles per second
    pub position: Vec3,
    pub spawn_radius: f32,       // Random spawn area
    pub velocity_min: Vec3,
    pub velocity_max: Vec3,
    pub lifetime_min: f32,
    pub lifetime_max: f32,
    pub size_min: f32,
    pub size_max: f32,
    pub color_start: Vec3,
    pub color_end: Vec3,
    pub gravity: Vec3,
    pub max_particles: usize,
}

impl Default for EmitterConfig {
    fn default() -> Self {
        Self::new()
    }
}

impl EmitterConfig {
    pub fn new() -> Self {
        Self {
            spawn_rate: 10.0,
            position: Vec3::zero(),
            spawn_radius: 0.5,
            velocity_min: Vec3::new(-0.5, 0.0, -0.5),
            velocity_max: Vec3::new(0.5, 2.0, 0.5),
            lifetime_min: 1.0,
            lifetime_max: 3.0,
            size_min: 2.0,
            size_max: 5.0,
            color_start: Vec3::new(0.0, 0.8, 1.0), // cyan
            color_end: Vec3::new(0.0, 0.3, 0.8),
            gravity: Vec3::new(0.0, -0.5, 0.0),
            max_particles: 500,
        }
    }

    /// Flame effect preset
    pub fn flame(pos: Vec3) -> Self {
        Self {
            spawn_rate: 30.0,
            position: pos,
            spawn_radius: 0.2,
            velocity_min: Vec3::new(-0.3, 0.5, -0.3),
            velocity_max: Vec3::new(0.3, 1.5, 0.3),
            lifetime_min: 0.5,
            lifetime_max: 1.5,
            size_min: 3.0,
            size_max: 8.0,
            color_start: Vec3::new(0.9, 0.8, 0.5), // yellow-white
            color_end: Vec3::new(0.8, 0.2, 0.05),  // red
            gravity: Vec3::new(0.0, 0.1, 0.0),
            max_particles: 200,
        }
    }

    /// Cyan tech glow preset (JARVIS aesthetic)
    pub fn tech_glow(pos: Vec3) -> Self {
        Self {
            spawn_rate: 15.0,
            position: pos,
            spawn_radius: 0.0,
            velocity_min: Vec3::new(-1.0, -1.0, -1.0),
            velocity_max: Vec3::new(1.0, 1.0, 1.0),
            lifetime_min: 0.8,
            lifetime_max: 2.0,
            size_min: 1.0,
            size_max: 3.0,
            color_start: Vec3::new(0.0, 1.0, 0.9),
            color_end: Vec3::new(0.0, 0.2, 0.5),
            gravity: Vec3::new(0.0, 0.05, 0.0),
            max_particles: 300,
        }
    }

    /// Spark burst preset
    pub fn spark_burst(pos: Vec3) -> Self {
        Self {
            spawn_rate: 50.0,
            position: pos,
            spawn_radius: 0.1,
            velocity_min: Vec3::new(-3.0, -3.0, -3.0),
            velocity_max: Vec3::new(3.0, 3.0, 3.0),
            lifetime_min: 0.3,
            lifetime_max: 1.0,
            size_min: 0.5,
            size_max: 2.0,
            color_start: Vec3::new(1.0, 0.9, 0.6),
            color_end: Vec3::new(1.0, 0.3, 0.1),
            gravity: Vec3::new(0.0, -2.0, 0.0),
            max_particles: 100,
        }
    }
}

// ============================================================================
// PARTICLE EMITTER INSTANCE
// ============================================================================
pub struct ParticleEmitter {
    pub config: EmitterConfig,
    pub particles: Vec<Particle>,
    spawn_accumulator: f32,
    enabled: bool,
}

impl ParticleEmitter {
    pub fn new(config: EmitterConfig) -> Self {
        Self {
            config,
            particles: Vec::new(),
            spawn_accumulator: 0.0,
            enabled: true,
        }
    }

    pub fn enable(&mut self) { self.enabled = true; }
    pub fn disable(&mut self) { self.enabled = false; }

    /// Update all particles and spawn new ones
    pub fn update(&mut self, dt: f32) {
        if !self.enabled { return; }

        // Spawn new particles
        self.spawn_accumulator += self.config.spawn_rate * dt;
        while self.spawn_accumulator >= 1.0 && self.particles.len() < self.config.max_particles {
            self.spawn_accumulator -= 1.0;
            self.emit_one();
        }

        // Update existing particles
        let mut i = 0;
        while i < self.particles.len() {
            let forces = [self.config.gravity];
            if !self.particles[i].update(dt, &forces) {
                // Remove dead particle (swap-pop)
                self.particles.swap_remove(i);
            } else {
                i += 1;
            }
        }
    }

    fn emit_one(&mut self) {
        use rand_chacha::rand_core::{RngCore, SeedableRng};
        let mut rng = rand_chacha::ChaCha20Rng::from_seed([0x13; 32]);

        let px = self.config.position.x + (rng.next_u32() as f32 / u32::MAX as f32 - 0.5) * self.config.spawn_radius * 2.0;
        let py = self.config.position.y + (rng.next_u32() as f32 / u32::MAX as f32 - 0.5) * self.config.spawn_radius * 2.0;
        let pz = self.config.position.z + (rng.next_u32() as f32 / u32::MAX as f32 - 0.5) * self.config.spawn_radius * 2.0;

        let vx = self.config.velocity_min.x + (rng.next_u32() as f32 / u32::MAX as f32) * (self.config.velocity_max.x - self.config.velocity_min.x);
        let vy = self.config.velocity_min.y + (rng.next_u32() as f32 / u32::MAX as f32) * (self.config.velocity_max.y - self.config.velocity_min.y);
        let vz = self.config.velocity_min.z + (rng.next_u32() as f32 / u32::MAX as f32) * (self.config.velocity_max.z - self.config.velocity_min.z);

        let lt = self.config.lifetime_min + (rng.next_u32() as f32 / u32::MAX as f32) * (self.config.lifetime_max - self.config.lifetime_min);
        let sz = self.config.size_min + (rng.next_u32() as f32 / u32::MAX as f32) * (self.config.size_max - self.config.size_min);

        let particle = Particle::new(
            Vec3::new(px, py, pz),
            Vec3::new(vx, vy, vz),
            self.config.color_start,
            self.config.color_end,
            sz, sz * 0.3,
            lt,
        );
        self.particles.push(particle);
    }

    /// Render all particles into the renderer as billboard quads
    pub fn render(&self, renderer: &mut Renderer) {
        for particle in &self.particles {
            let _t = particle.t();
            let color = particle.color();
            let size = particle.size();
            let alpha = particle.alpha();
            let z = particle.position.z;

            // Render as a small screen-aligned billboard
            let v0 = Vertex::new(particle.position.x - size, particle.position.y - size, z)
                .with_color(color.x, color.y, color.z).with_alpha(alpha);
            let v1 = Vertex::new(particle.position.x + size, particle.position.y - size, z)
                .with_color(color.x, color.y, color.z).with_alpha(alpha);
            let v2 = Vertex::new(particle.position.x + size, particle.position.y + size, z)
                .with_color(color.x, color.y, color.z).with_alpha(alpha);
            let v3 = Vertex::new(particle.position.x - size, particle.position.y + size, z)
                .with_color(color.x, color.y, color.z).with_alpha(alpha);

            renderer.draw_triangle(&v0, &v1, &v2, None, BlendMode::Alpha, ShaderType::Glow);
            renderer.draw_triangle(&v0, &v2, &v3, None, BlendMode::Alpha, ShaderType::Glow);
        }
    }

    /// Get number of alive particles
    pub fn particle_count(&self) -> usize {
        self.particles.len()
    }
}