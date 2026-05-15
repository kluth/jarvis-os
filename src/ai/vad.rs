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

        // Simple adaptive thresholding placeholder
        avg_energy > self.threshold
    }
}

/// A background task that polls an audio buffer for voice activity.
pub async fn vad_task(threshold: u32) {
    let mut vad = Vad::new(threshold);

    loop {
        // In a real implementation, we would poll the AudioStream DMA buffer here.
        // For now, we simulate waiting for data.

        let data: [i16; 0] = [];
        if vad.is_speech(&data) {
            crate::println!("Voice detected!");
        }

        crate::task::yield_now().await;
    }
}
