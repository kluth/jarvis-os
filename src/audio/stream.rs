use super::buffer::DmaBuffer;
use x86_64::VirtAddr;

pub enum StreamDirection {
    Input,
    Output,
}

/// HDA Stream Descriptor Registers (offset from base + 0x80 * stream_id)
/// Output streams: 0, 2, 4... Input streams: 1, 3, 5...
const SD_CTL: usize = 0x00; // Stream Descriptor Control (4 bytes)
const SD_STS: usize = 0x03; // Stream Descriptor Status (1 byte)
const SD_BDLPL: usize = 0x04; // BDL Pointer Lower (4 bytes)
const SD_BDLPU: usize = 0x08; // BDL Pointer Upper (4 bytes)
const SD_CBL: usize = 0x0C; // Cyclic Buffer Length (4 bytes)
const SD_LVI: usize = 0x10; // Last Valid Index (2 bytes)

// SD_CTL bits
const SD_CTL_RUN: u32 = 1 << 0;
const SD_CTL_SRST: u32 = 1 << 1;
const SD_CTL_STRIPE: u32 = 1 << 2;
const SD_CTL_TP: u32 = 1 << 3;
const SD_CTL_DEIE: u32 = 1 << 4;
const SD_CTL_FIFO_ERROR: u32 = 1 << 5;
const SD_CTL_IOCE: u32 = 1 << 6; // Interrupt on Completion Enable
const SD_CTL_FEIE: u32 = 1 << 7; // FIFO Error Interrupt Enable

// SD_STS bits
const SD_STS_BCIS: u8 = 1 << 2; // Buffer Completion Interrupt Status
const SD_STS_FIFO_READY: u8 = 1 << 3;
const SD_STS_DESE: u8 = 1 << 4; // Descriptor Error

pub struct AudioStream {
    buffer: DmaBuffer,
    _direction: StreamDirection,
    _sample_rate: u32,
    _channels: u8,
    hda_base: VirtAddr,
    stream_id: u8,
}

impl AudioStream {
    pub fn new(
        buffer: DmaBuffer,
        direction: StreamDirection,
        hda_base: VirtAddr,
        stream_id: u8,
    ) -> Self {
        AudioStream {
            buffer,
            _direction: direction,
            _sample_rate: 44100,
            _channels: 2,
            hda_base,
            stream_id,
        }
    }

    /// Configures and starts the HDA audio stream.
    ///
    /// # Safety
    ///
    /// The caller must ensure the HDA base address is valid and mapped,
    /// the DMA buffer is physically contiguous and correctly aligned,
    /// and that no other stream is using the same stream_id.
    pub unsafe fn start(&mut self) {
        let _stride = self._channels as u32 * 2; // 16-bit samples
        let stream_offset = 0x80 * (self.stream_id as usize);
        let sd_base = self.hda_base.as_mut_ptr::<u8>().add(stream_offset);

        // 1. Reset the stream via SRST
        let ctl_ptr = sd_base.add(SD_CTL).cast::<u32>();
        ctl_ptr.write_volatile(SD_CTL_SRST);
        // Wait for SRST to self-clear (reset complete) with timeout
        let mut timeout = 100_000;
        while ctl_ptr.read_volatile() & SD_CTL_SRST != 0 && timeout > 0 {
            timeout -= 1;
            core::hint::spin_loop();
        }

        // 2. Set up Buffer Descriptor List (BDL) — single entry pointing to our DMA buffer
        let bdl_phys = self.buffer.phys_addr();
        let bdl_phys_low = (bdl_phys.as_u64() & 0xFFFF_FFFF) as u32;
        let bdl_phys_high = (bdl_phys.as_u64() >> 32) as u32;

        // BDL Pointer Lower
        let bdlpl_ptr = sd_base.add(SD_BDLPL).cast::<u32>();
        bdlpl_ptr.write_volatile(bdl_phys_low);

        // BDL Pointer Upper
        let bdlpu_ptr = sd_base.add(SD_BDLPU).cast::<u32>();
        bdlpu_ptr.write_volatile(bdl_phys_high);

        // Cyclic Buffer Length (total bytes in the DMA buffer)
        let cbl_ptr = sd_base.add(SD_CBL).cast::<u32>();
        cbl_ptr.write_volatile(self.buffer.size() as u32);

        // Last Valid Index (0 for single-entry BDL, set IOC for interrupt)
        let lvi_ptr = sd_base.add(SD_LVI).cast::<u16>();
        lvi_ptr.write_volatile(0);

        // 3. Clear any pending status bits
        let sts_ptr = sd_base.add(SD_STS).cast::<u8>();
        sts_ptr.write_volatile(SD_STS_BCIS | SD_STS_DESE);

        // 4. Start the stream — set RUN, IOCE (interrupt on completion), TP (traffic priority)
        ctl_ptr.write_volatile(SD_CTL_RUN | SD_CTL_IOCE | SD_CTL_TP);

        // 5. Set stream format in the Output Stream Descriptor Format register
        // Format: bits 15-14 = 0 (8-bit) or 1 (16-bit) or 2 (20-bit) or 3 (24-bit)
        //         bits 13-3 = base rate (0x1F = 48kHz base)
        //         bits 2-0 = rate multiplier (0 = 1x, 1 = 2x, 2 = 4x)
        let fmt_ptr = sd_base.add(0x12).cast::<u16>();
        let sample_bits = 1u16 << 14; // 16-bit samples
        let base_rate = 0x1Fu16 << 3; // 48kHz base
        let channels = ((self._channels as u16) - 1) & 0xF;
        let fmt_val = sample_bits | base_rate | channels;
        fmt_ptr.write_volatile(fmt_val);
    }

    /// Stops the HDA audio stream.
    ///
    /// # Safety
    ///
    /// The caller must ensure the HDA base address is still valid.
    pub unsafe fn stop(&mut self) {
        let stream_offset = 0x80 * (self.stream_id as usize);
        let sd_base = self.hda_base.as_mut_ptr::<u8>().add(stream_offset);
        let ctl_ptr = sd_base.add(SD_CTL).cast::<u32>();

        // Clear RUN bit to stop the stream
        ctl_ptr.write_volatile(ctl_ptr.read_volatile() & !SD_CTL_RUN);

        // Wait for stream to actually stop with timeout
        let mut timeout = 100_000;
        while ctl_ptr.read_volatile() & SD_CTL_RUN != 0 && timeout > 0 {
            timeout -= 1;
            core::hint::spin_loop();
        }
    }
}
