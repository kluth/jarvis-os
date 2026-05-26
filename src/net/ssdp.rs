//! ============================================================================
//! JARVIS OS SSDP (Simple Service Discovery Protocol)
//! ============================================================================
//! Implements PERC-003: UPnP/SSDP discovery for local network perception.
//! ============================================================================

use crate::println;
use alloc::string::{String, ToString};

pub const SSDP_MULTICAST_ADDR: &str = "239.255.255.250";
pub const SSDP_PORT: u16 = 1900;

#[derive(Debug, Clone)]
pub struct SsdpService {
    pub location: String,
    pub st: String,  // Search Target
    pub usn: String, // Unique Service Name
}

pub fn parse_ssdp_response(packet: &[u8]) -> Option<SsdpService> {
    let data = String::from_utf8_lossy(packet);
    if !data.starts_with("HTTP/1.1 200 OK") {
        return None;
    }

    let mut location = String::new();
    let mut st = String::new();
    let mut usn = String::new();

    for line in data.lines() {
        let line = line.trim();
        if line.to_uppercase().starts_with("LOCATION:") {
            location = line[9..].trim().to_string();
        } else if line.to_uppercase().starts_with("ST:") {
            st = line[3..].trim().to_string();
        } else if line.to_uppercase().starts_with("USN:") {
            usn = line[4..].trim().to_string();
        }
    }

    if !location.is_empty() && !st.is_empty() {
        Some(SsdpService { location, st, usn })
    } else {
        None
    }
}

pub async fn discovery_task() {
    println!("Net: Starting SSDP discovery task...");

    loop {
        // In a real system, we'd send a M-SEARCH multicast packet here
        // and listen for responses.

        // Simulate a discovery response for now (following protocol format)
        let mock_response = b"HTTP/1.1 200 OK\r\n\
                             LOCATION: http://192.168.1.1:80/description.xml\r\n\
                             ST: upnp:rootdevice\r\n\
                             USN: uuid:f40c2981-7329-40b7-a6f6-f6c9441cb0d0::upnp:rootdevice\r\n\r\n";

        if let Some(_service) = parse_ssdp_response(mock_response) {
            // println!("Net: Discovered SSDP Service: {} at {}", service.st, service.location);
        }

        crate::task::sleep(1000).await; // Scan every 10 seconds
    }
}
