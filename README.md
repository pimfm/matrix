# Matrix Rain

A terminal-based Matrix digital rain effect written in Rust. Watch cascading green characters fall down your screen, complete with flickering glyphs, hidden easter eggs, and glitch effects — just like the iconic code rain from The Matrix.

<p align="center">
  <img src="demo.gif" alt="Matrix rain effect running in a terminal" width="800">
</p>

## Features

- **Authentic character set** — Half-width Katakana, digits, and symbols matching the original film aesthetic
- **Smooth animation** at 20 FPS with variable-speed streams and fading green trails
- **Easter eggs** — Hidden messages from the movie appear as flash text or embedded in the rain (look for "WAKE UP NEO", "THERE IS NO SPOON", and many more)
- **Glitch effects** — Random bright-white row flickers for that digital interference feel
- **Responsive** — Automatically adapts when you resize your terminal window

## Getting Started

### Prerequisites

You'll need [Rust](https://www.rust-lang.org/tools/install) installed on your system.

### Run it

```sh
git clone https://github.com/pimfm/matrix.git
cd matrix
cargo run --release
```

### Controls

| Key | Action |
|---|---|
| `q` or `Esc` | Quit |
| `Ctrl+C` | Quit |

## How It Works

The animation renders using [Ratatui](https://ratatui.rs/) (a Rust TUI framework) and [Crossterm](https://docs.rs/crossterm/) for terminal control. Each column of the terminal can have one or more falling "streams" — a head character in bright white followed by a green trail that fades to dark green. Characters randomly mutate as they fall, creating the signature flickering effect.

Every stream has a randomized speed, length, and starting position so the rain always looks organic and unpredictable.

## Built With

- [Rust](https://www.rust-lang.org/) — Systems programming language
- [Ratatui](https://ratatui.rs/) — Terminal UI framework
- [Crossterm](https://docs.rs/crossterm/) — Cross-platform terminal manipulation
- [rand](https://docs.rs/rand/) — Random number generation

## License

This project is open source. Feel free to use it, modify it, and share it.
