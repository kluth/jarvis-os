use crate::device_manager::{Capability, DeviceType};
use alloc::vec::Vec;

pub struct InferenceResult {
    pub dev_type: DeviceType,
    pub capabilities: Vec<Capability>,
    pub confidence: f32,
}

pub fn infer_device_from_json(json: &str) -> Option<InferenceResult> {
    let mut capabilities = Vec::new();
    let mut score_light = 0.0;
    let mut score_sensor = 0.0;
    let mut score_audio = 0.0;

    // A real implementation would use a proper JSON parser (e.g., serde_json)
    // but we can start with a robust pattern matcher that handles key/value pairs.

    if json.contains("\"power\"") || json.contains("\"state\"") {
        capabilities.push(Capability::PowerControl);
        score_light += 0.5;
    }

    if json.contains("\"brightness\"") || json.contains("\"dimmer\"") || json.contains("\"level\"")
    {
        capabilities.push(Capability::BrightnessControl);
        score_light += 1.0;
    }

    if json.contains("\"volume\"") || json.contains("\"mute\"") {
        capabilities.push(Capability::VolumeControl);
        score_audio += 1.0;
    }

    if json.contains("\"temperature\"")
        || json.contains("\"humidity\"")
        || json.contains("\"sensor\"")
    {
        capabilities.push(Capability::Diagnostic);
        score_sensor += 1.0;
    }

    // Determine the best match
    if score_light > score_sensor && score_light > score_audio {
        let confidence = score_light / 2.0;
        Some(InferenceResult {
            dev_type: DeviceType::Light,
            capabilities,
            confidence: if confidence > 1.0 { 1.0 } else { confidence },
        })
    } else if score_audio > score_sensor {
        let confidence = score_audio / 1.0;
        Some(InferenceResult {
            dev_type: DeviceType::Audio,
            capabilities,
            confidence: if confidence > 1.0 { 1.0 } else { confidence },
        })
    } else if score_sensor > 0.0 {
        let confidence = score_sensor / 1.0;
        Some(InferenceResult {
            dev_type: DeviceType::System, // Sensors map to System/Diagnostic for now
            capabilities,
            confidence: if confidence > 1.0 { 1.0 } else { confidence },
        })
    } else {
        None
    }
}

pub fn test_inference() {
    let light_payload = r#"{"state": "on", "brightness": 255}"#;
    let result = infer_device_from_json(light_payload).expect("Inference failed for light");
    assert!(result.dev_type == DeviceType::Light);
    assert!(result.capabilities.contains(&Capability::PowerControl));
    assert!(result.capabilities.contains(&Capability::BrightnessControl));

    let audio_payload = r#"{"volume": 50, "mute": false}"#;
    let result = infer_device_from_json(audio_payload).expect("Inference failed for audio");
    assert!(result.dev_type == DeviceType::Audio);
    assert!(result.capabilities.contains(&Capability::VolumeControl));
}
