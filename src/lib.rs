#![feature(portable_simd)]

use std::simd::{Simd, SimdPartialEq};

pub struct FastSplit<'a> {
    s: &'a [u8],
    c: u8,
}

impl<'a> FastSplit<'a> {
    pub fn new(s: &'a [u8], c: u8) -> Self {
        Self { s, c }
    }
}

impl<'a> Iterator for FastSplit<'a> {
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

fn segment_len(s: &[u8], splt: u8) -> usize {
    const N: usize = 64;

    let totlen = s.len();

    let (h, s, t) = s.as_simd::<N>();

    if let Some((l, _)) =
        h.iter().enumerate().filter(|(_, &c)| c == splt).next()
    {
        l
    } else if let Some((l, s)) = s
        .iter()
        .enumerate()
        .filter(|(_, s)| s.simd_eq(Simd::splat(splt)).any())
        .next()
    {
        h.len()
            + l * N
            + unsafe {
                s.as_array()
                    .iter()
                    .enumerate()
                    .filter(|(_, &c)| c == splt)
                    .next()
                    .unwrap_unchecked()
                    .0
            }
    } else if let Some((l, _)) =
        t.iter().enumerate().filter(|(_, &c)| c == splt).next()
    {
        h.len() + s.len() * N + l
    } else {
        totlen
    }
}

#[cfg(test)]
mod tests {
    use std::str::from_utf8;

    use crate::FastSplit;

    #[test]
    fn it_works() {
        let s = "314159265 1234578976543234567 12352352";
        let s = s.as_bytes();

        for s in FastSplit::new(s, b' ') {
            let s = from_utf8(s).unwrap();

            println!("{s}");
        }
    }
}
