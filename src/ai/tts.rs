use crate::storage::vfs::Result;
use alloc::vec::Vec;
use core::f32;

/// The Text-to-Speech (TTS) engine trait.
pub trait TtsEngine {
    /// Translates text into a buffer of raw PCM audio data.
    fn speak(&mut self, text: &str) -> Result<Vec<i16>>;
}

/// A simple software PCM tone generator — produces audible tones per character.
/// Used as a fallback when no external TTS engine (Flite, MaryTTS) is available.
pub struct SoftwareTts {
    sample_rate: u32,
    channels: u8,
    volume: f32,
}

impl Default for SoftwareTts {
    fn default() -> Self {
        Self {
            sample_rate: 22050,
            channels: 1,
            volume: 0.3,
        }
    }
}

impl SoftwareTts {
    pub fn new() -> Self {
        Self::default()
    }
}

impl TtsEngine for SoftwareTts {
    fn speak(&mut self, text: &str) -> Result<Vec<i16>> {
        // Generate distinct tones per character for auditory feedback.
        // Each character produces a short tone burst at a pitch determined by its ASCII value.
        let samples_per_char = self.sample_rate / 10; // 100ms per char
        let mut pcm = Vec::with_capacity(text.len() * samples_per_char as usize);

        for (idx, ch) in text.bytes().enumerate() {
            // Base frequency from character value (200-1200 Hz range)
            let base_freq = 200.0 + (ch as f32) * 5.0;
            // Frequency modulation for natural variation
            let freq = base_freq + (idx as f32 * 1.5) % 100.0;

            for i in 0..samples_per_char {
                let t = i as f32 / self.sample_rate as f32;
                let envelope = 1.0 - (i as f32 / samples_per_char as f32); // Linear decay
                let sample = libm::sinf(t * freq * core::f32::consts::TAU) * self.volume * envelope;

                // Apply slight harmonic for richness
                let harmonic = libm::sinf(t * freq * 2.0 * core::f32::consts::TAU)
                    * self.volume
                    * 0.3
                    * envelope;
                let mixed = sample + harmonic;

                let clamped =
                    (mixed * i16::MAX as f32).clamp(i16::MIN as f32, i16::MAX as f32) as i16;

                if self.channels == 1 {
                    pcm.push(clamped);
                } else {
                    pcm.push(clamped);
                    pcm.push(clamped); // Duplicate for stereo
                }
            }

            // Short gap between characters (10ms silence)
            let gap_samples = self.sample_rate / 100;
            for _ in 0..gap_samples {
                if self.channels == 1 {
                    pcm.push(0);
                } else {
                    pcm.push(0);
                    pcm.push(0);
                }
            }
        }

        Ok(pcm)
    }
}

/// JARVIS OS TTS Strategy:
/// 1. Low Footprint: Focus on embedded-friendly engines (Flite, MaryTTS).
/// 2. Audio Pipeline Integration: The output of speak() is fed directly into
///    the Output AudioStream.
/// 3. Async Synthesis: TTS generation should happen in a background task
///    to avoid stuttering during playback.
#[derive(Default)]
pub struct TtsManager {
    engine: Option<alloc::boxed::Box<dyn TtsEngine>>,
}

impl TtsManager {
    pub fn new() -> Self {
        // Auto-inject the software TTS as default fallback
        Self {
            engine: Some(alloc::boxed::Box::new(SoftwareTts::new())),
        }
    }

    pub fn set_engine(&mut self, engine: alloc::boxed::Box<dyn TtsEngine>) {
        self.engine = Some(engine);
    }

    pub fn speak(&mut self, text: &str) -> Result<Vec<i16>> {
        if let Some(ref mut engine) = self.engine {
            engine.speak(text)
        } else {
            Ok(Vec::new())
        }
    }
}
