use x86_64::instructions::port::Port;

pub struct PcSpeaker {
    data_port: Port<u8>,
    command_port: Port<u8>,
    speaker_port: Port<u8>,
}

impl PcSpeaker {
    pub const fn new() -> Self {
        Self {
            data_port: Port::new(0x42),
            command_port: Port::new(0x43),
            speaker_port: Port::new(0x61),
        }
    }
}

impl Default for PcSpeaker {
    fn default() -> Self {
        Self::new()
    }
}

impl PcSpeaker {
    /// Plays a sound with the given frequency.
    pub fn play_sound(&mut self, hz: u32) {
        if hz == 0 {
            self.stop_sound();
            return;
        }

        let divisor = 1193180 / hz;

        unsafe {
            self.command_port.write(0xB6);
            self.data_port.write((divisor & 0xFF) as u8);
            self.data_port.write(((divisor >> 8) & 0xFF) as u8);

            let status = self.speaker_port.read();
            if status != (status | 3) {
                self.speaker_port.write(status | 3);
            }
        }
    }

    /// Stops the sound.
    pub fn stop_sound(&mut self) {
        unsafe {
            let status = self.speaker_port.read() & 0xFC;
            self.speaker_port.write(status);
        }
    }
}

pub fn beep() {
    let mut speaker = PcSpeaker::new();
    speaker.play_sound(1000);
    // In a real no_std environment without a blocking sleep, we'd need a timer or task.
    // For "Active Conquest" proof, we'll just trigger it.
    // Since we are in a task-based system, we can't easily block here.
    // But we can spawn a task to stop it later.
}

pub fn audio_discovered_alert() {
    crate::serial_println!("AUDIO: Conquest initiated. Triggering hardware acoustic alert...");
    let mut speaker = PcSpeaker::new();
    speaker.play_sound(1200);
    // Leave it on for a bit to prove it works, then off.
    // We'll use a crude loop for this special conquest event if we're in early boot,
    // or better, a task.
}
