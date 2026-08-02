//! Physical dice: the only source whose entropy the user can reason about
//! directly. A fair d6 yields log2(6) = 2.585 bits; we credit 2.5 to leave
//! headroom for imperfect dice and imperfect throws.

use std::io;

use crate::pool::{EntropyPool, Source};
use crate::term::prompt_line;

const BITS_PER_DIE: f64 = 2.5;

pub fn collect(pool: &mut EntropyPool) -> io::Result<()> {
    println!();
    println!("  Dice rolls  ({BITS_PER_DIE} bits credited per die)");
    println!();
    println!("  Roll a physical die and type the results as digits 1-6.");
    println!("  You can batch them, e.g.  41526314  — spaces are ignored.");
    println!("  Press Enter on an empty line when you're done.");
    println!();

    loop {
        let Some(line) = prompt_line("  dice> ")? else {
            return Ok(());
        };
        if line.is_empty() {
            return Ok(());
        }

        let cleaned: String = line.chars().filter(|c| !c.is_whitespace()).collect();
        if let Some(bad) = cleaned.chars().find(|c| !matches!(c, '1'..='6')) {
            println!("  ! '{bad}' is not a die face — only the digits 1-6 are accepted.");
            continue;
        }

        for c in cleaned.chars() {
            pool.mix(Source::Dice, &[c as u8], BITS_PER_DIE);
        }

        println!(
            "  + {} rolls  ({:.0} bits from dice, {:.0} bits total)",
            cleaned.len(),
            pool.bits(Source::Dice),
            pool.total_bits(),
        );
    }
}
