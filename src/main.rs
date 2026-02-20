use crossterm::{
    event::{self, Event, KeyCode, KeyEventKind},
    terminal::{self, EnterAlternateScreen, LeaveAlternateScreen},
    ExecutableCommand,
};
use rand::Rng;
use ratatui::{
    backend::CrosstermBackend,
    buffer::Buffer,
    layout::Rect,
    style::Color,
    widgets::Widget,
    Terminal,
};
use std::{
    io::{self, stdout},
    time::{Duration, Instant},
};

// Katakana-ish + latin + digits + symbols for that authentic matrix look
const MATRIX_CHARS: &[char] = &[
    // Half-width katakana
    'ｦ', 'ｧ', 'ｨ', 'ｩ', 'ｪ', 'ｫ', 'ｬ', 'ｭ', 'ｮ', 'ｯ',
    'ｰ', 'ｱ', 'ｲ', 'ｳ', 'ｴ', 'ｵ', 'ｶ', 'ｷ', 'ｸ', 'ｹ',
    'ｺ', 'ｻ', 'ｼ', 'ｽ', 'ｾ', 'ｿ', 'ﾀ', 'ﾁ', 'ﾂ', 'ﾃ',
    'ﾄ', 'ﾅ', 'ﾆ', 'ﾇ', 'ﾈ', 'ﾉ', 'ﾊ', 'ﾋ', 'ﾌ', 'ﾍ',
    'ﾎ', 'ﾏ', 'ﾐ', 'ﾑ', 'ﾒ', 'ﾓ', 'ﾔ', 'ﾕ', 'ﾖ', 'ﾗ',
    'ﾘ', 'ﾙ', 'ﾚ', 'ﾛ', 'ﾜ', 'ﾝ',
    // Digits
    '0', '1', '2', '3', '4', '5', '6', '7', '8', '9',
    // Symbols
    ':', '.', '"', '=', '*', '+', '-', '<', '>', '|',
    '¦', '╌', '┊', '∞', '≡', '±', '∓', '∴', '∵', '⊕',
];

// Easter egg phrases - hidden messages in the rain
const EASTER_EGGS: &[&str] = &[
    "WAKE UP NEO",
    "FOLLOW THE WHITE RABBIT",
    "THERE IS NO SPOON",
    "THE ONE",
    "KNOCK KNOCK",
    "FREE YOUR MIND",
    "RED PILL",
    "BLUE PILL",
    "MORPHEUS",
    "TRINITY",
    "ZION",
    "WHOA",
    "I KNOW KUNG FU",
    "DEJA VU",
    "RABBIT HOLE",
    "MR ANDERSON",
    "THE MATRIX HAS YOU",
    "CHOICE IS AN ILLUSION",
    "NOT LIKE THIS",
    "DODGE THIS",
    "GUNS LOTS OF GUNS",
    "WELCOME TO THE DESERT OF THE REAL",
    "WHAT IS REAL",
    "BELIEVE",
    "SYSTEM FAILURE",
    "HE IS THE ONE",
    "DO NOT TRY TO BEND THE SPOON",
    "THERE IS NO SPOON ONLY ZUUL",
    "TAKE THE RED PILL",
    "WERE YOU LISTENING OR LOOKING AT THE WOMAN IN THE RED DRESS",
    "IM GOING TO SHOW THEM A WORLD WITHOUT RULES",
    "42",
    "HELLO WORLD",
    "COGITO ERGO SUM",
    "WHY DO MY EYES HURT",
    "BECAUSE YOUVE NEVER USED THEM BEFORE",
    "THE CAKE IS A LIE",
];

/// A single falling stream/column of characters
struct Stream {
    col: u16,
    head_y: f64,       // Current y position of the head (fractional for smooth speed)
    speed: f64,        // Cells per tick
    length: u16,       // Trail length
    chars: Vec<char>,  // Pre-generated characters for the trail
    active: bool,
    // Easter egg: if set, this stream spells out a word vertically
    easter_egg: Option<EasterEgg>,
    // Random character mutation timer
    mutate_counter: u16,
}

struct EasterEgg {
    word: &'static str,
    char_index: usize, // Which character of the word we're currently showing
}

struct MatrixRain {
    streams: Vec<Stream>,
    width: u16,
    height: u16,
    tick: u64,
    // Horizontal easter egg: a phrase that flashes briefly across columns
    flash_message: Option<FlashMessage>,
}

struct FlashMessage {
    text: &'static str,
    row: u16,
    start_col: u16,
    ticks_remaining: u16,
    fade_stage: u8, // 0=bright, 1=medium, 2=dim
}

impl MatrixRain {
    fn new(width: u16, height: u16) -> Self {
        let mut rain = MatrixRain {
            streams: Vec::new(),
            width,
            height,
            tick: 0,
            flash_message: None,
        };
        rain.populate_streams();
        rain
    }

    fn populate_streams(&mut self) {
        let mut rng = rand::rng();
        // Figure out which columns already have a stream
        let mut col_has_stream = vec![false; self.width as usize];
        for stream in &self.streams {
            if (stream.col as usize) < col_has_stream.len() {
                col_has_stream[stream.col as usize] = true;
            }
        }
        // Create streams for empty columns, staggered
        for col in 0..self.width {
            if !col_has_stream[col as usize] && rng.random_range(0..100) < 70 {
                self.spawn_stream(col, true);
            }
        }
    }

    fn spawn_stream(&mut self, col: u16, random_start: bool) {
        let mut rng = rand::rng();
        let length = rng.random_range(4..=self.height.max(8).min(40));
        let speed = rng.random_range(3..=12) as f64 / 10.0; // 0.3 to 1.2

        let head_y = if random_start {
            rng.random_range(-(self.height as f64)..self.height as f64)
        } else {
            -(length as f64)
        };

        // Generate random characters for the trail
        let chars: Vec<char> = (0..length as usize + 10)
            .map(|_| MATRIX_CHARS[rng.random_range(0..MATRIX_CHARS.len())])
            .collect();

        // Small chance this stream carries an easter egg word (vertical)
        let easter_egg = if rng.random_range(0..100) < 3 {
            let word = EASTER_EGGS[rng.random_range(0..EASTER_EGGS.len())];
            Some(EasterEgg {
                word,
                char_index: 0,
            })
        } else {
            None
        };

        self.streams.push(Stream {
            col,
            head_y,
            speed,
            length,
            chars,
            active: true,
            easter_egg,
            mutate_counter: rng.random_range(3..15),
        });
    }

    fn tick(&mut self) {
        self.tick += 1;
        let mut rng = rand::rng();
        let height = self.height;
        let width = self.width;

        // Advance all streams
        for stream in &mut self.streams {
            stream.head_y += stream.speed;

            // Mutate a random character in the trail occasionally
            stream.mutate_counter = stream.mutate_counter.saturating_sub(1);
            if stream.mutate_counter == 0 {
                let idx = rng.random_range(0..stream.chars.len());
                // If this is an easter egg stream, sometimes inject the next letter
                if let Some(ref mut egg) = stream.easter_egg {
                    if egg.char_index < egg.word.len() {
                        let c = egg.word.as_bytes()[egg.char_index] as char;
                        // Place the easter egg character near the head
                        let place = rng.random_range(0..stream.chars.len().min(3).max(1));
                        stream.chars[place] = c;
                        egg.char_index += 1;
                    }
                } else {
                    stream.chars[idx] = MATRIX_CHARS[rng.random_range(0..MATRIX_CHARS.len())];
                }
                stream.mutate_counter = rng.random_range(3..15);
            }

            // Mark inactive if fully off screen
            if stream.head_y > (height as f64 + stream.length as f64 + 5.0) {
                stream.active = false;
            }
        }

        // Remove dead streams
        self.streams.retain(|s| s.active);

        // Respawn streams on columns that have no active stream
        let mut col_has_stream = vec![false; width as usize];
        for stream in &self.streams {
            if (stream.col as usize) < col_has_stream.len() {
                col_has_stream[stream.col as usize] = true;
            }
        }
        for col in 0..width {
            if !col_has_stream[col as usize] && rng.random_range(0..100) < 15 {
                self.spawn_stream(col, false);
            }
        }

        // Occasionally spawn extra streams on existing columns for density
        if rng.random_range(0..100) < 8 {
            let col = rng.random_range(0..width);
            self.spawn_stream(col, false);
        }

        // Flash message logic
        if let Some(ref mut flash) = self.flash_message {
            flash.ticks_remaining = flash.ticks_remaining.saturating_sub(1);
            if flash.ticks_remaining == 0 {
                match flash.fade_stage {
                    0 => {
                        flash.fade_stage = 1;
                        flash.ticks_remaining = 8;
                    }
                    1 => {
                        flash.fade_stage = 2;
                        flash.ticks_remaining = 6;
                    }
                    _ => {}
                }
            }
        }
        if self
            .flash_message
            .as_ref()
            .is_some_and(|f| f.fade_stage >= 2 && f.ticks_remaining == 0)
        {
            self.flash_message = None;
        }

        // Small chance to trigger a new flash message
        if self.flash_message.is_none() && rng.random_range(0..800) < 1 {
            let text = EASTER_EGGS[rng.random_range(0..EASTER_EGGS.len())];
            let text_len = text.len() as u16;
            if width > text_len + 2 {
                let start_col = rng.random_range(0..(width - text_len));
                let row = rng.random_range(2..height.saturating_sub(2).max(3));
                self.flash_message = Some(FlashMessage {
                    text,
                    row,
                    start_col,
                    ticks_remaining: 15,
                    fade_stage: 0,
                });
            }
        }
    }

    fn resize(&mut self, width: u16, height: u16) {
        self.width = width;
        self.height = height;
        // Drop streams that are now off-screen horizontally
        self.streams.retain(|s| s.col < width);
        // Clamp trail lengths so they don't wildly exceed the new height
        for stream in &mut self.streams {
            if stream.length > height.max(8).min(40) {
                stream.length = height.max(8).min(40);
            }
        }
        // Kill flash message if it no longer fits
        if let Some(ref flash) = self.flash_message {
            if flash.row >= height || flash.start_col + flash.text.len() as u16 > width {
                self.flash_message = None;
            }
        }
        // Fill any new/empty columns
        self.populate_streams();
    }
}

impl Widget for &MatrixRain {
    fn render(self, area: Rect, buf: &mut Buffer) {
        let mut rng = rand::rng();

        // Render each stream
        for stream in &self.streams {
            let head_y = stream.head_y as i32;
            let col = stream.col;
            if col >= area.width {
                continue;
            }

            for i in 0..=stream.length as i32 {
                let y = head_y - i;
                if y < 0 || y >= area.height as i32 {
                    continue;
                }

                let char_idx = (i as usize) % stream.chars.len();
                let ch = if i == 0 {
                    // Head character: sometimes mutate for that flickering effect
                    if rng.random_range(0..3) == 0 {
                        MATRIX_CHARS[rng.random_range(0..MATRIX_CHARS.len())]
                    } else {
                        stream.chars[0]
                    }
                } else {
                    stream.chars[char_idx]
                };

                let color = if i == 0 {
                    // Bright white head
                    Color::Rgb(220, 255, 220)
                } else if i <= 2 {
                    // Near-head: bright green
                    Color::Rgb(0, 255, 65)
                } else {
                    // Fade to darker green based on distance from head
                    let ratio = i as f64 / stream.length as f64;
                    let g = (200.0 * (1.0 - ratio * 0.8)) as u8;
                    let r = (20.0 * (1.0 - ratio)) as u8;
                    Color::Rgb(r, g, 0)
                };

                let cell = &mut buf[(area.x + col, area.y + y as u16)];
                cell.set_char(ch);
                cell.set_fg(color);
            }
        }

        // Render flash message on top
        if let Some(ref flash) = self.flash_message {
            let color = match flash.fade_stage {
                0 => Color::Rgb(180, 255, 180),
                1 => Color::Rgb(80, 180, 80),
                _ => Color::Rgb(30, 90, 30),
            };

            for (i, ch) in flash.text.chars().enumerate() {
                let col = flash.start_col + i as u16;
                if col < area.width && flash.row < area.height {
                    let cell = &mut buf[(area.x + col, area.y + flash.row)];
                    cell.set_char(ch);
                    cell.set_fg(color);
                }
            }
        }

        // Very rare: glitch effect - a row briefly flickers
        if rng.random_range(0..500) == 0 {
            let glitch_row = rng.random_range(0..area.height);
            let glitch_len = rng.random_range(3..area.width.min(20));
            let glitch_start = rng.random_range(0..area.width.saturating_sub(glitch_len));
            for col in glitch_start..glitch_start + glitch_len {
                if col < area.width {
                    let cell = &mut buf[(area.x + col, area.y + glitch_row)];
                    cell.set_fg(Color::Rgb(255, 255, 255));
                }
            }
        }
    }
}

fn main() -> io::Result<()> {
    // Setup terminal
    terminal::enable_raw_mode()?;
    stdout().execute(EnterAlternateScreen)?;
    stdout().execute(crossterm::cursor::Hide)?;

    let backend = CrosstermBackend::new(stdout());
    let mut terminal = Terminal::new(backend)?;

    let size = terminal.size()?;
    let mut rain = MatrixRain::new(size.width, size.height);

    let tick_rate = Duration::from_millis(50); // ~20 FPS
    let mut last_tick = Instant::now();

    loop {
        terminal.draw(|frame| {
            frame.render_widget(&rain, frame.area());
        })?;

        let timeout = tick_rate.saturating_sub(last_tick.elapsed());
        if event::poll(timeout)? {
            match event::read()? {
                Event::Key(key) if key.kind == KeyEventKind::Press => match key.code {
                    KeyCode::Char('q') | KeyCode::Esc => break,
                    KeyCode::Char('c')
                        if key
                            .modifiers
                            .contains(crossterm::event::KeyModifiers::CONTROL) =>
                    {
                        break
                    }
                    _ => {}
                },
                Event::Resize(w, h) => rain.resize(w, h),
                _ => {}
            }
        }

        if last_tick.elapsed() >= tick_rate {
            rain.tick();
            last_tick = Instant::now();
        }
    }

    // Restore terminal
    stdout().execute(crossterm::cursor::Show)?;
    stdout().execute(LeaveAlternateScreen)?;
    terminal::disable_raw_mode()?;

    Ok(())
}
