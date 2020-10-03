# CHIP-8 Emulator

This is an implementation of an emulator of a chip supporting the CHIP-8 
[instruction set](https://en.wikipedia.org/wiki/CHIP-8). For graphical output, 
it relies on the (cursive)[https://github.com/gyscos/cursive] text user interface 
library.


# Usage

The emulator is written in Rust. For a guide on getting started with Rust, refer 
to [this page](https://www.rust-lang.org/learn/get-started).

The emulator takes the path to the CHIP-8 program to be executed.

```
cargo run [path-to-chip-8-program]
```

## Key Mapping

CHIP-8 has 16 input pins (`0x0` - `0xF`). They can be set using keys according to the
mapping below.

| Key (`Mapping`) | Key (`Mapping`) | Key (`Mapping`) | Key (`Mapping`) | 
| :-------------: | :-------------: | :-------------: | :-------------: | 
|    1 (`0x1`)    |    2 (`0x2`)    |    3 (`0x3`)    |    4 (`0xC`)    |
|    Q (`0x4`)    |    W (`0x5`)    |    E (`0x6`)    |    R (`0xD`)    |
|    A (`0x7`)    |    S (`0x8`)    |    D (`0x9`)    |    F (`0xE`)    |
|    Z (`0xA`)    |    X (`0x0`)    |    C (`0xB`)    |    V (`0xF`)    |

Given that OS X requires special permissions to listen to key-release events, we decided to
work around this by _mapping the space bar as a key-release bar_. So once a key is pressed
it will remain set until the space bar is hit. Pressing escape will quit the emulator.
