# Chip-8 Emulator

An emulator for the Chip-8 programming language.  

It can run most demo programs but seems to have an issue with input, so most games aren't fun.

## Features
- Supports all 35 instructions (excluding the deprecated SYS)
- Includes GUI, keyboard and sound support
- Synchronized emulation for stable execution speed
- Cross plattform support (Windows, Linux, Mac) but only tested on Windows

## TODO
- Fix input issue: Most but not all programs only seem to read key 0x0
- Turn sound on by default
- Add support for defining an execution speed factor
- Support variable display and memory sizes

## Development Dependencies
- Rust 1.36.0 (https://www.rust-lang.org/) or compatible
- SDL 2.0.12 development libraries (http://www.libsdl.org/)

## License
This project is licensed under the GNU GPL v3.0.  
See the [license file](LICENSE) for further information.
