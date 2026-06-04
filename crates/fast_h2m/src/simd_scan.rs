//! Portable SIMD byte-scanning helpers.
//!
//! This module is compiled only behind the `simd` feature. `std::simd` is still
//! nightly-only, so callers must keep scalar fallbacks outside this module.

use std::simd::Simd;
use std::simd::cmp::{SimdPartialEq, SimdPartialOrd};

const LANES: usize = 32;

#[inline]
pub(crate) fn count_binary_markers(bytes: &[u8]) -> (usize, usize) {
    let zero = Simd::<u8, LANES>::splat(0);
    let tab = Simd::<u8, LANES>::splat(0x09);
    let control_start = Simd::<u8, LANES>::splat(0x0E);
    let control_end = Simd::<u8, LANES>::splat(0x20);
    let mut control_count = 0usize;
    let mut nul_count = 0usize;
    let mut chunks = bytes.chunks_exact(LANES);

    for chunk in &mut chunks {
        let vector = Simd::<u8, LANES>::from_slice(chunk);
        let nul_mask = vector.simd_eq(zero);
        let control_mask =
            vector.simd_lt(tab) | (vector.simd_ge(control_start) & vector.simd_lt(control_end));

        nul_count += nul_mask.to_bitmask().count_ones() as usize;
        control_count += control_mask.to_bitmask().count_ones() as usize;
    }

    for &byte in chunks.remainder() {
        if byte == 0 {
            nul_count += 1;
        }
        let is_control = (byte < 0x09) || (0x0E..0x20).contains(&byte);
        if is_control {
            control_count += 1;
        }
    }

    (control_count, nul_count)
}

#[inline]
pub(crate) fn contains_byte(bytes: &[u8], needle: u8) -> bool {
    let needle_byte = needle;
    let needle = Simd::<u8, LANES>::splat(needle_byte);
    let mut chunks = bytes.chunks_exact(LANES);

    for chunk in &mut chunks {
        let vector = Simd::<u8, LANES>::from_slice(chunk);
        if vector.simd_eq(needle).any() {
            return true;
        }
    }

    chunks.remainder().contains(&needle_byte)
}

#[inline]
pub(crate) fn contains_any2(bytes: &[u8], first: u8, second: u8) -> bool {
    let first_byte = first;
    let second_byte = second;
    let first = Simd::<u8, LANES>::splat(first_byte);
    let second = Simd::<u8, LANES>::splat(second_byte);
    let mut chunks = bytes.chunks_exact(LANES);

    for chunk in &mut chunks {
        let vector = Simd::<u8, LANES>::from_slice(chunk);
        if (vector.simd_eq(first) | vector.simd_eq(second)).any() {
            return true;
        }
    }

    chunks
        .remainder()
        .iter()
        .any(|byte| *byte == first_byte || *byte == second_byte)
}

#[inline]
pub(crate) fn contains_ascii_whitespace_or_non_ascii(bytes: &[u8]) -> bool {
    let space = Simd::<u8, LANES>::splat(b' ');
    let ascii_limit = Simd::<u8, LANES>::splat(0x80);
    let mut chunks = bytes.chunks_exact(LANES);

    for chunk in &mut chunks {
        let vector = Simd::<u8, LANES>::from_slice(chunk);
        if (vector.simd_le(space) | vector.simd_ge(ascii_limit)).any() {
            return true;
        }
    }

    chunks
        .remainder()
        .iter()
        .any(|byte| *byte <= b' ' || *byte >= 0x80)
}

#[cfg(test)]
mod tests {
    use super::*;

    fn cases() -> Vec<Vec<u8>> {
        vec![
            Vec::new(),
            b"short < slice".to_vec(),
            b"abcdefghijklmnopqrstuvwxyzABCDEF".to_vec(),
            b"abcdefghijklmnopqrstuvwxyzABCDE<".to_vec(),
            b"abcdefghijklmnopqrstuvwxyzABCDEFGHIJKLMNOPQRSTUVWXYZ0123456789<>".to_vec(),
            b"abcdefghijklmnopqrstuvwxyzABCDEFGHIJKLMNOPQRSTUVWXYZ0123456789".to_vec(),
            b"\0\0\0abc\x01\x02\x1F".to_vec(),
            "hello\u{a0}world".as_bytes().to_vec(),
        ]
    }

    #[test]
    fn count_binary_markers_matches_scalar() {
        for bytes in cases() {
            let control = bytes
                .iter()
                .filter(|&&byte| (byte < 0x09) || (0x0E..0x20).contains(&byte))
                .count();
            let nul = bytes.iter().filter(|&&byte| byte == 0).count();
            assert_eq!(count_binary_markers(&bytes), (control, nul));
        }
    }

    #[test]
    fn contains_byte_matches_scalar() {
        for bytes in cases() {
            for needle in [0, b'<', b'>', b'z'] {
                assert_eq!(
                    contains_byte(&bytes, needle),
                    bytes.contains(&needle),
                    "bytes={bytes:?} needle={needle}"
                );
            }
        }
    }

    #[test]
    fn contains_any2_matches_scalar() {
        for bytes in cases() {
            assert_eq!(
                contains_any2(&bytes, b'<', b'>'),
                bytes.iter().any(|byte| matches!(*byte, b'<' | b'>'))
            );
        }
    }

    #[test]
    fn contains_ascii_whitespace_or_non_ascii_matches_scalar() {
        for bytes in cases() {
            assert_eq!(
                contains_ascii_whitespace_or_non_ascii(&bytes),
                bytes.iter().any(|byte| *byte <= b' ' || *byte >= 0x80)
            );
        }
    }
}
