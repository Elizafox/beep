# beep
[![CI](https://github.com/Elizafox/beep/actions/workflows/ci.yml/badge.svg)](https://github.com/Elizafox/beep/actions/workflows/ci.yml)
[![Release](https://github.com/Elizafox/beep/actions/workflows/release.yml/badge.svg)](https://github.com/Elizafox/beep/actions/workflows/release.yml)
[![License: Unlicense](https://img.shields.io/badge/license-Unlicense-blue.svg)](https://unlicense.org/)

A cross-platform reimplementation of the classic Unix [`beep`](https://github.com/johnath/beep) utility, written in Rust.

It uses the system's audio output rather than the PC speaker, so it works on modern machines.

This utility emulates the frequency response of a classic PC speaker by running a square wave through high-pass and low-pass filters, because an unfiltered square wave through good laptop speakers sounds worse than you'd think (and far less authentic).

## Installation
```sh
cargo install --path .
```

Or build and copy the binary yourself:

```sh
cargo build --release
cp target/release/beep ~/.local/bin/
```

## Usage
```sh
beep                          # default 440 Hz beep for 200 ms
beep -f 750                   # terminal-bell frequency
beep -f 1000 -l 500 -r 3      # three half-second beeps at 1 kHz
beep -f 523 -l 100 -n -f 659 -l 100 -n -f 784 -l 200   # chord sequence
```

### Options

| Flag | Description | Default |
|------|-------------|---------|
| `-f FREQ` | Frequency in Hz | 440 |
| `-l LEN` | Tone length in ms | 200 |
| `-r REPS` | Number of repetitions | 1 |
| `-d DELAY` | Delay between beeps in ms (no delay after final beep) | 100 |
| `-D DELAY` | Same as `-d`, but delay after the final beep too | — |
| `-s` | Beep after each line of stdin (echoes input to stdout) | — |
| `-c` | Beep after each byte of stdin (echoes input to stdout) | — |
| `-n`, `--new` | Start a new beep with default values | — |
| `-v`, `-V`, `--version` | Print version | — |
| `-h`, `--help` | Print help | — |

### Stream modes
With `-s` or `-c`, beep reads stdin and writes it back to stdout unchanged, beeping on each line or byte respectively. This makes it easy to splice into a pipeline:

```sh
make 2>&1 | beep -s | tee build.log
```

### Chaining notes with `-n`
Each `--new` starts a fresh beep with default values, letting you play sequences of different tones:

```sh
beep -f 523 -l 200 -n -f 659 -l 200 -n -f 784 -l 400
```

This is different from `-r`, which repeats the *same* beep multiple times.

## Compatibility with the original
This is a faithful reimplementation of the [original `beep`](https://github.com/johnath/beep) by Johnathan Nightingale, with a few differences:

- Uses the default audio output device via [rodio](https://docs.rs/rodio) instead of the PC speaker. No root or `ioperm` needed.
- Runs on Linux, macOS, Windows, and anywhere else rodio supports.
- The `-e`/`--device` flag is accepted but not implemented (no underlying PC speaker to choose).

## Why any of this?
The original `beep` is a wonderful tool that doesn't work on most modern systems, because most modern systems don't have PC speakers.

## License
Released into the public domain under the [Unlicense](https://unlicense.org/). See `LICENSE` for the full text.
