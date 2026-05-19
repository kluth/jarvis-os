use super::stt::SttEngine;
use super::tts::TtsEngine;
use crate::{device_manager, telemetry};

#[derive(Debug, Clone, Copy)]
pub enum Intent {
    SystemStatus,
    InitializeDiagnostics,
    ControlHardware {
        device: &'static str,
        action: &'static str,
    },
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
#[derive(Default)]
pub struct VoiceShell {
    _stt: Option<alloc::boxed::Box<dyn SttEngine>>,
    tts: Option<alloc::boxed::Box<dyn TtsEngine>>,
}

impl VoiceShell {
    pub fn new() -> Self {
        Self::default()
    }
}

impl VoiceShell {
    /// Processes a recognized command string and maps it to an Intent.
    pub fn handle_command(&mut self, text: &str) {
        crate::serial_println!("Voice Shell: Handling command -> {}", text);

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
                crate::serial_println!("JARVIS: Found {} devices in registry.", count);
                self.respond("Displaying all discovered hardware on the HUD.");
            }
            Intent::ScanNetwork => {
                self.respond("Scanning local subnet for active nodes.");
            }
            Intent::ControlHardware { device, action } => {
                crate::serial_println!("Action: {} on device: {}", action, device);
                self.respond("Command processed.");
            }
            Intent::Unknown => {
                self.respond("I'm sorry, I didn't catch that. Could you repeat?");
            }
        }
    }

    fn map_text_to_intent(&self, text: &str) -> Intent {
        let normalized = text.to_lowercase();

        // 1. Check for specific keywords
        if normalized.contains("status") || normalized.contains("how are you") {
            return Intent::SystemStatus;
        }
        if normalized.contains("diagnostic") || normalized.contains("check up") {
            return Intent::InitializeDiagnostics;
        }
        if normalized.contains("hello") || normalized.contains("hi") {
            return Intent::Greeting;
        }
        if normalized.contains("device") || normalized.contains("hardware") {
            return Intent::ListDevices;
        }
        if normalized.contains("network") || normalized.contains("scan") {
            return Intent::ScanNetwork;
        }

        // 2. Resolve complex intents using Capability-based discovery
        if normalized.contains("turn on")
            || normalized.contains("activate")
            || normalized.contains("start")
        {
            // Look for PowerControl capability
            let targets = device_manager::MANAGER
                .lock()
                .find_by_capability(device_manager::Capability::PowerControl);
            if !targets.is_empty() {
                // Heuristic: check if user mentioned a specific device name
                for dev in &targets {
                    if normalized.contains(&dev.name.to_lowercase()) {
                        return Intent::ControlHardware {
                            device: dev.name,
                            action: "on",
                        };
                    }
                }
                // If no specific name mentioned, default to the first one (or handle conflict)
                return Intent::ControlHardware {
                    device: targets[0].name,
                    action: "on",
                };
            }
        }

        if normalized.contains("volume")
            || normalized.contains("louder")
            || normalized.contains("quieter")
        {
            let targets = device_manager::MANAGER
                .lock()
                .find_by_capability(device_manager::Capability::VolumeControl);
            if !targets.is_empty() {
                return Intent::ControlHardware {
                    device: targets[0].name,
                    action: "adjust",
                };
            }
        }

        Intent::Unknown
    }

    fn respond(&mut self, text: &str) {
        crate::serial_println!("JARVIS: {}", text);
        if let Some(ref mut tts) = self.tts {
            let _ = tts.speak(text);
        }
    }
}

/// The 'Always-Listening' loop task.
pub async fn shell_task() {
    let mut _shell = VoiceShell::new();

    loop {
        super::VOICE_DETECTED.wait().await;
        super::VOICE_DETECTED.reset();

        // Here we would check the 'Voice Detected' signal from VAD
        // let text = stt.transcribe(buffer);
        // shell.handle_command(&text);
        crate::serial_println!("Voice Shell: Signal received, processing voice input...");
    }
}
