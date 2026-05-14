use super::buffer::DmaBuffer;

pub enum StreamDirection {
    Input,
    Output,
}

pub struct AudioStream {
    buffer: DmaBuffer,
    direction: StreamDirection,
    sample_rate: u32,
    channels: u8,
}

impl AudioStream {
    pub fn new(buffer: DmaBuffer, direction: StreamDirection) -> Self {
        AudioStream {
            buffer,
            direction,
            sample_rate: 44100,
            channels: 2,
        }
    }

    pub fn start(&mut self) {
        // Here we would configure the HDA controller to start this stream
    }

    pub fn stop(&mut self) {
        // Here we would configure the HDA controller to stop this stream
    }
}
