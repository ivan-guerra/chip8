# Chip8

https://github.com/user-attachments/assets/cac078d3-fc50-45d2-9451-3cdb248475ce

This project implements a Chip8 emulator in Rust. This particular emulator
implements the original Chip8 specification for the COMSAC VIP microcomputer as
decribed in Tobias Langoff's ["Guide to Making a Chip8 Emulator"][1].

### Usage

Below is the program usage:

```bash
A simple CHIP-8 emulator written in Rust

Usage: chip8 [OPTIONS] --rom-path <ROM_PATH>

Options:
  -f, --frame-rate <FRAME_RATE>  Frame rate in frames per second [default: 60]
  -i, --ips <IPS>                Instructions per second [default: 700]
  -r, --rom-path <ROM_PATH>      Path to the ROM file to run
  -h, --help                     Print help
  -V, --version                  Print version
```

The frame rate and number of instructions per second are the only emulator
tunables. You may find that some games run a bit too fast at the default 60Hz
frame rate. You can reduce the rate using the `--frame-rate` option as needed.
The `--ips` option allows you to set the number of instructions per second. Most
games from the COMSAC VIP era run at roughly 700 IPS. See the game ROM's README
for more information as to whether you need to change the IPS setting.

Here are some links to a number of fun Chip8 game ROMs:

- [Chip8Archive][2]
- [Alexander Dickenson's Chip8 ROMs][3]

Most games do not come with a README or any sort of instructions. You have to
muck with the keys to figure out how to play!

> **Note:** A number of the ROMs utilize the `0x0NNN` or "Execute Machine
> Language" instruction. This instruction is designed to execute machine code on
> the COMSAC VIP's 1802 CPU. You're probably not running the emulator on a
> machine with a 1802 CPU. ROMs with the `0x0NNN` instruction are unplayable.
> The emulator will exit with an error if it encounters this instruction in a
> ROM.

### Testing

This emulator passes [Timendu's Chip8 Test Suite][4]. The relevant ROMs from the
test suite are included in the [`tests/`](tests/) directory. You can run the
tests using the following command:

```bash
chip8 --rom-path tests/<TEST-NAME>.ch8
```

[1]: https://tobiasvl.github.io/blog/write-a-chip-8-emulator/
[2]: https://github.com/JohnEarnest/chip8Archive/tree/master/roms
[3]: https://github.com/alexanderdickson/Chip-8-Emulator/tree/master/roms
[4]: https://github.com/Timendus/chip8-test-suite?tab=readme-ov-file#chip-8-test-suite
