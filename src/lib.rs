#![allow(incomplete_features)]
#![feature(portable_simd, slice_as_chunks, generic_const_exprs)]

use std::{
    mem::transmute,
    simd::{Simd, SimdPartialEq},
};

pub struct FastSplitIter<'a> {
    s: &'a [u8],
    c: u8,
}

impl<'a> FastSplitIter<'a> {
    pub fn new(s: &'a [u8], c: u8) -> Self {
        Self { s, c }
    }
}

impl<'a> Iterator for FastSplitIter<'a> {
    type Item = &'a [u8];

    fn next(&mut self) -> Option<Self::Item> {
        if self.s.len() == 0 {
            None
        } else {
            let l = segment_len(self.s, self.c);

            let (h, t) = self.s.split_at(l);

            self.s = if t.len() != 0 { &t[1..] } else { t };

            Some(h)
        }
    }
}

#[inline(always)]
fn segment_len_2(s: Simd<u8, 2>, c: u8) -> Option<usize> {
    s.simd_eq(Simd::splat(c)).any().then(|| {
        let [h, _] = s.to_array();

        if h == c {
            0
        } else {
            1
        }
    })
}

#[inline(always)]
fn segment_len_4(s: Simd<u8, 4>, c: u8) -> Option<usize> {
    s.simd_eq(Simd::splat(c)).any().then(|| {
        let [h, t]: [Simd<u8, 2>; 2] = unsafe { transmute(s) };

        unsafe {
            segment_len_2(h, c)
                .or_else(|| Some(2 + segment_len_2(t, c).unwrap_unchecked()))
                .unwrap_unchecked()
        }
    })
}

#[inline(always)]
fn segment_len_8(s: Simd<u8, 8>, c: u8) -> Option<usize> {
    s.simd_eq(Simd::splat(c)).any().then(|| {
        let [h, t]: [Simd<u8, 4>; 2] = unsafe { transmute(s) };

        unsafe {
            segment_len_4(h, c)
                .or_else(|| Some(4 + segment_len_4(t, c).unwrap_unchecked()))
                .unwrap_unchecked()
        }
    })
}

#[inline(always)]
fn segment_len_16(s: Simd<u8, 16>, c: u8) -> Option<usize> {
    s.simd_eq(Simd::splat(c)).any().then(|| {
        let [h, t]: [Simd<u8, 8>; 2] = unsafe { transmute(s) };

        unsafe {
            segment_len_8(h, c)
                .or_else(|| Some(8 + segment_len_8(t, c).unwrap_unchecked()))
                .unwrap_unchecked()
        }
    })
}

#[inline(always)]
fn segment_len_32(s: Simd<u8, 32>, c: u8) -> Option<usize> {
    s.simd_eq(Simd::splat(c)).any().then(|| {
        let [h, t]: [Simd<u8, 16>; 2] = unsafe { transmute(s) };

        unsafe {
            segment_len_16(h, c)
                .or_else(|| Some(16 + segment_len_16(t, c).unwrap_unchecked()))
                .unwrap_unchecked()
        }
    })
}

#[inline(always)]
fn segment_len_64(s: Simd<u8, 64>, c: u8) -> Option<usize> {
    s.simd_eq(Simd::splat(c)).any().then(|| {
        let [h, t]: [Simd<u8, 32>; 2] = unsafe { transmute(s) };

        unsafe {
            segment_len_32(h, c)
                .or_else(|| Some(32 + segment_len_32(t, c).unwrap_unchecked()))
                .unwrap_unchecked()
        }
    })
}

fn segment_len(s: &[u8], splt: u8) -> usize {
    const N: usize = 64;

    let totlen = s.len();

    let (h, s, t) = s.as_simd::<N>();

    if let Some((l, _)) =
        h.iter().enumerate().filter(|(_, &c)| c == splt).next()
    {
        l
    } else if let Some(l) = s
        .iter()
        .enumerate()
        .filter_map(|(i, s)| {
            segment_len_64(*s, splt).and_then(|l| Some(l + i * N))
        })
        .next()
    {
        l
    } else if let Some((l, _)) =
        t.iter().enumerate().filter(|(_, &c)| c == splt).next()
    {
        h.len() + s.len() * N + l
    } else {
        totlen
    }
}

pub trait FastSplit {
    fn fast_split(&self, c: u8) -> FastSplitIter;
}

impl FastSplit for &[u8] {
    fn fast_split(&self, c: u8) -> FastSplitIter {
        FastSplitIter::new(self, c)
    }
}

#[cfg(test)]
mod tests {
    use std::str::from_utf8;

    use crate::FastSplit;

    #[test]
    fn it_works() {
        let s = "314159265,1234578976543234567,12352352";
        let s = s.as_bytes();

        for s in s.fast_split(b',') {
            let s = from_utf8(s).unwrap();

            println!("s: {s}");
        }
    }
}
