use super::stt::SttEngine;
use super::tts::TtsEngine;
use crate::println;

/// The Jarvis Voice Shell: The primary interaction model.
pub struct VoiceShell {
    stt: Option<alloc::boxed::Box<dyn SttEngine>>,
    tts: Option<alloc::boxed::Box<dyn TtsEngine>>,
}

impl VoiceShell {
    pub fn new() -> Self {
        VoiceShell { stt: None, tts: None }
    }

    /// Processes a recognized command string.
    pub fn handle_command(&mut self, text: &str) {
        println!("Voice Shell: Handling command -> {}", text);
        
        let normalized = text.to_lowercase();
        
        if normalized.contains("status") {
            self.respond("All systems are operational, sir.");
        } else if normalized.contains("hello") {
            self.respond("Hello. I am JARVIS. How can I help you?");
        } else {
            self.respond("I'm sorry, I didn't catch that.");
        }
    }

    fn respond(&mut self, text: &str) {
        println!("JARVIS: {}", text);
        if let Some(ref mut tts) = self.tts {
            let _ = tts.speak(text);
        }
    }
}

/// The 'Always-Listening' loop task.
pub async fn shell_task() {
    let mut shell = VoiceShell::new();
    
    loop {
        // Here we would check the 'Voice Detected' signal from VAD
        // if voice_detected {
        //    let text = stt.transcribe(buffer);
        //    shell.handle_command(&text);
        // }
        
        core::future::ready(()).await;
    }
}
