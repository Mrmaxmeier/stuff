use std::io;
use std::io::prelude::*;
use std::time::Duration;
use term;
use term_size;

const SPINNER: [&str; 8] = ["⡆", "⠇", "⠋", "⠙", "⠸", "⢰", "⣠", "⣄"];

pub struct ProgressBar {
    sleep: Duration,
    term: Box<term::Terminal<Output = io::Stdout>>,
    tick: usize,
    term_size: usize,
    pub current: Duration,
}

impl ProgressBar {
    pub fn new(sleep_threshold: Duration) -> ProgressBar {
        let term_size = if let Some((w, _)) = term_size::dimensions() {
            w
        } else {
            80
        };
        ProgressBar {
            tick: 0,
            sleep: sleep_threshold,
            term: term::stdout().unwrap(),
            current: Duration::from_secs(0),
            term_size,
        }
    }

    pub fn render(&mut self) -> Result<(), io::Error> {
        self.tick = (self.tick + 1) % SPINNER.len();

        if self.tick == 0 {
            if let Some((w, _)) = term_size::dimensions() {
                self.term_size = w;
            }
        }

        let s = format!(
            "[{}] {} / {}",
            SPINNER[self.tick],
            self.current.as_secs(),
            self.sleep.as_secs()
        );

        write!(self.term, "\n{} [", s)?;
        let total = self.term_size - s.len() - 1;
        let a = (self.current.as_secs() as f64 / self.sleep.as_secs() as f64).min(1.0);
        for _ in 0..(total as f64 * a) as usize {
            write!(self.term, "#")?
        }
        for _ in 0..total - (total as f64 * a) as usize {
            write!(self.term, " ")?
        }
        write!(self.term, "]")?;
        self.term.cursor_up()?;
        self.term.carriage_return()?;
        self.term.get_mut().flush()
    }
}
