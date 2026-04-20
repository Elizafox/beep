//! `beep` — a cross-platform reimplementation of the classic Unix beep utility.
//!
//! Plays tones through the system audio device instead of the PC speaker,
//! making it work on modern laptops, macOS, and Windows. Supports the same
//! command-line grammar as the original, including `-n`/`--new` for chaining
//! multiple beeps and `-s`/`-c` stream modes for pipeline integration.

#![warn(rust_2018_idioms, unused_lifetimes)]

use lexopt::{Parser, prelude::*};
use rodio::{
    Player,
    source::{Source, SquareWave},
};
use std::{
    io::{BufReader, prelude::*, stdin, stdout},
    process::exit,
    thread::sleep,
    time::Duration,
};

#[derive(Debug, Clone, PartialEq, Eq)]
enum StreamMode {
    Lines,
    Chars,
}

#[derive(Debug, Clone)]
struct Note {
    freq: f32,
    length: u64,
    reps: u64,
    delay: u64,
    delay_after_last: bool,
    stream_mode: Option<StreamMode>,
}

impl Default for Note {
    fn default() -> Self {
        Self {
            freq: 440.0,
            length: 200,
            reps: 1,
            delay: 100,
            delay_after_last: false,
            stream_mode: None,
        }
    }
}

/// Reset SIGPIPE handling to the kernel default (terminate on broken pipe).
///
/// # Safety
///
/// This function modifies process-global signal disposition. The caller must
/// ensure no other code in the process relies on the current SIGPIPE handler.
/// In practise this means calling it once, early in `main`, before spawning
/// threads or initialising libraries that might install their own handlers.
#[cfg(unix)]
unsafe fn install_sigpipe() -> nix::Result<()> {
    use nix::sys::signal::{SaFlags, SigAction, SigHandler, SigSet, Signal, sigaction};
    let action = SigAction::new(SigHandler::SigDfl, SaFlags::empty(), SigSet::empty());

    // Safety: SIG_DFL is always safe to install for SIGPIPE
    unsafe {
        sigaction(Signal::SIGPIPE, &action)?;
    }
    Ok(())
}

fn parse_args_from<I, S>(args: I) -> Result<(bool, Vec<Note>), lexopt::Error>
where
    I: IntoIterator<Item = S>,
    S: Into<std::ffi::OsString>,
{
    let mut notes = Vec::new();
    let mut current = Note::default();
    let mut parser = Parser::from_iter(args);
    let mut verbose = false;
    let mut warn_e = false;

    while let Some(arg) = parser.next()? {
        match arg {
            Short('f') => current.freq = parser.value()?.parse()?,
            Short('l') => current.length = parser.value()?.parse()?,
            Short('r') => current.reps = parser.value()?.parse()?,
            Short('d') => {
                current.delay = parser.value()?.parse()?;
                current.delay_after_last = false;
            }
            Short('D') => {
                current.delay = parser.value()?.parse()?;
                current.delay_after_last = true;
            }
            Short('c') => current.stream_mode = Some(StreamMode::Chars),
            Short('s') => current.stream_mode = Some(StreamMode::Lines),
            Short('n') | Long("new") => {
                notes.push(current);
                current = Note::default();
            }
            Short('h') | Long("help") => {
                print_help();
                exit(0);
            }
            Short('v') | Short('V') | Long("version") => {
                print_version();
                exit(0);
            }
            Short('e') | Long("device") => {
                let _ = parser.value()?;
                warn_e = true;
            }
            Long("verbose") | Long("debug") => verbose = true,
            _ => return Err(arg.unexpected()),
        }
    }

    if warn_e {
        eprintln!("Warning: -e flag is ignored (no device to select)");
    }

    notes.push(current);
    Ok((verbose, notes))
}

fn parse_args() -> Result<(bool, Vec<Note>), lexopt::Error> {
    parse_args_from(std::env::args_os())
}

fn print_help() {
    println!("beep - play a tone through the default audio device");
    println!();
    println!("Usage: beep [OPTIONS]...");
    println!();
    println!("Options:");
    println!("  -f FREQ               Frequency in Hz (default: 440)");
    println!("  -l LEN                Tone length in ms (default: 200)");
    println!("  -r REPS               Number of repetitions (default: 1)");
    println!("  -d DELAY              Delay between beeps in ms (default: 100)");
    println!("  -D DELAY              Same as -d, but delay after the final beep too");
    println!("  -s                    Beep after each line of stdin (echoes input)");
    println!("  -c                    Beep after each byte of stdin (echoes input)");
    println!("  -n, --new             Start a new beep with default values");
    println!(
        "  --verbose, --debug    Print beep parameters to stderr before playing (compatibility)"
    );
    println!("  -v, --version         Show version");
    println!("  -h, --help            Show this help");
    println!();
    println!("Accepted for compatibility with the original beep (ignored):");
    println!("  -e, --device DEVICE  (no PC speaker to select)");
    println!();
    println!(
        "Usage: beep [-f FREQ] [-l LEN] [-r REPS] [-d|-D DELAY] [-c] [-s] [--verbose|--debug] [-n ...]"
    );
}

fn print_version() {
    println!("beep {}", env!("CARGO_PKG_VERSION"));
    println!("Written by Elizabeth Ashford. Released under the Unlicense.");
    println!("This is free and unencumbered software released into the public domain.");
}

fn play_note(player: &Player, note: &Note, verbose: bool) {
    if verbose {
        // Compatible with the output of the original beep program
        eprintln!(
            "[DEBUG] {} times {} ms beeps ({} delay between, {} after) @ {} Hz",
            note.reps,
            note.length,
            note.delay,
            if note.delay_after_last { note.delay } else { 0 },
            note.freq,
        );
    }

    for i in 1..=note.reps {
        // The below filters are designed to somewhat faithfully reproduce the frequency response
        // of the beeper found in most x86 PCs that have them (most modern ones do not, or only
        // have it as an optional component). It is a rather difficult task to reproduce the real
        // sound of an old PC beeper, as factors such as the size of the beeper, the type of
        // beeper (some computers used a small magnetic speaker, others used a piezoelectric
        // element; both are low quality, but sound different), case resonance, mounting of the
        // beeper (some are surface-mount on the board, some are connected via cable and just
        // hang there, some are mounted onto the case), etc. However, this should still be much
        // better than for example, a typical laptop's implementation of it (just output a square
        // wave through the speakers).
        let source = SquareWave::new(note.freq)
            .take_duration(Duration::from_millis(note.length))
            .amplify(0.2)
            .high_pass_with_q(1500, 0.707)
            .low_pass_with_q(9000, 0.707);
        player.append(source);
        player.sleep_until_end();

        if i != note.reps || note.delay_after_last {
            sleep(Duration::from_millis(note.delay));
        }
    }
}

fn stream_mode_lines(player: &Player, template: &Note, verbose: bool) -> std::io::Result<()> {
    let stdin = stdin();
    let reader = BufReader::new(stdin.lock());
    let mut stdout = stdout().lock();

    for line in reader.lines() {
        let line = line?;
        if let Err(e) = writeln!(stdout, "{line}").and_then(|()| stdout.flush()) {
            if e.kind() == std::io::ErrorKind::BrokenPipe {
                return Ok(());
            }
            return Err(e);
        }
        play_note(player, template, verbose);
    }
    Ok(())
}

// False positive lint on stdin, which we want to keep held for the whole function
#[allow(clippy::significant_drop_tightening)]
fn stream_mode_chars(player: &Player, template: &Note, verbose: bool) -> std::io::Result<()> {
    let mut stdin = stdin().lock();
    let mut stdout = stdout().lock();
    let mut buf = [0u8; 1];

    loop {
        if stdin.read(&mut buf)? == 0 {
            break;
        }
        if let Err(e) = stdout.write_all(&buf).and_then(|()| stdout.flush()) {
            if e.kind() == std::io::ErrorKind::BrokenPipe {
                return Ok(());
            }
            return Err(e);
        }
        play_note(player, template, verbose);
    }
    Ok(())
}

fn main() -> Result<(), Box<dyn std::error::Error>> {
    // Only required on Unix
    // Safety: SIG_DFL is always safe to install for SIGPIPE
    #[cfg(unix)]
    unsafe {
        install_sigpipe()?;
    }

    let (verbose, notes) = parse_args()?;

    let mut handle = rodio::DeviceSinkBuilder::open_default_sink()?;
    handle.log_on_drop(false); // Ignore spurious warnings
    let player = Player::connect_new(handle.mixer());

    for note in &notes {
        match note.stream_mode {
            None => play_note(&player, note, verbose),
            Some(StreamMode::Lines) => stream_mode_lines(&player, note, verbose)?,
            Some(StreamMode::Chars) => stream_mode_chars(&player, note, verbose)?,
        }
    }

    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn default_note_when_no_args() {
        let (verbose, notes) = parse_args_from::<_, &str>([]).unwrap();
        assert!(!verbose);
        assert_eq!(notes.len(), 1);
        assert_eq!(notes[0].freq, 440.0);
        assert_eq!(notes[0].length, 200);
    }

    #[test]
    fn last_value_of_repeated_flag_wins() {
        let (verbose, notes) = parse_args_from(["beep", "-f", "200", "-f", "300"]).unwrap();
        assert!(!verbose);
        assert_eq!(notes.len(), 1);
        assert_eq!(notes[0].freq, 300.0);
    }

    #[test]
    fn new_flag_creates_separate_notes() {
        let (verbose, notes) = parse_args_from(["beep", "-f", "500", "-n", "-f", "800"]).unwrap();
        assert!(!verbose);
        assert_eq!(notes.len(), 2);
        assert_eq!(notes[0].freq, 500.0);
        assert_eq!(notes[1].freq, 800.0);
        // second note should have defaults for unspecified fields
        assert_eq!(notes[1].length, 200);
    }

    #[test]
    fn new_flag_resets_to_defaults() {
        let (verbose, notes) =
            parse_args_from(["beep", "-f", "500", "-l", "1000", "-n", "-f", "800"]).unwrap();
        assert!(!verbose);
        assert_eq!(notes[1].length, 200); // not 1000 from first note
    }

    #[test]
    fn lowercase_d_disables_end_delay() {
        let (verbose, notes) = parse_args_from(["beep", "-d", "50"]).unwrap();
        assert!(!verbose);
        assert_eq!(notes[0].delay, 50);
        assert!(!notes[0].delay_after_last);
    }

    #[test]
    fn uppercase_d_enables_end_delay() {
        let (verbose, notes) = parse_args_from(["beep", "-D", "50"]).unwrap();
        assert!(!verbose);
        assert_eq!(notes[0].delay, 50);
        assert!(notes[0].delay_after_last);
    }

    #[test]
    fn mixing_lowercase_d_and_uppercase_d_last_wins() {
        let (verbose, notes) = parse_args_from(["beep", "-d", "100", "-D", "50"]).unwrap();
        assert!(!verbose);
        assert!(notes[0].delay_after_last);

        let (verbose, notes) = parse_args_from(["beep", "-D", "50", "-d", "100"]).unwrap();
        assert!(!verbose);
        assert!(!notes[0].delay_after_last);
    }

    #[test]
    fn stream_mode_s() {
        let (verbose, notes) = parse_args_from(["beep", "-s"]).unwrap();
        assert!(!verbose);
        assert_eq!(notes[0].stream_mode, Some(StreamMode::Lines));
    }

    #[test]
    fn e_consumes_device() {
        let (verbose, notes) = parse_args_from(["beep", "-e", "foo", "-f", "500"]).unwrap();
        assert!(!verbose);
        assert_eq!(notes.len(), 1);
        assert_eq!(notes[0].freq, 500.0);
        assert_eq!(notes[0].length, 200);
    }

    #[test]
    fn verbose_set() {
        let (verbose, notes) = parse_args_from(["beep", "--verbose"]).unwrap();
        assert!(verbose);
        assert_eq!(notes.len(), 1);
        assert_eq!(notes[0].freq, 440.0);
        assert_eq!(notes[0].length, 200);
    }

    #[test]
    fn verbose_set_debug() {
        let (verbose, notes) = parse_args_from(["beep", "--debug"]).unwrap();
        assert!(verbose);
        assert_eq!(notes.len(), 1);
        assert_eq!(notes[0].freq, 440.0);
        assert_eq!(notes[0].length, 200);
    }

    #[test]
    fn verbose_set_with_params() {
        let (verbose, notes) = parse_args_from(["beep", "--verbose", "-f", "500"]).unwrap();
        assert!(verbose);
        assert_eq!(notes.len(), 1);
        assert_eq!(notes[0].freq, 500.0);
        assert_eq!(notes[0].length, 200);
    }

    #[test]
    fn verbose_set_with_params_debug() {
        let (verbose, notes) = parse_args_from(["beep", "--debug", "-f", "500"]).unwrap();
        assert!(verbose);
        assert_eq!(notes.len(), 1);
        assert_eq!(notes[0].freq, 500.0);
        assert_eq!(notes[0].length, 200);
    }

    #[test]
    fn verbose_set_after_n() {
        let (verbose, notes) =
            parse_args_from(["beep", "-f", "500", "-n", "-f", "300", "--verbose"]).unwrap();
        assert!(verbose);
        assert_eq!(notes.len(), 2);
        assert_eq!(notes[0].freq, 500.0);
        assert_eq!(notes[0].length, 200);
        assert_eq!(notes[1].freq, 300.0);
        assert_eq!(notes[1].length, 200);
    }

    #[test]
    fn verbose_set_after_n_debug() {
        let (verbose, notes) =
            parse_args_from(["beep", "-f", "500", "-n", "-f", "300", "--debug"]).unwrap();
        assert!(verbose);
        assert_eq!(notes.len(), 2);
        assert_eq!(notes[0].freq, 500.0);
        assert_eq!(notes[0].length, 200);
        assert_eq!(notes[1].freq, 300.0);
        assert_eq!(notes[1].length, 200);
    }

    #[test]
    fn verbose_debug_both_set() {
        let (verbose, notes) = parse_args_from(["beep", "--verbose", "--debug"]).unwrap();
        assert!(verbose);
        assert_eq!(notes[0].freq, 440.0);
        assert_eq!(notes[0].length, 200);
    }

    #[test]
    fn unknown_flag_errors() {
        assert!(parse_args_from(["beep", "--unknown"]).is_err());
    }
}
