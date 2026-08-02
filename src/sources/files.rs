//! Files and images. A photo straight off a camera carries a lot of sensor
//! noise, but compressed formats and file headers are highly structured, so we
//! credit only 4 bits per KiB and cap each file at 256 bits. Files are
//! deduplicated by digest so adding the same photo twice earns nothing.

use std::fs::{self, File};
use std::io::{self, Read, Write};
use std::path::{Path, PathBuf};

use sha2::{Digest, Sha512};

use crate::pool::{EntropyPool, Source};
use crate::term::prompt_line;

const BITS_PER_KIB: f64 = 4.0;
const MAX_BITS_PER_FILE: f64 = 256.0;
const CHUNK: usize = 64 * 1024;

pub fn collect(pool: &mut EntropyPool) -> io::Result<()> {
    println!();
    println!("  Files / images  ({BITS_PER_KIB} bits per KiB, capped at {MAX_BITS_PER_FILE:.0} bits per file)");
    println!();
    println!("  Enter a path to a file or a directory. Camera photos are ideal;");
    println!("  their sensor noise is genuinely unpredictable. Contents are hashed");
    println!("  in memory and never copied anywhere.");
    println!("  Press Enter on an empty line when you're done.");
    println!();

    loop {
        let Some(line) = prompt_line("  path> ")? else {
            return Ok(());
        };
        if line.is_empty() {
            return Ok(());
        }

        let path = PathBuf::from(expand_tilde(&line));
        let meta = match fs::metadata(&path) {
            Ok(m) => m,
            Err(e) => {
                println!("  ! {}: {e}", path.display());
                continue;
            }
        };

        let files: Vec<PathBuf> = if meta.is_dir() {
            match list_dir(&path) {
                Ok(v) if v.is_empty() => {
                    println!("  ! {} contains no regular files.", path.display());
                    continue;
                }
                Ok(v) => v,
                Err(e) => {
                    println!("  ! {}: {e}", path.display());
                    continue;
                }
            }
        } else {
            vec![path]
        };

        let mut added = 0usize;
        let mut gained = 0.0f64;
        for f in &files {
            match absorb_file(pool, f) {
                Ok(Some(bits)) => {
                    added += 1;
                    gained += bits;
                }
                Ok(None) => println!("  · {} — already absorbed, skipped.", display_name(f)),
                Err(e) => println!("  ! {}: {e}", display_name(f)),
            }
        }

        if added > 0 {
            println!(
                "  + {added} file(s), {gained:.0} bits  ({:.0} bits from files, {:.0} bits total)",
                pool.bits(Source::Files),
                pool.total_bits(),
            );
        }
    }
}

/// Streams a file into the pool. Returns the bits credited, or `None` if this
/// exact file has already been absorbed.
fn absorb_file(pool: &mut EntropyPool, path: &Path) -> io::Result<Option<f64>> {
    let mut file = File::open(path)?;
    let mut buf = vec![0u8; CHUNK];
    let mut digest = Sha512::new();
    let mut len: u64 = 0;

    print!("  reading {}… ", display_name(path));
    io::stdout().flush()?;

    loop {
        let n = file.read(&mut buf)?;
        if n == 0 {
            break;
        }
        digest.update(&buf[..n]);
        pool.absorb(Source::Files, &buf[..n]);
        len += n as u64;
    }

    let mut d = [0u8; 64];
    d.copy_from_slice(&digest.finalize());
    if !pool.note_file(d) {
        println!();
        return Ok(None);
    }

    let bits = ((len as f64 / 1024.0) * BITS_PER_KIB).min(MAX_BITS_PER_FILE);
    pool.credit(Source::Files, bits);
    println!("{} bytes, {bits:.0} bits", len);
    Ok(Some(bits))
}

fn list_dir(dir: &Path) -> io::Result<Vec<PathBuf>> {
    let mut out = Vec::new();
    for entry in fs::read_dir(dir)? {
        let entry = entry?;
        if entry.file_type()?.is_file() {
            out.push(entry.path());
        }
    }
    out.sort();
    Ok(out)
}

fn display_name(path: &Path) -> String {
    path.file_name()
        .map(|n| n.to_string_lossy().into_owned())
        .unwrap_or_else(|| path.display().to_string())
}

fn expand_tilde(input: &str) -> String {
    let unquoted = input.trim_matches(|c| c == '"' || c == '\'');
    if let Some(rest) = unquoted.strip_prefix("~/")
        && let Ok(home) = std::env::var("HOME")
    {
        return format!("{home}/{rest}");
    }
    unquoted.to_string()
}
