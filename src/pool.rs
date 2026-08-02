//! The entropy pool: a SHA-512 accumulator plus conservative bit accounting.
//!
//! Every raw input event from every source is streamed into one hasher with a
//! domain-separating source tag, a length prefix and a monotonic timestamp, so
//! no two event streams can be confused for one another. Bit *credits* are only
//! a UI-facing estimate of how much unpredictability the user has supplied; the
//! cryptographic strength of the result never depends on them being accurate.

use std::collections::BTreeSet;
use std::fmt;
use std::time::Instant;

use sha2::{Digest, Sha256, Sha512};
use zeroize::Zeroizing;

/// How much credited entropy we require before allowing a phrase to be generated.
pub const TARGET_BITS: f64 = 1024.0;

#[derive(Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Debug)]
pub enum Source {
    Dice,
    Keyboard,
    Mouse,
    Files,
}

impl Source {
    /// Domain-separation tag mixed in ahead of every event from this source.
    fn tag(self) -> u8 {
        match self {
            Source::Dice => 1,
            Source::Keyboard => 2,
            Source::Mouse => 3,
            Source::Files => 4,
        }
    }

    pub const ALL: [Source; 4] = [Source::Dice, Source::Keyboard, Source::Mouse, Source::Files];
}

impl fmt::Display for Source {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let name = match self {
            Source::Dice => "Dice rolls",
            Source::Keyboard => "Keyboard mashing",
            Source::Mouse => "Mouse movement",
            Source::Files => "Files / images",
        };
        f.write_str(name)
    }
}

pub struct EntropyPool {
    hasher: Sha512,
    started: Instant,
    credits: [f64; 4],
    events: [u64; 4],
    /// SHA-512 digests of files already absorbed, so re-adding one earns nothing.
    seen_files: BTreeSet<[u8; 64]>,
}

impl Default for EntropyPool {
    fn default() -> Self {
        Self::new()
    }
}

impl EntropyPool {
    pub fn new() -> Self {
        let mut hasher = Sha512::new();
        hasher.update(b"seed-entropy/v1/pool");
        EntropyPool {
            hasher,
            started: Instant::now(),
            credits: [0.0; 4],
            events: [0; 4],
            seen_files: BTreeSet::new(),
        }
    }

    /// Absorb one event. `bits` is the conservative credit for it.
    pub fn mix(&mut self, source: Source, data: &[u8], bits: f64) {
        self.absorb(source, data);
        let idx = source as usize;
        self.credits[idx] += bits;
        self.events[idx] += 1;
    }

    /// Absorb data without crediting any entropy (used for bulk file contents,
    /// which are credited once per file rather than once per chunk).
    pub fn absorb(&mut self, source: Source, data: &[u8]) {
        let nanos = self.started.elapsed().as_nanos();
        self.hasher.update([source.tag()]);
        self.hasher.update((data.len() as u64).to_le_bytes());
        self.hasher.update(data);
        self.hasher.update(nanos.to_le_bytes());
    }

    /// Credit an event whose data was streamed in separately via `absorb`
    /// (file contents arrive as many chunks but count as one event).
    pub fn credit(&mut self, source: Source, bits: f64) {
        self.credits[source as usize] += bits;
        self.events[source as usize] += 1;
    }

    pub fn bits(&self, source: Source) -> f64 {
        self.credits[source as usize]
    }

    pub fn events(&self, source: Source) -> u64 {
        self.events[source as usize]
    }

    pub fn total_bits(&self) -> f64 {
        self.credits.iter().sum()
    }

    pub fn is_ready(&self) -> bool {
        self.total_bits() >= TARGET_BITS
    }

    /// Returns true if this file's digest is new (and records it).
    pub fn note_file(&mut self, digest: [u8; 64]) -> bool {
        self.seen_files.insert(digest)
    }

    /// Derive the final 256 bits of BIP39 entropy.
    ///
    /// The pool digest is combined with fresh bytes from the OS CSPRNG, so the
    /// output is at least as unpredictable as the stronger of the two — a user
    /// source that turns out to be weaker than credited cannot degrade it, and
    /// a compromised kernel RNG cannot either.
    pub fn finalize(self) -> std::io::Result<Zeroizing<[u8; 32]>> {
        let mut pool = Zeroizing::new([0u8; 64]);
        pool.copy_from_slice(&self.hasher.finalize());

        let mut os_bytes = Zeroizing::new([0u8; 32]);
        getrandom::fill(os_bytes.as_mut_slice())
            .map_err(|e| std::io::Error::other(format!("OS CSPRNG unavailable: {e}")))?;

        let mut h = Sha256::new();
        h.update(b"seed-entropy/v1/final");
        h.update(pool.as_slice());
        h.update(os_bytes.as_slice());

        let mut out = Zeroizing::new([0u8; 32]);
        out.copy_from_slice(&h.finalize());
        Ok(out)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn credits_accumulate_per_source() {
        let mut pool = EntropyPool::new();
        pool.mix(Source::Dice, b"4", 2.5);
        pool.mix(Source::Dice, b"6", 2.5);
        pool.mix(Source::Mouse, b"xy", 1.0);

        assert_eq!(pool.bits(Source::Dice), 5.0);
        assert_eq!(pool.bits(Source::Mouse), 1.0);
        assert_eq!(pool.events(Source::Dice), 2);
        assert_eq!(pool.total_bits(), 6.0);
        assert!(!pool.is_ready());
    }

    #[test]
    fn reaches_ready_at_target() {
        let mut pool = EntropyPool::new();
        for _ in 0..410 {
            pool.mix(Source::Dice, b"3", 2.5);
        }
        assert!(pool.is_ready());
    }

    #[test]
    fn duplicate_files_are_only_counted_once() {
        let mut pool = EntropyPool::new();
        let digest = [7u8; 64];
        assert!(pool.note_file(digest));
        assert!(!pool.note_file(digest));
    }

    #[test]
    fn distinct_inputs_give_distinct_output() {
        let mut a = EntropyPool::new();
        a.mix(Source::Dice, b"111111", 15.0);
        let mut b = EntropyPool::new();
        b.mix(Source::Dice, b"222222", 15.0);

        let out_a = a.finalize().unwrap();
        let out_b = b.finalize().unwrap();
        assert_ne!(out_a.as_slice(), out_b.as_slice());
        assert_eq!(out_a.len(), 32);
    }
}
