use super::buffer::DmaBuffer;

pub enum StreamDirection {
    Input,
    Output,
}

pub struct AudioStream {
    _buffer: DmaBuffer,
    _direction: StreamDirection,
    _sample_rate: u32,
    _channels: u8,
}

impl AudioStream {
    pub fn new(buffer: DmaBuffer, direction: StreamDirection) -> Self {
        AudioStream {
            _buffer: buffer,
            _direction: direction,
            _sample_rate: 44100,
            _channels: 2,
        }
    }

    pub fn start(&mut self) {
        // Here we would configure the HDA controller to start this stream
    }

    pub fn stop(&mut self) {
        // Here we would configure the HDA controller to stop this stream
    }
}
