use alloc::string::String;
use alloc::sync::Arc;
use alloc::vec::Vec;
use core::sync::atomic::{AtomicUsize, Ordering};
use lazy_static::lazy_static;
use spinning_top::Spinlock;

use crate::println;

#[derive(Debug, Clone)]
pub struct Point3D {
    pub x: f32,
    pub y: f32,
    pub z: f32,
}

#[derive(Debug, Clone)]
pub struct BoundingBox {
    pub center: Point3D,
    pub size: Point3D,
    pub label: String,
    pub confidence: u8,
}

pub struct SceneGraph {
    objects: Spinlock<alloc::collections::BTreeMap<usize, BoundingBox>>,
    update_counter: AtomicUsize,
}

impl Default for SceneGraph {
    fn default() -> Self {
        Self {
            objects: Spinlock::new(alloc::collections::BTreeMap::new()),
            update_counter: AtomicUsize::new(0),
        }
    }
}

impl SceneGraph {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn register_object(&self, bbox: BoundingBox) -> usize {
        let id = self.update_counter.fetch_add(1, Ordering::SeqCst);
        println!("Scene: Registered object '{}' at ID {}", bbox.label, id);
        self.objects.lock().insert(id, bbox);
        id
    }

    pub fn update_object(&self, id: usize, new_bbox: BoundingBox) {
        if let Some(obj) = self.objects.lock().get_mut(&id) {
            // println!("Scene: Updated object '{}' (ID {})", obj.label, id);
            *obj = new_bbox;
        }
    }

    pub fn remove_object(&self, id: usize) {
        if self.objects.lock().remove(&id).is_some() {
            println!("Scene: Removed object ID {}", id);
        }
    }

    pub fn get_objects(&self) -> Vec<BoundingBox> {
        self.objects.lock().values().cloned().collect()
    }
}

lazy_static! {
    pub static ref SCENE: Arc<SceneGraph> = Arc::new(SceneGraph::new());
}

/// Background task to process simulated sensor data into a scene graph.
pub async fn scene_task() {
    println!("Scene Reconstruction: Task initialized.");

    // Simulate initial scene discovery
    let desk = BoundingBox {
        center: Point3D {
            x: 0.0,
            y: 0.0,
            z: 0.0,
        },
        size: Point3D {
            x: 1.5,
            y: 0.8,
            z: 0.75,
        },
        label: String::from("Desk"),
        confidence: 99,
    };
    let _id = SCENE.register_object(desk);

    loop {
        // In a real system, we would poll spatial sensors (LiDAR, cameras)
        // and update the bounding boxes via Wasm drivers.

        crate::task::yield_now().await;
    }
}

#[cfg(feature = "test")]
pub fn test_scene_logic() {
    crate::serial_print!("test_scene_logic... ");
    let graph = SceneGraph::new();

    let box1 = BoundingBox {
        center: Point3D {
            x: 1.0,
            y: 1.0,
            z: 1.0,
        },
        size: Point3D {
            x: 0.5,
            y: 0.5,
            z: 0.5,
        },
        label: String::from("Coffee Mug"),
        confidence: 90,
    };

    let id = graph.register_object(box1);
    assert_eq!(graph.get_objects().len(), 1);

    graph.remove_object(id);
    assert_eq!(graph.get_objects().len(), 0);

    crate::serial_println!("[ok]");
}
