use std::sync::mpsc;
use std::thread;
use std::time::{Duration, Instant};

use anyhow::Result;
use crossterm::event::{self, Event as CrosstermEvent, KeyEvent};

pub enum Event {
    Input(KeyEvent),
    Tick,
    Resize(u16, u16),
}

/// A small event handler that wrap crossterm input and tick events
pub struct Events {
    /// The event receiver channel
    rx: mpsc::Receiver<Event>,
    /// To make sure only one instance of Events exists at a time
    _tx: mpsc::Sender<Event>,
}

impl Events {
    /// Constructs an new instance of `Events` with custom config.
    pub fn with_config(config: Config) -> Self {
        let (tx, rx) = mpsc::channel();
        let event_tx = tx.clone();
        let tick_rate = config.tick_rate;

        thread::spawn(move || {
            let mut last_tick = Instant::now();
            loop {
                // Poll for events with a timeout matching tick rate
                let timeout = tick_rate
                    .checked_sub(last_tick.elapsed())
                    .unwrap_or_else(|| Duration::from_secs(0));

                match event::poll(timeout) {
                    Ok(true) => match event::read() {
                        Ok(CrosstermEvent::Key(key)) => {
                            if event_tx.send(Event::Input(key)).is_err() {
                                // Receiver dropped (app is shutting down) —
                                // exit quietly instead of panicking on the
                                // next send.
                                break;
                            }
                        }
                        Ok(CrosstermEvent::Resize(width, height)) => {
                            if event_tx.send(Event::Resize(width, height)).is_err() {
                                break;
                            }
                        }
                        Ok(_) => {}
                        // Terminal event stream error (e.g. stdin closed) —
                        // nothing sensible to do but stop polling.
                        Err(_) => break,
                    },
                    Ok(false) => {}
                    // Polling itself failed — same as above, stop cleanly.
                    Err(_) => break,
                }

                if last_tick.elapsed() >= tick_rate {
                    if event_tx.send(Event::Tick).is_err() {
                        break;
                    }
                    last_tick = Instant::now();
                }
            }
        });

        Self { rx, _tx: tx }
    }

    /// Attempts to read an event.
    pub fn next(&self) -> Result<Event> {
        Ok(self.rx.recv()?)
    }
}

pub struct Config {
    pub tick_rate: Duration,
}

impl Default for Config {
    fn default() -> Self {
        Self {
            tick_rate: Duration::from_millis(250),
        }
    }
}
