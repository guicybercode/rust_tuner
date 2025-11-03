mod audio;
mod tuner;
mod ui;

use audio::AudioCapture;
use cpal::SampleRate;
use crossterm::event::{self, DisableMouseCapture, EnableMouseCapture, Event, KeyCode, KeyEventKind};
use crossterm::execute;
use crossterm::terminal::{disable_raw_mode, enable_raw_mode, EnterAlternateScreen, LeaveAlternateScreen};
use ratatui::backend::CrosstermBackend;
use ratatui::Terminal;
use std::io;
use crossbeam_channel;
use std::thread;
use std::time::Duration;
use tuner::Tuner;
use ui::{render_ui, UiState};

const NOTES: [&str; 12] = ["A", "A#", "B", "C", "C#", "D", "D#", "E", "F", "F#", "G", "G#"];

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let mut terminal = init_terminal()?;

    let audio_capture = AudioCapture::new()?;
    let sample_rate = audio_capture.sample_rate();
    let (tx, rx) = crossbeam_channel::unbounded();

    let stream = audio_capture.start_capture(SampleRate(sample_rate), tx)?;

    let mut tuner = Tuner::new(sample_rate);
    let mut ui_state = UiState::new();
    let mut audio_buffer: Vec<f32> = Vec::new();

    loop {
        terminal.draw(|f| render_ui(f, &ui_state))?;

        if event::poll(Duration::from_millis(16))? {
            if let Event::Key(key) = event::read()? {
                if key.kind == KeyEventKind::Press {
                    match key.code {
                        KeyCode::Esc => break,
                        KeyCode::Left => {
                            let current_idx = NOTES
                                .iter()
                                .position(|&n| n == ui_state.target_note)
                                .unwrap_or(0);
                            let new_idx = (current_idx + 11) % 12;
                            ui_state.target_note = NOTES[new_idx].to_string();
                        }
                        KeyCode::Right => {
                            let current_idx = NOTES
                                .iter()
                                .position(|&n| n == ui_state.target_note)
                                .unwrap_or(0);
                            let new_idx = (current_idx + 1) % 12;
                            ui_state.target_note = NOTES[new_idx].to_string();
                        }
                        KeyCode::Up => {
                            ui_state.target_octave = (ui_state.target_octave + 1).min(8);
                        }
                        KeyCode::Down => {
                            ui_state.target_octave = (ui_state.target_octave - 1).max(0);
                        }
                        KeyCode::Char('+') | KeyCode::Char('=') => {
                            if ui_state.a4_freq < 450.0 {
                                ui_state.a4_freq = (ui_state.a4_freq + 0.1).min(450.0);
                            }
                        }
                        KeyCode::Char('-') | KeyCode::Char('_') => {
                            if ui_state.a4_freq > 432.0 {
                                ui_state.a4_freq = (ui_state.a4_freq - 0.1).max(432.0);
                            }
                        }
                        _ => {}
                    }
                }
            }
        }

        while let Ok(samples) = rx.try_recv() {
            audio_buffer.extend_from_slice(&samples);
            if audio_buffer.len() > 4096 {
                if let Some(freq) = tuner.detect_frequency(&audio_buffer) {
                    let (note, octave, _deviation_cents) =
                        tuner.frequency_to_note(freq, ui_state.a4_freq);
                    let target_freq =
                        Tuner::note_name_to_frequency(&ui_state.target_note, ui_state.target_octave, ui_state.a4_freq);
                    let target_deviation = 1200.0 * (freq / target_freq).log2();

                    ui_state.current_freq = Some(freq);
                    ui_state.current_note = Some(note);
                    ui_state.current_octave = Some(octave);
                    ui_state.deviation_cents = Some(target_deviation);
                } else {
                    ui_state.current_freq = None;
                    ui_state.current_note = None;
                    ui_state.current_octave = None;
                    ui_state.deviation_cents = None;
                }
                audio_buffer.drain(0..audio_buffer.len().saturating_sub(2048));
            }
        }

        thread::sleep(Duration::from_millis(16));
    }

    drop(stream);
    restore_terminal(terminal)?;
    Ok(())
}

fn init_terminal() -> Result<Terminal<CrosstermBackend<io::Stdout>>, Box<dyn std::error::Error>> {
    enable_raw_mode()?;
    let mut stdout = io::stdout();
    execute!(stdout, EnterAlternateScreen, EnableMouseCapture)?;
    let backend = CrosstermBackend::new(stdout);
    let terminal = Terminal::new(backend)?;
    Ok(terminal)
}

fn restore_terminal(
    mut terminal: Terminal<CrosstermBackend<io::Stdout>>,
) -> Result<(), Box<dyn std::error::Error>> {
    disable_raw_mode()?;
    execute!(
        terminal.backend_mut(),
        LeaveAlternateScreen,
        DisableMouseCapture
    )?;
    terminal.show_cursor()?;
    Ok(())
}

