use crate::storage::vfs::Result;

/// The Text-to-Speech (TTS) engine trait.
pub trait TtsEngine {
    /// Translates text into a buffer of raw PCM audio data.
    fn speak(&mut self, text: &str) -> Result<alloc::vec::Vec<i16>>;
}

/// JARVIS OS TTS Strategy:
/// 1. Low Footprint: Focus on embedded-friendly engines (Flite, MaryTTS).
/// 2. Audio Pipeline Integration: The output of speak() is fed directly into
///    the Output AudioStream.
/// 3. Async Synthesis: TTS generation should happen in a background task
///    to avoid stuttering during playback.
#[derive(Default)]
pub struct TtsManager {
    _engine: Option<alloc::boxed::Box<dyn TtsEngine>>,
}

impl TtsManager {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn set_engine(&mut self, engine: alloc::boxed::Box<dyn TtsEngine>) {
        self._engine = Some(engine);
    }
}
