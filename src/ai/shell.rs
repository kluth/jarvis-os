use super::stt::SttEngine;
use super::tts::TtsEngine;
use crate::{println, telemetry, device_manager};

#[derive(Debug, Clone, Copy)]
pub enum Intent {
    SystemStatus,
    InitializeDiagnostics,
    ControlHardware { device: &'static str, action: &'static str },
    ListDevices,
    ScanNetwork,
    Greeting,
    Unknown,
}

impl Intent {
    pub fn as_str(&self) -> &'static str {
        match self {
            Intent::SystemStatus => "SystemStatus",
            Intent::InitializeDiagnostics => "InitializeDiagnostics",
            Intent::ControlHardware { .. } => "ControlHardware",
            Intent::ListDevices => "ListDevices",
            Intent::ScanNetwork => "ScanNetwork",
            Intent::Greeting => "Greeting",
            Intent::Unknown => "Unknown",
        }
    }
}

/// The Jarvis Voice Shell: The primary interaction model.
pub struct VoiceShell {
    _stt: Option<alloc::boxed::Box<dyn SttEngine>>,
    tts: Option<alloc::boxed::Box<dyn TtsEngine>>,
}

impl VoiceShell {
    pub fn new() -> Self {
        VoiceShell { _stt: None, tts: None }
    }

    /// Processes a recognized command string and maps it to an Intent.
    pub fn handle_command(&mut self, text: &str) {
        println!("Voice Shell: Handling command -> {}", text);
        
        let intent = self.map_text_to_intent(text);
        // Log the intent type (static str) instead of the dynamic input text
        telemetry::log(telemetry::TelemetryData::AIIntentDetected(intent.as_str()));

        match intent {
            Intent::SystemStatus => {
                self.respond("All systems are operational, sir.");
            }
            Intent::InitializeDiagnostics => {
                self.respond("Initiating full system diagnostics.");
            }
            Intent::Greeting => {
                self.respond("Hello. I am JARVIS. How can I help you?");
            }
            Intent::ListDevices => {
                let count = device_manager::MANAGER.lock().get_devices().len();
                println!("JARVIS: Found {} devices in registry.", count);
                self.respond("Displaying all discovered hardware on the HUD.");
            }
            Intent::ScanNetwork => {
                self.respond("Scanning local subnet for active nodes.");
            }
            Intent::ControlHardware { device, action } => {
                println!("Action: {} on device: {}", action, device);
                self.respond("Command processed.");
            }
            Intent::Unknown => {
                self.respond("I'm sorry, I didn't catch that. Could you repeat?");
            }
        }
    }

    fn map_text_to_intent(&self, text: &str) -> Intent {
        let normalized = text.to_lowercase();
        
        if normalized.contains("status") || normalized.contains("how are you") {
            Intent::SystemStatus
        } else if normalized.contains("diagnostic") || normalized.contains("check up") {
            Intent::InitializeDiagnostics
        } else if normalized.contains("hello") || normalized.contains("hi") {
            Intent::Greeting
        } else if normalized.contains("device") || normalized.contains("hardware") {
            Intent::ListDevices
        } else if normalized.contains("network") || normalized.contains("scan") {
            Intent::ScanNetwork
        } else if normalized.contains("turn on") {
            Intent::ControlHardware { device: "lights", action: "on" }
        } else {
            Intent::Unknown
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
    let mut _shell = VoiceShell::new();
    
    loop {
        // Here we would check the 'Voice Detected' signal from VAD
        // if voice_detected {
        //    let text = stt.transcribe(buffer);
        //    shell.handle_command(&text);
        // }
        
        core::future::ready(()).await;
    }
}
