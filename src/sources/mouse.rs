//! Mouse movement, via xterm mouse reporting. Terminal cells are coarse and
//! successive positions are heavily correlated, so a move event is worth far
//! less than it looks: we credit 1 bit each, and most of the real value comes
//! from the timing deltas that ride along with every event.

use std::io::{self, Write};
use std::time::{Duration, Instant};

use crossterm::event::{self, Event, KeyCode, KeyEventKind, KeyModifiers, MouseEventKind};

use crate::pool::{EntropyPool, Source};
use crate::term::{MouseCapture, RawMode};

const BITS_PER_EVENT: f64 = 1.0;
const POLL: Duration = Duration::from_millis(100);

pub fn collect(pool: &mut EntropyPool) -> io::Result<()> {
    println!();
    println!("  Mouse movement  ({BITS_PER_EVENT} bit credited per event)");
    println!();
    println!("  Move the mouse around over this terminal window — scribble,");
    println!("  don't trace straight lines. Position and timing are absorbed.");
    println!("  Press Esc when you're done.");
    println!();
    println!("  (If nothing counts up, your terminal doesn't report mouse events.)");
    println!();

    let _raw = RawMode::enter()?;
    let _mouse = MouseCapture::enter()?;
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
        match ev {
            Event::Mouse(m) => {
                let gap = last.elapsed().as_nanos();
                last = Instant::now();

                let mut buf = Vec::with_capacity(32);
                buf.extend_from_slice(&m.column.to_le_bytes());
                buf.extend_from_slice(&m.row.to_le_bytes());
                buf.push(kind_tag(m.kind));
                buf.push(m.modifiers.bits());
                buf.extend_from_slice(&gap.to_le_bytes());
                pool.mix(Source::Mouse, &buf, BITS_PER_EVENT);

                count += 1;
                write!(
                    out,
                    "\r  {count} events   {:.0} bits from mouse   {:.0} bits total   ",
                    pool.bits(Source::Mouse),
                    pool.total_bits(),
                )?;
                out.flush()?;
            }
            Event::Key(key) => {
                if key.kind != KeyEventKind::Press && key.kind != KeyEventKind::Repeat {
                    continue;
                }
                if key.code == KeyCode::Esc
                    || (key.code == KeyCode::Char('c')
                        && key.modifiers.contains(KeyModifiers::CONTROL))
                {
                    break;
                }
            }
            _ => {}
        }
    }

    write!(out, "\r\n")?;
    out.flush()?;

    if count == 0 {
        println!("  No mouse events were received — this terminal may not support them.");
        println!("  Use dice, keyboard or files instead.");
    }
    Ok(())
}

fn kind_tag(kind: MouseEventKind) -> u8 {
    use crossterm::event::MouseButton::*;
    match kind {
        MouseEventKind::Moved => 0,
        MouseEventKind::Down(Left) => 1,
        MouseEventKind::Down(Right) => 2,
        MouseEventKind::Down(Middle) => 3,
        MouseEventKind::Up(Left) => 4,
        MouseEventKind::Up(Right) => 5,
        MouseEventKind::Up(Middle) => 6,
        MouseEventKind::Drag(Left) => 7,
        MouseEventKind::Drag(Right) => 8,
        MouseEventKind::Drag(Middle) => 9,
        MouseEventKind::ScrollDown => 10,
        MouseEventKind::ScrollUp => 11,
        MouseEventKind::ScrollLeft => 12,
        MouseEventKind::ScrollRight => 13,
    }
}
