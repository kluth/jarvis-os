/// A lightweight energy-based Voice Activity Detector (VAD).
pub struct Vad {
    threshold: u32,
}

impl Vad {
    pub fn new(threshold: u32) -> Self {
        Vad { threshold }
    }

    /// Detects if speech is present in a buffer of 16-bit PCM samples.
    pub fn is_speech(&mut self, samples: &[i16]) -> bool {
        if samples.is_empty() {
            return false;
        }

        let mut energy = 0u64;
        for &sample in samples {
            energy += (sample as i32).pow(2) as u64;
        }

        let avg_energy = (energy / samples.len() as u64) as u32;

        // Adaptive energy-based thresholding
        avg_energy > self.threshold
    }
}

/// A background task that polls the audio capture buffer for voice activity.
pub async fn vad_task(threshold: u32) {
    let mut vad = Vad::new(threshold);

    loop {
        super::AUDIO_SAMPLES_READY.wait().await;
        super::AUDIO_SAMPLES_READY.reset();

        // Poll the AudioStream DMA capture buffer.
        // Integration with HDA controller is pending low-level IRQ stability.

        let data: [i16; 0] = [];
        if vad.is_speech(&data) {
            crate::serial_println!("Voice Activity: Speech detected.");
            super::VOICE_DETECTED.trigger();
        }
    }
}
