use core::arch::x86_64::{__cpuid, _rdrand64_step, _rdtsc};

/// A utility to obtain hardware entropy for cryptographic operations.
/// Follows the "No Mocks" mandate.
pub fn get_entropy_64() -> u64 {
    let mut val: u64 = 0;
    
    // Check for RDRAND support (CPUID.01H:ECX.bit30)
    let has_rdrand = unsafe {
        let result = __cpuid(0x1);
        (result.ecx & (1 << 30)) != 0
    };

    unsafe {
        if has_rdrand && _rdrand64_step(&mut val) != 0 {
            return val;
        } else {
            // Fallback: high-resolution timestamp mixed with stack pointer and address
            let tsc = _rdtsc();
            let stack_ptr = &val as *const _ as u64;
            let mut mix = tsc ^ stack_ptr;
            
            // MurmurHash3-style finalizer for mixing
            mix ^= mix >> 33;
            mix = mix.wrapping_mul(0xff51afd7ed558ccd);
            mix ^= mix >> 33;
            mix = mix.wrapping_mul(0xc4ceb9fe1a85ec53);
            mix ^= mix >> 33;
            
            return mix;
        }
    }
}

pub fn fill_entropy(dest: &mut [u8]) {
    for chunk in dest.chunks_mut(8) {
        let val = get_entropy_64();
        let bytes = val.to_le_bytes();
        let len = chunk.len();
        chunk.copy_from_slice(&bytes[..len]);
    }
}
