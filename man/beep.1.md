% BEEP(1) beep 0.1.0 | User Commands
% Elizabeth Ashford <elizabeth.jennifer.myers@gmail.com>
% April 2026

# NAME
beep - play tones through the default audio device

# SYNOPSIS

**beep** \[*OPTIONS*\]...

**beep** \[*OPTIONS*\] \[**-n**\] \[*OPTIONS*\] ...

**beep** \[**-h**|**\--help**\]

**beep** \[**-v**|**\--version**\]

# DESCRIPTION
**beep** plays one or more tones through the system's default audio output device. It is a cross-platform reimplementation of the classic Unix *beep*(1) utility by Johnathan Nightingale, which drove the PC speaker directly. Because modern systems rarely have PC speakers, this version uses the standard audio subsystem via the *rodio* library, working on Linux, macOS, and Windows.

The generated tone is a square wave passed through a high-pass and low-pass filter, approximating the frequency response of a small magnetic speaker or piezoelectric buzzer typical of PC-era beepers.

All durations are in milliseconds, frequencies in hertz, and repetition counts are dimensionless.

If an option is specified more than once, the last occurrence wins. So **beep -f 200 -f 300** beeps at 300 Hz.

# OPTIONS

**-f** *FREQ*
:   Frequency in hertz. Default: 440.

**-l** *LEN*
:   Tone length in milliseconds. Default: 200.

**-r** *REPS*
:   Number of repetitions. Default: 1.

**-d** *DELAY*
:   Delay in milliseconds between beeps. No delay occurs after the final beep. Default: 100.

**-D** *DELAY*
:   Same as **-d**, but also applies the delay after the final beep. Useful when chaining *beep* commands together, where consistent timing between invocations matters.

**-s**
:   Read stdin line by line, echoing each line to stdout and beeping once per line. Useful in pipelines.

**-c**
:   Read stdin byte by byte, echoing each byte to stdout and beeping once per byte.

**-n**, **\--new**
:   Start a new beep with default values. All following options apply to the new beep. This allows chaining multiple different beeps in a single invocation. See EXAMPLES.

**-v**, **\--version**
:   Print version and exit.

**-h**, **\--help**
:   Print usage information and exit.

# EXAMPLES
Play the default beep (440 Hz for 200 ms):

    beep

Play a higher-pitched beep like the traditional terminal bell:

    beep -f 750

Play three half-second beeps at 1 kHz with 200 ms between them:

    beep -f 1000 -l 500 -r 3 -d 200

Chain three different notes to play a short melody:

    beep -f 523 -l 200 -n -f 659 -l 200 -n -f 784 -l 400

Beep whenever a line of build output is produced, preserving the output for logging:

    make 2>&1 | beep -s | tee build.log

Beep on completion of a long-running command:

    long-running-command; beep -r 3

# EXIT STATUS
**0**
:   Success.

**non-zero**
:   Argument parsing failed, audio device could not be opened, or an I/O error occurred.

On Unix, *beep* resets SIGPIPE to its default disposition, so it terminates silently when a downstream process in a pipeline closes stdin.

# LIMITATIONS
*beep* does not support the **-e**/**\--device** flag of the original, as there is no underlying PC speaker to select; audio output goes to the system default.

The **\--verbose**/**\--debug** flag of the original is not implemented.

# SEE ALSO
The original *beep* utility: <https://github.com/johnath/beep>

*rodio*, the Rust audio library used: <https://docs.rs/rodio>

# LICENSE
Released into the public domain under the Unlicense.
