# seed-entropy

**Build a 24-word Bitcoin seed phrase from entropy you generate yourself, offline.**

Most wallets hand you 24 words and ask you to trust their random number
generator. This tool lets you supply the randomness — physical dice, keyboard
timing, mouse movement, your own photographs — and mixes it with your operating
system's generator so the result is at least as strong as whichever source is
better.

It requires **1024 bits** of collected entropy before it will produce a phrase.
That is four times the 256 bits the phrase actually carries, which is deliberate:
the surplus is margin against the possibility that your inputs are less random
than they look.

```bash
git clone https://github.com/barrydeen/bitcoin-seed-entropy
cd bitcoin-seed-entropy
cargo build --release
./target/release/seed-entropy
```

> [!CAUTION]
> **This software has never been audited by anyone. Use it entirely at your own
> risk.**
>
> - **No third-party security audit has been performed.** None. Not by a security
>   firm, not by an independent cryptographer, not by anyone other than the
>   author. Do not mistake a confident README for a reviewed codebase.
> - **No warranty of any kind is provided.** See the [MIT License](LICENSE) and
>   the [Disclaimer](#disclaimer). If this software loses your coins, you have no
>   recourse against anyone.
> - **This is young software with a small number of users.** Bugs that only show
>   up on some platform, some terminal, or some edge case may well still be in
>   here.
> - **It generates real Bitcoin keys.** Anyone who sees the 24 words can take
>   every coin the wallet holds, permanently and irreversibly. If you lose the
>   words, nobody can recover them for you — not the author, not an exchange, not
>   anyone.
> - **Never put more into a wallet than you are willing to lose entirely**, and
>   only after you have tested a restore from your paper backup.
>
> Read **[Using it safely](#using-it-safely)** in full before generating a phrase
> you intend to actually use. If you are not prepared to review the source
> yourself, or to have someone you trust review it, use a hardware wallet instead.

---

## Contents

- [Should you use this?](#should-you-use-this)
- [Using it safely](#using-it-safely)
  - [1. Prepare the machine](#1-prepare-the-machine)
  - [2. Check your terminal isn't recording you](#2-check-your-terminal-isnt-recording-you)
  - [3. Collect entropy well](#3-collect-entropy-well)
  - [4. Record the phrase](#4-record-the-phrase)
  - [5. Verify before you fund it](#5-verify-before-you-fund-it)
  - [6. Clean up](#6-clean-up)
- [Entropy sources](#entropy-sources)
- [How the key is derived](#how-the-key-is-derived)
- [Threat model](#threat-model)
- [Auditing the code](#auditing-the-code)
- [Building and testing](#building-and-testing)
- [FAQ](#faq)
- [Disclaimer](#disclaimer)
- [License](#license)

---

## Should you use this?

**A hardware wallet is the right default for most people.** A Coldcard, Trezor or
Ledger generates keys inside a chip built to resist physical extraction, and the
words never touch a general-purpose computer at all. That is a stronger position
than any software running on a laptop, this tool included.

Use this tool if you have a specific reason to distrust or supplement a device
RNG, if you want entropy you can personally verify (dice you can watch), or if
you want to generate a phrase with no proprietary firmware in the path. Several
hardware wallets also accept dice entropy directly — if yours does, that is an
excellent option and this tool is an alternative to it, not a replacement.

Do **not** use this if you are going to run it on your everyday laptop while
connected to the internet. That undoes essentially all of its value.

---

## Using it safely

The tool's cryptography is the easy part. Everything that actually goes wrong
with seed phrases happens in the environment around it: a screenshot, a
synchronised clipboard, a photo in a cloud backup, a terminal that saved its
scrollback to disk. Work through this section.

### 1. Prepare the machine

**Best: a live USB with no persistence.** Boot Tails or a Debian/Ubuntu live
image, unplug the ethernet cable, disable Wi-Fi in the OS, and build and run the
tool from RAM. Nothing survives the reboot, which removes an entire category of
mistakes at once.

**Acceptable: a dedicated offline machine.** An old laptop with the Wi-Fi card
removed or disabled and no network cable, which you use for nothing else.

**Take these steps whichever you choose:**

- **Disconnect the network before you start.** Not after. The tool never opens a
  socket, but everything else on your machine might.
- **Disable swap**, or use a live system with no swap partition. Memory holding
  the phrase can otherwise be paged to disk and persist after the process exits:
  `sudo swapoff -a` for the session.
- **Never run this over SSH.** The phrase would cross the network and land in the
  memory of the machine you are typing on.
- **Disable hibernation.** Hibernating writes all of RAM to disk.
- **Check what is watching.** Close screen-sharing and recording software. Cover
  or unplug webcams. Draw the curtains — this is not paranoia, a phone camera
  pointed at a window is a real and documented attack.
- **Watch for shoulder surfers.** Including anyone whose phone is on the table.

### 2. Check your terminal isn't recording you

This is the mistake most people miss. The tool purges the screen and scrollback
when you're done, but it cannot reach into a log file your terminal wrote.

| Software | What to check |
|---|---|
| **iTerm2** | Turn off *Settings → Profiles → Session → Automatically log session to files*. iTerm2 can log every line to disk. |
| **tmux / screen** | Make sure `pipe-pane` logging is off. Detached panes also keep scrollback in memory long after you finish. |
| **Windows Terminal / VS Code** | Do not use the integrated terminal — editors keep their own buffers and crash dumps. Use a plain terminal. |
| **Anything with cloud sync** | Terminal profiles that sync settings or history to a vendor account. |
| **`script`, `asciinema`, `ttyrec`** | Obviously — but check you don't have one aliased or running from a previous session. |

The tool refuses to start if its output is not a terminal (exit code `2`), so you
cannot accidentally pipe a phrase into a file or a pager. That guard does not and
cannot cover terminal-side logging. You have to check that yourself.

### 3. Collect entropy well

You can mix any of the four sources in any order; the progress bar tracks the
total. Some guidance that materially affects quality:

**Dice — the source you can actually verify.**
This is the gold standard, because you can watch the randomness happen. Use
casino-grade dice if you have them (cheap dice are measurably biased). Roll
properly — shake and throw, don't tip the die over. Roll one die repeatedly, or
several distinguishable dice at once. Type the faces as digits 1–6; you can batch
them. Reaching 1024 bits on dice alone takes **410 rolls**, which is about half an
hour. That is a reasonable price for keys you never have to take on faith.

**Photos — one specific trap.**
Camera sensor noise is genuinely unpredictable, and a photo straight off your own
camera is an excellent source. But **never use an image that exists anywhere
else** — nothing you have posted to social media, sent in a message, backed up to
a cloud service, or downloaded. Public data has no entropy: if an attacker can
obtain the file, it contributes nothing. Take a fresh photo of something
cluttered, use it, then delete it.

**Keyboard and mouse — mash genuinely.**
Vary both what you press and the rhythm you press it with; the nanosecond timing
between events is doing much of the work. Don't type words, don't drum a steady
beat, and don't trace straight lines with the mouse. If the mouse counter doesn't
move, your terminal doesn't report mouse events — just use the other sources.

**Prefer a mix.** Using several independent sources means no single one has to be
as good as you hope it is.

### 4. Record the phrase

- **Write the words on paper, by hand, in order, and number them.** Then check
  them against the screen a second time. A single wrong word means the funds are
  gone with no error message to warn you.
- **Never photograph the paper.** A phone photo is in a cloud backup within
  seconds.
- **Never type the words into any computer**, including password managers, notes
  apps, spreadsheets, or "seed phrase checker" websites. Every one of those is a
  documented way people have lost coins.
- **Consider a metal backup plate** for anything you intend to hold long-term.
  Paper does not survive fire or flood.
- **Store it somewhere a burglar won't look and a fire won't reach.** If you split
  the phrase across locations, understand that naive splitting (12 words here, 12
  there) dramatically weakens it — use a real scheme like SLIP-39 or don't split.

### 5. Verify before you fund it

The tool already re-parses the phrase it generated and confirms it decodes back
to exactly the entropy it produced; it refuses to display anything if that check
fails. But verify your **transcription**, which is what actually goes wrong:

1. Restore the phrase into your wallet or hardware device from your written copy.
   A wrong word almost always fails the BIP39 checksum and is rejected outright.
2. **Send a small test amount first.** Confirm it arrives.
3. **Wipe the wallet and restore it again from paper**, then confirm the test
   amount is still visible. This proves your backup works — which is the thing
   you are actually betting on.
4. Only then fund it properly.

Do not paste your phrase into an online BIP39 tool to "check" it, even one that
claims to work offline in the browser. If you want an independent check, do it on
the same air-gapped machine with a local library.

### 6. Clean up

- Press Enter at the prompt only once the words are written down and verified —
  the tool then purges the screen and scrollback.
- **Reboot the machine.** On a live USB this destroys everything. On a persistent
  system it clears memory that zeroization cannot guarantee it reached.
- Delete any photo you used as an entropy source.
- Close the terminal window entirely rather than leaving it open.

---

## Entropy sources

| Source | What is absorbed | Credited | To reach 1024 bits alone |
|---|---|---|---|
| **Dice rolls** | Each d6 face you type | 2.5 bits per die | 410 rolls |
| **Keyboard mashing** | Keycode, modifiers, and the nanosecond gap since the last press | 3 bits per keystroke | 342 keystrokes |
| **Mouse movement** | Cursor position, event kind, and inter-event timing | 1 bit per event | 1024 events |
| **Files / images** | Full file contents, streamed and hashed | 4 bits per KiB, capped at 256 bits per file | 4 files of 64 KiB+ |

Every credit is deliberately below any plausible true value. A fair d6 actually
carries log₂(6) = 2.585 bits; a camera photo carries far more sensor noise than 4
bits per KiB. Mouse events are credited most stingily because terminal cells are
coarse and successive positions are heavily correlated — most of their real value
comes from the timing deltas riding along with them.

Files are deduplicated by SHA-512 digest, so adding the same photo twice earns
nothing.

**The bit counter is an honest lower-bound budget, not a measurement.** No program
can measure how unpredictable your dice throws really were. It tracks how much
unpredictability you have *claimed* to supply, under conservative assumptions.
The cryptographic strength of the output does not depend on that number being
accurate — see below.

---

## How the key is derived

1. Every event streams into a single SHA-512 pool. Each is prefixed with a
   domain-separating source tag, a length prefix, and a monotonic timestamp, so no
   two event streams can be confused for one another.
2. Once 1024 credited bits are collected, generation unlocks.
3. The final entropy is:

   ```
   entropy = SHA-256( "seed-entropy/v1/final" ‖ pool_digest ‖ 32 bytes from the OS CSPRNG )
   ```

4. Those 256 bits become the phrase: an 8-bit checksum is appended, the result is
   split into 24 × 11-bit indices, and each maps to a word in the English BIP39
   wordlist.

**Why the OS generator is mixed in.** This is defense-in-depth in both directions.
If your dice are biased or your credits are optimistic, the kernel CSPRNG still
guarantees full strength. If the kernel CSPRNG were backdoored, your dice still
do. Because both feed a hash, the output is as strong as the **better** of the
two, not the worse. You do not have to decide which one to trust.

The consequence worth stating plainly: **the phrase is not reproducible from your
dice rolls.** Re-entering the same rolls will not regenerate it. This is
intentional. If you want a phrase derived from dice alone so you can recreate it,
this is the wrong tool — use your hardware wallet's dice-entropy mode.

---

## Threat model

**What this defends against**

- A weak, biased, or backdoored operating-system RNG.
- A weak or backdoored RNG in a hardware wallet, if you use this instead.
- Any single entropy source being worse than you believed.
- Accidental disclosure through pipes, files, logs, or the network — the tool
  writes to nothing but the terminal and refuses to run without one.
- Scrollback recovery after generation, which is purged.

**What it does not defend against**

- **A compromised machine.** A keylogger, screen-capture malware, or a malicious
  terminal emulator sees the phrase regardless of what this program does. This is
  why the air gap matters more than the cryptography.
- **Memory forensics.** Buffers are zeroized (`Zeroizing`, and the mnemonic is
  `ZeroizeOnDrop`), but Rust may move values in memory before that, and
  zeroization cannot reach swap, hibernation images, or core dumps. Disable swap;
  reboot afterwards.
- **Cameras, screens, and people.** The phrase is displayed in plaintext because
  you have to write it down. Anything that can see your screen can see it.
- **Your backup.** Once the words are on paper, their security is physical.
- **Supply-chain risk in the toolchain.** You are trusting `rustc`, five crates,
  and your OS. See below.

**Not implemented:** BIP39 passphrases (the "25th word"). Add one in your wallet
after restoring if you want that layer.

---

## Auditing the code

The whole program is **under 900 lines of Rust including tests**, which is small
enough to read properly in an afternoon. If you are going to trust it with money,
read it — that is the point of it being this small.

| File | Lines | What to check |
|---|---|---|
| `src/pool.rs` | 199 | The SHA-512 accumulator, bit accounting, and final derivation. **This is the security-critical file.** |
| `src/main.rs` | 254 | Menu, the TTY guard, the round-trip self-check, phrase display, zeroization. |
| `src/sources/files.rs` | 145 | File reading and deduplication. |
| `src/sources/mouse.rs` | 109 | Mouse capture. |
| `src/sources/keyboard.rs` | 73 | Keystroke and timing capture. |
| `src/term.rs` | 68 | Terminal RAII guards, prompts, screen purge. |
| `src/sources/dice.rs` | 46 | Dice input validation. |

Five dependencies, all widely used:

| Crate | Why |
|---|---|
| [`sha2`](https://crates.io/crates/sha2) | SHA-256/SHA-512 (RustCrypto) |
| [`bip39`](https://crates.io/crates/bip39) | Wordlist and entropy→mnemonic encoding (rust-bitcoin) |
| [`crossterm`](https://crates.io/crates/crossterm) | Raw mode, key and mouse events |
| [`getrandom`](https://crates.io/crates/getrandom) | OS CSPRNG |
| [`zeroize`](https://crates.io/crates/zeroize) | Wiping secret buffers |

Things worth grepping for yourself, to confirm rather than take on trust: there
is no `std::net`, no `reqwest`, no `File::create`, and no `write` to any path.
The only filesystem access is `File::open` for entropy sources you choose.

To be thorough, `cargo vendor` the dependencies on an online machine, review or
carry them to the offline machine, and build with `--offline`.

---

## Building and testing

Requires **Rust 1.88 or newer** (edition 2024 and let-chains).

```bash
cargo build --release      # binary at ./target/release/seed-entropy
cargo test                 # 7 tests
cargo clippy --all-targets # clean
```

The test suite covers the five official BIP39 (Trezor) 256-bit test vectors,
entropy round-trips through parsing, and the pool's accounting and deduplication
behaviour. The vector tests are what prove the entropy→words step is correct: if
they pass, this tool's encoding agrees exactly with the reference implementation
every other wallet also agrees with.

**Controls:** `1`–`4` pick a source, `g` generates once you have 1024 bits, `q`
quits. Inside keyboard and mouse capture, press `Esc` to return to the menu. Dice
and file prompts end on an empty line.

---

## FAQ

**Can I regenerate the same phrase from the same dice rolls?**
No. The OS CSPRNG is mixed in, so every run is unique. This is intentional — see
[How the key is derived](#how-the-key-is-derived).

**Why 1024 bits when the phrase only holds 256?**
Because the bit counter is an estimate of *your* inputs, not a measurement. The
4× margin means the phrase still has full strength even if your entropy turns out
to be four times worse than credited.

**Is 24 words really better than 12?**
For practical purposes 12 words (128 bits) is already beyond brute force. 24 words
(256 bits) is the conservative choice and what this tool produces.

**Can I use this phrase with any wallet?**
Yes. It is standard BIP39, verified against the official test vectors, so it
restores into any BIP39-compatible wallet or hardware device.

**The mouse counter doesn't move.**
Your terminal doesn't report mouse events. Use dice, keyboard, or files instead.

**It printed an error instead of a phrase.**
The tool re-parses every phrase before displaying it and refuses to show one that
doesn't decode back to the entropy it generated. If you see that error, do not use
any partial output — please open an issue.

---

## Disclaimer

**Read this before using the software with real funds.**

**No audit.** This software has not been audited or formally reviewed by any
third party — no security firm, no independent cryptographer, no external
reviewer of any kind. The only review it has received is from its author. Passing
the official BIP39 test vectors proves the encoding step matches the reference
implementation; it proves nothing about the rest of the program, the entropy
collection, the memory handling, or the behaviour of your particular system.

**No warranty.** The software is provided **"as is", without warranty of any
kind**, express or implied, including but not limited to the warranties of
merchantability, fitness for a particular purpose, and noninfringement. See the
[LICENSE](LICENSE) file for the full text.

**No liability.** In no event shall the author or copyright holder be liable for
any claim, damages, or other liability — including, without limitation, the total
and permanent loss of any cryptocurrency — arising from or in connection with this
software or its use.

**Use at your own risk.** By using this software you accept that:

- You are solely responsible for the security of any keys it generates and for
  every consequence that follows from using them.
- Bitcoin transactions are irreversible and there is no recovery mechanism, no
  support line, and no authority who can undo a mistake or restore a lost phrase.
- Undiscovered bugs, platform-specific behaviour, a compromised machine, a
  compromised dependency, or your own operational mistakes could result in the
  irreversible loss of everything the wallet holds.
- You have satisfied yourself that the code does what it claims — ideally by
  reading it, since it is deliberately small enough to read.

**Test first.** Generate a phrase, restore it from your written paper backup,
send a small amount, wipe the wallet, and restore from paper again. Only fund it
properly once you have proven the whole loop works. Never store more than you are
willing to lose entirely.

**This is not financial, legal, or security advice.** You are responsible for
complying with the laws that apply to you.

---

## Contributing

Review and scrutiny are genuinely welcome — this is exactly the kind of software
that benefits from more eyes. Please open an issue for anything that looks wrong,
especially in `src/pool.rs`.

Please report suspected security issues privately through GitHub's
[security advisory](https://github.com/barrydeen/bitcoin-seed-entropy/security/advisories/new)
form rather than in a public issue, so they can be fixed before they are
advertised.

## License

[MIT](LICENSE) © 2026 Barry Deen
