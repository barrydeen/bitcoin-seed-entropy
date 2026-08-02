//! Keyboard mashing. Two things are harvested per keystroke: which key it was,
//! and the nanosecond gap since the previous one. Human inter-keystroke timing
//! carries several bits of jitter on its own, but we credit only 3 bits per
//! press in total to stay well under any plausible true value.

use std::io::{self, Write};
use std::time::{Duration, Instant};

use crossterm::event::{self, Event, KeyCode, KeyEventKind, KeyModifiers};

use crate::pool::{EntropyPool, Source};
use crate::term::RawMode;

const BITS_PER_PRESS: f64 = 3.0;
const POLL: Duration = Duration::from_millis(100);

pub fn collect(pool: &mut EntropyPool) -> io::Result<()> {
    println!();
    println!("  Keyboard mashing  ({BITS_PER_PRESS} bits credited per keystroke)");
    println!();
    println!("  Type unpredictably — vary the keys and vary the rhythm.");
    println!("  Both the characters and the timing between them are absorbed.");
    println!("  Press Esc when you're done.");
    println!();

    let _raw = RawMode::enter()?;
    let mut out = io::stdout();
    let mut last = Instant::now();
    let mut count: u64 = 0;

    loop {
        // Poll rather than block, so a closed or silent stdin can never wedge
        // the program with the terminal left in raw mode.
        if !event::poll(POLL)? {
            continue;
        }
        let ev = match event::read() {
            Ok(ev) => ev,
            Err(_) => break,
        };
        let Event::Key(key) = ev else { continue };
        if key.kind != KeyEventKind::Press && key.kind != KeyEventKind::Repeat {
            continue;
        }
        if key.code == KeyCode::Esc
            || (key.code == KeyCode::Char('c') && key.modifiers.contains(KeyModifiers::CONTROL))
        {
            break;
        }

        let gap = last.elapsed().as_nanos();
        last = Instant::now();

        let mut buf = Vec::with_capacity(32);
        buf.extend_from_slice(format!("{:?}", key.code).as_bytes());
        buf.push(key.modifiers.bits());
        buf.extend_from_slice(&gap.to_le_bytes());
        pool.mix(Source::Keyboard, &buf, BITS_PER_PRESS);

        count += 1;
        write!(
            out,
            "\r  {count} keystrokes   {:.0} bits from keyboard   {:.0} bits total   ",
            pool.bits(Source::Keyboard),
            pool.total_bits(),
        )?;
        out.flush()?;
    }

    write!(out, "\r\n")?;
    out.flush()?;
    Ok(())
}
