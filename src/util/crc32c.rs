
use std::arch::x86_64::{_mm_crc32_u64, _mm_crc32_u8};
const CRC_POLY_REV: u32 = 0x82F63B78;
const INITIAL_INV: u32 = 0;

const BITS_PER_BYTE: u8 = 8;
const BIT_MASK_8: u8 = 0xFF;

const LOOKUP_TABLE_SIZE: usize = 256;
const LOOKUP_TABLE: [u32; LOOKUP_TABLE_SIZE] = compute_lookup_table();

pub fn crc32c(buf: &[u8]) -> u32 {
    crc32c_continue(INITIAL_INV, buf)
}

#[cfg(target_arch = "x86_64")]
pub fn crc32c_continue(initial: u32, buf: &[u8]) -> u32 {
    if is_x86_feature_detected!("sse4.2") {
        crc32c_x86_64_sse42(initial, buf)
    } else {
        // SSE4.2 not supported, fall back to software implementation
        crc32c_sw(initial, buf)
    }
}

#[cfg(not(target_arch = "x86_64"))]
pub fn crc32c_continue(initial: u32, buf: &[u8]) -> u32 {
    crc32c_sw(initial, buf)
}

fn crc32c_sw(initial: u32, buf: &[u8]) -> u32 {
    let mut crc: u32 = initial;
    for i in 0..buf.len() {
        let lookup_index = ((crc & BIT_MASK_8 as u32) as u8 ^ buf[i]) as usize;
        crc = (crc >> BITS_PER_BYTE) ^ LOOKUP_TABLE[lookup_index];
    }
    
    let crc_final = !crc;
    crc_final
}

fn crc32c_x86_64_sse42(initial: u32, buf: &[u8]) -> u32 {
    const DATA_BLOCK_LEN: usize = 8;

    let mut crc: u32 = initial;
    for i in 0..(buf.len() / DATA_BLOCK_LEN) {
        unsafe {
            crc = _mm_crc32_u64(
                crc as u64,
                *(buf[(i * 8)..((i + 1) * 8)].as_ptr() as *const u64)
            ) as u32;
        }
    }
    for i in 0..(buf.len() % DATA_BLOCK_LEN) {
        unsafe {
            crc = _mm_crc32_u8(crc, buf[buf.len() - (buf.len() % 8) + i]);
        }
    }

    let crc_final = !crc;
    crc_final
}

const fn compute_lookup_table() -> [u32; LOOKUP_TABLE_SIZE] {
    let mut table = [0; LOOKUP_TABLE_SIZE];

    let mut i: u16 = 0;
    loop {
        let mut crc = i as u32;
        let mut j: u8 = 0;
        loop {
            crc = if (crc & 1) != 0 {
                (crc >> 1) ^ CRC_POLY_REV
            } else {
                crc >> 1
            };

            j += 1;
            if j == BITS_PER_BYTE {
                break;
            }
        }
        table[i as usize] = crc;

        i += 1;
        if i == LOOKUP_TABLE_SIZE as u16 {
            break;
        }
    }

    table
}
