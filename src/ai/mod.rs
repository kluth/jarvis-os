pub mod perception;
pub mod shell;
pub mod stt;
pub mod swarm;
pub mod tts;
mod vad;
pub use vad::vad_task;

use crate::task::signal::Signal;

pub static VOICE_DETECTED: Signal = Signal::new();
pub static AUDIO_SAMPLES_READY: Signal = Signal::new();
