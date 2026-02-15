// Resurrected from `rand` 0.9.2, with several modifications; see
// https://github.com/rust-random/rand/blob/0.9.2/src/rngs/mock.rs.

// ---

// Copyright 2018 Developers of the Rand project.
//
// Licensed under the Apache License, Version 2.0 <LICENSE-APACHE or
// https://www.apache.org/licenses/LICENSE-2.0> or the MIT license
// <LICENSE-MIT or https://opensource.org/licenses/MIT>, at your
// option. This file may not be copied, modified, or distributed
// except according to those terms.

use rand_core::{utils, TryRng};

/// A mock generator yielding very predictable output.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct StepRng {
    v: u64,
    a: u64,
}

impl StepRng {
    /// Create a `StepRng`, yielding an arithmetic sequence starting with
    /// `initial` and incremented by `increment` each time.
    pub fn new(initial: u64, increment: u64) -> Self {
        StepRng { v: initial, a: increment }
    }
}

impl TryRng for StepRng {
    type Error = std::convert::Infallible;

    #[inline]
    fn try_next_u32(&mut self) -> Result<u32, Self::Error> {
        self.try_next_u64().map(|n| n as u32)
    }

    #[inline]
    fn try_next_u64(&mut self) -> Result<u64, Self::Error> {
        let res = self.v;
        self.v = self.v.wrapping_add(self.a);
        Ok(res)
    }

    #[inline]
    fn try_fill_bytes(&mut self, dst: &mut [u8]) -> Result<(), Self::Error> {
        utils::fill_bytes_via_next_word(dst, || self.try_next_u64());
        Ok(())
    }
}
