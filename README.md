![Test Status](https://github.com/nirajrao/chip8/actions/workflows/tests.yml/badge.svg)

# CHIP-8

## Background

This was a fun side project for me to learn about Rust, emulators, and graphics.
CHIP-8 tends to be the recommended emulator to implement so I went with that!
(CHIP-8 isn't hardware so this is technically a CHIP-8 interpreter, not a
CHIP-8 emulator).

I never implemented sound, but most other things should be working. I tried to
include comments whenever possible in the code, mainly for my own
understanding. I also wrote a ton of tests.

Some resources are included below which helped me greatly when working on this project. In no particular order:

* https://multigesture.net/articles/how-to-write-an-emulator-chip-8-interpreter/
* https://github.com/mattmikolay/chip-8/wiki/Mastering-CHIP%E2%80%908
* https://github.com/starrhorne/chip8-rust
* https://en.wikipedia.org/wiki/CHIP-8
* https://tobiasvl.github.io/blog/write-a-chip-8-emulator/

## Instructions

To run, simply `cargo run --filename` (e.g. `cargo run bc_test.ch8`)
