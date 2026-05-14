use crate::storage::vfs::Result;

/// The Speech-to-Text (STT) engine trait.
/// Implementations can wrap local inference engines (Whisper.cpp) 
/// or hardware accelerators.
pub trait SttEngine {
    /// Translates raw PCM audio data into a string.
    fn transcribe(&mut self, audio: &[i16]) -> Result<alloc::string::String>;
}

/// JARVIS OS STT Strategy:
/// 1. Local-First: Priority on low-latency local inference.
/// 2. Modular: Support for multiple backends via the SttEngine trait.
/// 3. Streaming: Transcription should happen in chunks to provide immediate feedback.
pub struct SttManager {
    engine: Option<alloc::boxed::Box<dyn SttEngine>>,
}

impl SttManager {
    pub fn new() -> Self {
        SttManager { engine: None }
    }

    pub fn set_engine(&mut self, engine: alloc::boxed::Box<dyn SttEngine>) {
        self.engine = Some(engine);
    }
}
