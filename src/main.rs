//! seed-entropy — build a 24-word BIP39 seed phrase from user-generated,
//! offline entropy.
//!
//! Nothing is written to disk, nothing touches the network, and the phrase is
//! only ever printed to a terminal. Run this on an air-gapped machine.

mod pool;
mod sources;
mod term;

use std::io::{self, IsTerminal, Write};

use bip39::Mnemonic;
use zeroize::Zeroize;

use pool::{EntropyPool, Source, TARGET_BITS};
use term::{clear_all, prompt_line};

fn main() {
    if !io::stdout().is_terminal() {
        eprintln!(
            "seed-entropy refuses to run when output is not a terminal — a seed \
             phrase must never be piped into a file, a pager or a log."
        );
        std::process::exit(2);
    }

    if let Err(e) = run() {
        // Restore the terminal before reporting, in case we died mid-capture.
        let _ = crossterm::terminal::disable_raw_mode();
        eprintln!("\nerror: {e}");
        std::process::exit(1);
    }
}

fn run() -> io::Result<()> {
    banner();
    let mut pool = EntropyPool::new();

    loop {
        show_status(&pool);
        let Some(choice) = prompt_line("  choose> ")? else {
            println!("\n  Input closed. No phrase was generated.\n");
            return Ok(());
        };

        match choice.trim().to_ascii_lowercase().as_str() {
            "1" => sources::dice::collect(&mut pool)?,
            "2" => sources::keyboard::collect(&mut pool)?,
            "3" => sources::mouse::collect(&mut pool)?,
            "4" => sources::files::collect(&mut pool)?,
            "g" if pool.is_ready() => {
                generate(pool)?;
                return Ok(());
            }
            "g" => {
                println!(
                    "\n  Not yet — {:.0} more bits needed before generating.\n",
                    TARGET_BITS - pool.total_bits()
                );
            }
            "q" => {
                println!("\n  Quit. No phrase was generated.\n");
                return Ok(());
            }
            "" => {}
            other => println!("\n  '{other}' isn't an option.\n"),
        }
    }
}

fn banner() {
    println!();
    println!("  ┌────────────────────────────────────────────────────────────┐");
    println!("  │  seed-entropy — 24-word BIP39 phrase from offline entropy  │");
    println!("  └────────────────────────────────────────────────────────────┘");
    println!();
    println!("  Every input you provide is absorbed into a SHA-512 pool. Once");
    println!("  {TARGET_BITS:.0} bits have been collected, the pool is combined with the");
    println!("  OS random generator and compressed to the 256 bits that become");
    println!("  your phrase.");
    println!();
    println!("  Before you start: disconnect from the network, and have paper");
    println!("  and a pen ready. The phrase is never saved anywhere.");
    println!();
    println!("  ⚠  This software is UNAUDITED and comes with NO WARRANTY. It has");
    println!("     had no third-party security review. Use it at your own risk,");
    println!("     and never store more than you are willing to lose entirely.");
    println!();
}

fn show_status(pool: &EntropyPool) {
    let total = pool.total_bits();
    let frac = (total / TARGET_BITS).min(1.0);
    let width = 40usize;
    let filled = ((frac * width as f64).round() as usize).min(width);

    println!("  ┌{}┐", "─".repeat(width + 2));
    println!(
        "  │ {}{} │",
        "█".repeat(filled),
        "░".repeat(width - filled),
    );
    println!("  └{}┘", "─".repeat(width + 2));
    println!("    {total:.0} of {TARGET_BITS:.0} bits collected");
    println!();

    for (i, src) in Source::ALL.iter().enumerate() {
        println!(
            "   [{}] {:<18} {:>5.0} bits   {} events",
            i + 1,
            src.to_string(),
            pool.bits(*src),
            pool.events(*src),
        );
    }
    println!();
    if pool.is_ready() {
        println!("   [g] Generate the 24-word phrase   ✓ ready");
    } else {
        println!(
            "   [g] Generate the 24-word phrase   — {:.0} more bits needed",
            TARGET_BITS - total
        );
    }
    println!("   [q] Quit without generating");
    println!();
}

fn generate(pool: EntropyPool) -> io::Result<()> {
    let entropy = pool.finalize()?;
    let mnemonic = Mnemonic::from_entropy(entropy.as_slice())
        .map_err(|e| io::Error::other(format!("BIP39 encoding failed: {e}")))?;

    // Round-trip check: re-parsing the words must reproduce the same entropy.
    // A mismatch means the encoder is wrong — refuse to display a bad phrase.
    let reparsed = Mnemonic::parse(mnemonic.to_string())
        .map_err(|e| io::Error::other(format!("self-check failed to parse: {e}")))?;
    let (round, len) = reparsed.to_entropy_array();
    if len != 32 || round[..32] != entropy[..] {
        return Err(io::Error::other(
            "self-check failed: the phrase does not decode back to the generated entropy",
        ));
    }

    clear_all()?;
    println!();
    println!("  ┌────────────────────────────────────────────────────────────┐");
    println!("  │  Your 24-word seed phrase                                  │");
    println!("  └────────────────────────────────────────────────────────────┘");
    println!();

    let words: Vec<&'static str> = mnemonic.words().collect();
    for row in 0..6 {
        let mut line = String::from("   ");
        for col in 0..4 {
            let i = row + col * 6;
            line.push_str(&format!("{:>2}. {:<12}", i + 1, words[i]));
        }
        println!("{}", line.trim_end());
    }

    println!();
    println!("  Write these down on paper, in order, and check them twice.");
    println!("  Anyone who sees them owns every coin the wallet holds.");
    println!("  They are not stored anywhere — if you lose them, they are gone.");
    println!();

    let _ = prompt_line("  Press Enter once they are written down and verified… ")?;

    // `entropy` is Zeroizing and `Mnemonic` is ZeroizeOnDrop (bip39's "zeroize"
    // feature), so both wipe themselves here. The rendered string is ours.
    let mut phrase = mnemonic.to_string();
    phrase.zeroize();

    clear_all()?;
    println!("  Screen cleared. Keep that paper safe.\n");
    io::stdout().flush()
}

#[cfg(test)]
mod tests {
    use bip39::Mnemonic;

    /// Official BIP39 (Trezor) test vectors for 256-bit entropy.
    const VECTORS: &[(&str, &str)] = &[
        (
            "0000000000000000000000000000000000000000000000000000000000000000",
            "abandon abandon abandon abandon abandon abandon abandon abandon abandon abandon \
             abandon abandon abandon abandon abandon abandon abandon abandon abandon abandon \
             abandon abandon abandon art",
        ),
        (
            "7f7f7f7f7f7f7f7f7f7f7f7f7f7f7f7f7f7f7f7f7f7f7f7f7f7f7f7f7f7f7f7f",
            "legal winner thank year wave sausage worth useful legal winner thank year wave \
             sausage worth useful legal winner thank year wave sausage worth title",
        ),
        (
            "8080808080808080808080808080808080808080808080808080808080808080",
            "letter advice cage absurd amount doctor acoustic avoid letter advice cage absurd \
             amount doctor acoustic avoid letter advice cage absurd amount doctor acoustic bless",
        ),
        (
            "ffffffffffffffffffffffffffffffffffffffffffffffffffffffffffffffff",
            "zoo zoo zoo zoo zoo zoo zoo zoo zoo zoo zoo zoo zoo zoo zoo zoo zoo zoo zoo zoo \
             zoo zoo zoo vote",
        ),
        (
            "9f6a2878b2520799a44ef18bc7df394e7061a224d2c33cd015b157d746869863",
            "panda eyebrow bullet gorilla call smoke muffin taste mesh discover soft ostrich \
             alcohol speed nation flash devote level hobby quick inner drive ghost inside",
        ),
    ];

    fn unhex(s: &str) -> Vec<u8> {
        (0..s.len())
            .step_by(2)
            .map(|i| u8::from_str_radix(&s[i..i + 2], 16).unwrap())
            .collect()
    }

    fn normalize(s: &str) -> String {
        s.split_whitespace().collect::<Vec<_>>().join(" ")
    }

    #[test]
    fn matches_official_bip39_vectors() {
        for (hex, expected) in VECTORS {
            let m = Mnemonic::from_entropy(&unhex(hex)).unwrap();
            assert_eq!(m.to_string(), normalize(expected), "vector {hex}");
            assert_eq!(m.words().count(), 24);
        }
    }

    #[test]
    fn phrases_round_trip_back_to_their_entropy() {
        for (hex, _) in VECTORS {
            let entropy = unhex(hex);
            let m = Mnemonic::from_entropy(&entropy).unwrap();
            let (round, len) = Mnemonic::parse(m.to_string()).unwrap().to_entropy_array();
            assert_eq!(len, 32);
            assert_eq!(&round[..32], &entropy[..]);
        }
    }

    #[test]
    fn pool_output_produces_a_valid_24_word_phrase() {
        use crate::pool::{EntropyPool, Source};
        let mut pool = EntropyPool::new();
        for c in b"415263145362415263145362" {
            pool.mix(Source::Dice, &[*c], 2.5);
        }
        let entropy = pool.finalize().unwrap();
        let m = Mnemonic::from_entropy(entropy.as_slice()).unwrap();
        assert_eq!(m.words().count(), 24);
        assert!(Mnemonic::parse(m.to_string()).is_ok());
    }
}
