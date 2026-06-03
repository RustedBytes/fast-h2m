//! Portable SIMD byte-scanning helpers.
//!
//! This module is compiled only behind the `simd` feature. `std::simd` is still
//! nightly-only, so callers must keep scalar fallbacks outside this module.

use std::simd::Simd;
use std::simd::cmp::{SimdPartialEq, SimdPartialOrd};

const LANES: usize = 32;

#[inline]
pub(crate) fn find_byte(bytes: &[u8], needle: u8) -> Option<usize> {
    let needle_byte = needle;
    let needle = Simd::<u8, LANES>::splat(needle_byte);
    let mut offset = 0usize;
    let mut chunks = bytes.chunks_exact(LANES);

    for chunk in &mut chunks {
        let vector = Simd::<u8, LANES>::from_slice(chunk);
        let mask = vector.simd_eq(needle).to_bitmask();
        if mask != 0 {
            return Some(offset + mask.trailing_zeros() as usize);
        }
        offset += LANES;
    }

    chunks
        .remainder()
        .iter()
        .position(|&byte| byte == needle_byte)
        .map(|pos| offset + pos)
}

#[inline]
pub(crate) fn find_any2(bytes: &[u8], a: u8, b: u8) -> Option<usize> {
    let a_byte = a;
    let b_byte = b;
    let a = Simd::<u8, LANES>::splat(a_byte);
    let b = Simd::<u8, LANES>::splat(b_byte);
    let mut offset = 0usize;
    let mut chunks = bytes.chunks_exact(LANES);

    for chunk in &mut chunks {
        let vector = Simd::<u8, LANES>::from_slice(chunk);
        let mask = (vector.simd_eq(a) | vector.simd_eq(b)).to_bitmask();
        if mask != 0 {
            return Some(offset + mask.trailing_zeros() as usize);
        }
        offset += LANES;
    }

    chunks
        .remainder()
        .iter()
        .position(|&byte| byte == a_byte || byte == b_byte)
        .map(|pos| offset + pos)
}

#[inline]
pub(crate) fn find_any3(bytes: &[u8], a: u8, b: u8, c: u8) -> Option<usize> {
    let a_byte = a;
    let b_byte = b;
    let c_byte = c;
    let a = Simd::<u8, LANES>::splat(a_byte);
    let b = Simd::<u8, LANES>::splat(b_byte);
    let c = Simd::<u8, LANES>::splat(c_byte);
    let mut offset = 0usize;
    let mut chunks = bytes.chunks_exact(LANES);

    for chunk in &mut chunks {
        let vector = Simd::<u8, LANES>::from_slice(chunk);
        let mask = (vector.simd_eq(a) | vector.simd_eq(b) | vector.simd_eq(c)).to_bitmask();
        if mask != 0 {
            return Some(offset + mask.trailing_zeros() as usize);
        }
        offset += LANES;
    }

    chunks
        .remainder()
        .iter()
        .position(|&byte| byte == a_byte || byte == b_byte || byte == c_byte)
        .map(|pos| offset + pos)
}

#[inline]
pub(crate) fn contains_ascii_punctuation(bytes: &[u8]) -> bool {
    let excl = Simd::<u8, LANES>::splat(b'!');
    let slash = Simd::<u8, LANES>::splat(b'/');
    let colon = Simd::<u8, LANES>::splat(b':');
    let at = Simd::<u8, LANES>::splat(b'@');
    let open_bracket = Simd::<u8, LANES>::splat(b'[');
    let backtick = Simd::<u8, LANES>::splat(b'`');
    let open_brace = Simd::<u8, LANES>::splat(b'{');
    let tilde = Simd::<u8, LANES>::splat(b'~');
    let mut chunks = bytes.chunks_exact(LANES);

    for chunk in &mut chunks {
        let vector = Simd::<u8, LANES>::from_slice(chunk);
        let punctuation = (vector.simd_ge(excl) & vector.simd_le(slash))
            | (vector.simd_ge(colon) & vector.simd_le(at))
            | (vector.simd_ge(open_bracket) & vector.simd_le(backtick))
            | (vector.simd_ge(open_brace) & vector.simd_le(tilde));
        if punctuation.any() {
            return true;
        }
    }

    chunks
        .remainder()
        .iter()
        .any(|byte| byte.is_ascii_punctuation())
}

#[inline]
pub(crate) fn contains_misc_markdown(bytes: &[u8]) -> bool {
    let needles = [
        b'\\', b'&', b'<', b'`', b'[', b']', b'>', b'~', b'#', b'=', b'+', b'|', b'-',
    ];
    let mut chunks = bytes.chunks_exact(LANES);

    for chunk in &mut chunks {
        let vector = Simd::<u8, LANES>::from_slice(chunk);
        let mut mask = vector.simd_eq(Simd::<u8, LANES>::splat(needles[0]));
        for &needle in &needles[1..] {
            mask |= vector.simd_eq(Simd::<u8, LANES>::splat(needle));
        }
        if mask.any() {
            return true;
        }
    }

    chunks.remainder().iter().any(|byte| needles.contains(byte))
}

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
        ]
    }

    #[test]
    fn find_byte_matches_scalar() {
        for bytes in cases() {
            for needle in [b'<', b'>', b'?', b'z'] {
                assert_eq!(
                    find_byte(&bytes, needle),
                    bytes.iter().position(|&byte| byte == needle)
                );
            }
        }
    }

    #[test]
    fn find_any2_matches_scalar() {
        for bytes in cases() {
            assert_eq!(
                find_any2(&bytes, b'<', b'>'),
                bytes.iter().position(|&byte| byte == b'<' || byte == b'>')
            );
        }
    }

    #[test]
    fn find_any3_matches_scalar() {
        for bytes in cases() {
            assert_eq!(
                find_any3(&bytes, b'<', b'>', b'"'),
                bytes
                    .iter()
                    .position(|&byte| byte == b'<' || byte == b'>' || byte == b'"')
            );
        }
    }

    #[test]
    fn contains_helpers_match_scalar() {
        for bytes in cases() {
            assert_eq!(
                contains_ascii_punctuation(&bytes),
                bytes.iter().any(|byte| byte.is_ascii_punctuation())
            );
            assert_eq!(
                contains_misc_markdown(&bytes),
                bytes.iter().any(|byte| {
                    matches!(
                        byte,
                        b'\\'
                            | b'&'
                            | b'<'
                            | b'`'
                            | b'['
                            | b']'
                            | b'>'
                            | b'~'
                            | b'#'
                            | b'='
                            | b'+'
                            | b'|'
                            | b'-'
                    )
                })
            );
        }
    }
}
