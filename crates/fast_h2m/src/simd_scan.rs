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
