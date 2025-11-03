use ratatui::{
    layout::{Alignment, Constraint, Direction, Layout, Rect},
    style::{Color, Modifier, Style},
    text::{Line, Span},
    widgets::{Block, Borders, Paragraph, Widget},
    Frame,
};

pub struct UiState {
    pub current_freq: Option<f32>,
    pub current_note: Option<String>,
    pub current_octave: Option<i32>,
    pub deviation_cents: Option<f32>,
    pub target_note: String,
    pub target_octave: i32,
    pub a4_freq: f32,
}

impl UiState {
    pub fn new() -> Self {
        Self {
            current_freq: None,
            current_note: None,
            current_octave: None,
            deviation_cents: None,
            target_note: "A".to_string(),
            target_octave: 4,
            a4_freq: 440.0,
        }
    }

    pub fn get_tuning_status(&self) -> TuningStatus {
        if let Some(deviation) = self.deviation_cents {
            if deviation.abs() < 5.0 {
                TuningStatus::Perfect
            } else if deviation.abs() < 20.0 {
                TuningStatus::Close
            } else {
                TuningStatus::Far
            }
        } else {
            TuningStatus::NoSignal
        }
    }
}

pub enum TuningStatus {
    Perfect,
    Close,
    Far,
    NoSignal,
}

pub fn render_ui(frame: &mut Frame, state: &UiState) {
    let size = frame.size();
    let vertical = Layout::default()
        .direction(Direction::Vertical)
        .constraints([
            Constraint::Length(3),
            Constraint::Min(10),
            Constraint::Length(5),
            Constraint::Length(3),
            Constraint::Length(3),
        ])
        .split(size);

    let title_block = Block::default()
        .borders(Borders::ALL)
        .border_style(Style::default().fg(Color::Cyan))
        .title("Guitar Tuner")
        .title_alignment(Alignment::Center);

    let title_text = Line::from(vec![
        Span::styled("🎸 ", Style::default().fg(Color::Yellow)),
        Span::styled("Guitar Tuner", Style::default().fg(Color::Cyan).add_modifier(Modifier::BOLD)),
    ]);

    Paragraph::new(title_text)
        .block(title_block)
        .alignment(Alignment::Center)
        .render(vertical[0], frame.buffer_mut());

    render_tuning_indicator(frame, state, vertical[1]);
    render_frequency_display(frame, state, vertical[2]);
    render_target_note_selector(frame, state, vertical[3]);
    render_controls(frame, vertical[4]);
}

fn render_tuning_indicator(frame: &mut Frame, state: &UiState, area: Rect) {
    let status = state.get_tuning_status();

    let (color, symbol, text) = match status {
        TuningStatus::Perfect => (Color::Green, "●", "IN TUNE"),
        TuningStatus::Close => (Color::Yellow, "◐", "CLOSE"),
        TuningStatus::Far => (Color::Red, "◑", "OUT OF TUNE"),
        TuningStatus::NoSignal => (Color::DarkGray, "○", "NO SIGNAL"),
    };

    let center_x = area.x + area.width / 2;
    let center_y = area.y + area.height / 2;
    let radius = (area.width.min(area.height) / 2 - 2) as i32;

    let indicator_area = Rect::new(
        center_x.saturating_sub(radius as u16),
        center_y.saturating_sub(radius as u16),
        ((radius * 2) as u16).min(area.width),
        ((radius * 2) as u16).min(area.height),
    );

    let block = Block::default()
        .borders(Borders::ALL)
        .border_style(Style::default().fg(color))
        .title("Tuning Indicator")
        .title_alignment(Alignment::Center);

    frame.render_widget(block, area);

    if let Some(deviation) = state.deviation_cents {
        let normalized_deviation = (deviation / 50.0).clamp(-1.0, 1.0);
        let angle = (normalized_deviation * std::f32::consts::PI / 2.0) + std::f32::consts::PI / 2.0;
        let needle_length = (radius - 1) as f32 * 0.8;
        let end_x = center_x as f32 + angle.cos() * needle_length;
        let end_y = center_y as f32 - angle.sin() * needle_length;

        let buffer = frame.buffer_mut();
        let steps = (needle_length as u16).max(1);
        for i in 0..=steps {
            let t = i as f32 / steps as f32;
            let x = (center_x as f32 + (end_x - center_x as f32) * t) as u16;
            let y_pos = (center_y as f32 - (center_y as f32 - end_y) * t) as u16;
            if x < area.width && y_pos < area.height {
                let cell = buffer.get_mut(x + area.x, y_pos + area.y);
                cell.set_char('│');
                cell.set_fg(color);
            }
        }

        for i in 0..20 {
            let angle = (i as f32 / 20.0) * std::f32::consts::PI;
            let x = (center_x as f32 + angle.cos() * radius as f32) as u16;
            let y = (center_y as f32 - angle.sin() * radius as f32) as u16;
            if x < indicator_area.width && y < indicator_area.height {
                let cell = buffer.get_mut(x + indicator_area.x, y + indicator_area.y);
                if i == 10 {
                    cell.set_char('─');
                    cell.set_fg(Color::Green);
                } else {
                    cell.set_char('·');
                    cell.set_fg(Color::DarkGray);
                }
            }
        }
    }

    let text_area = Rect::new(
        area.x + 2,
        area.y + area.height.saturating_sub(2),
        area.width.saturating_sub(4),
        1,
    );

    let text_line = Line::from(vec![
        Span::styled(symbol, Style::default().fg(color)),
        Span::raw(" "),
        Span::styled(text, Style::default().fg(color).add_modifier(Modifier::BOLD)),
    ]);

    Paragraph::new(text_line)
        .alignment(Alignment::Center)
        .render(text_area, frame.buffer_mut());
}

fn render_frequency_display(frame: &mut Frame, state: &UiState, area: Rect) {
    let block = Block::default()
        .borders(Borders::ALL)
        .border_style(Style::default().fg(Color::Magenta))
        .title("Frequency")
        .title_alignment(Alignment::Center);

    let freq_text = if let Some(freq) = state.current_freq {
        format!("{:.2} Hz", freq)
    } else {
        "--- Hz".to_string()
    };

    let note_text = if let (Some(note), Some(octave)) = (state.current_note.as_ref(), state.current_octave) {
        format!("{}{}", note, octave)
    } else {
        "---".to_string()
    };

    let deviation_text = if let Some(dev) = state.deviation_cents {
        if dev.abs() < 0.1 {
            "±0.0 cents".to_string()
        } else if dev > 0.0 {
            format!("+{:.1} cents", dev)
        } else {
            format!("{:.1} cents", dev)
        }
    } else {
        "---".to_string()
    };

    let text = Line::from(vec![
        Span::styled(freq_text, Style::default().fg(Color::Yellow)),
        Span::raw(" | "),
        Span::styled(note_text, Style::default().fg(Color::Cyan).add_modifier(Modifier::BOLD)),
        Span::raw(" | "),
        Span::styled(deviation_text, Style::default().fg(Color::Green)),
    ]);

    Paragraph::new(text)
        .block(block)
        .alignment(Alignment::Center)
        .render(area, frame.buffer_mut());
}

fn render_target_note_selector(frame: &mut Frame, state: &UiState, area: Rect) {
    let block = Block::default()
        .borders(Borders::ALL)
        .border_style(Style::default().fg(Color::Blue))
        .title("Target")
        .title_alignment(Alignment::Center);

    let text = Line::from(vec![
        Span::styled("Target: ", Style::default().fg(Color::White)),
        Span::styled(
            format!("{}{}", state.target_note, state.target_octave),
            Style::default().fg(Color::Yellow).add_modifier(Modifier::BOLD),
        ),
        Span::raw(" | "),
        Span::styled("A4: ", Style::default().fg(Color::White)),
        Span::styled(
            format!("{:.1} Hz", state.a4_freq),
            Style::default().fg(Color::Cyan),
        ),
    ]);

    Paragraph::new(text)
        .block(block)
        .alignment(Alignment::Center)
        .render(area, frame.buffer_mut());
}

fn render_controls(frame: &mut Frame, area: Rect) {
    let block = Block::default()
        .borders(Borders::ALL)
        .border_style(Style::default().fg(Color::DarkGray))
        .title("Controls")
        .title_alignment(Alignment::Center);

    let controls_text = Line::from(vec![
        Span::styled("←/→", Style::default().fg(Color::Yellow)),
        Span::raw(" Note | "),
        Span::styled("↑/↓", Style::default().fg(Color::Yellow)),
        Span::raw(" Octave | "),
        Span::styled("+/-", Style::default().fg(Color::Yellow)),
        Span::raw(" A4 Freq | "),
        Span::styled("ESC", Style::default().fg(Color::Red)),
        Span::raw(" Quit"),
    ]);

    Paragraph::new(controls_text)
        .block(block)
        .alignment(Alignment::Center)
        .render(area, frame.buffer_mut());
}

