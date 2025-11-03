# Guitar Tuner

A beautiful terminal-based guitar tuner application written in Rust. Tune your guitar by musical note with real-time frequency detection, visual tuning indicators, and customizable A4 reference frequency.

## Features

- **Real-time Pitch Detection**: Uses FFT-based frequency analysis for accurate pitch detection
- **Note-based Tuning**: Tune to any musical note (A, A#, B, C, C#, D, D#, E, F, F#, G, G#) across multiple octaves
- **Customizable A4 Frequency**: Adjust the reference frequency from 432 Hz to 450 Hz
- **Beautiful Terminal UI**: Colorful interface with rounded borders and smooth animations
- **Circular Tuning Indicator**: Visual arc gauge showing tuning accuracy with color-coded feedback
- **Cross-platform Audio Support**: Works on Linux, macOS, and Windows via cpal

## Installation

### Prerequisites

- Rust toolchain (1.70 or later)
- Audio input device (microphone or audio interface)

### Building from Source

```bash
git clone <repository-url>
cd rust_project_guitar
cargo build --release
```

The binary will be available at `target/release/guitar-tuner`.

## Usage

Run the application:

```bash
cargo run --release
```

Or use the compiled binary:

```bash
./target/release/guitar-tuner
```

### Controls

- **← / →**: Navigate between notes (A, A#, B, C, etc.)
- **↑ / ↓**: Change target octave (0-8)
- **+ / -**: Adjust A4 reference frequency (432-450 Hz)
- **ESC**: Exit the application

### How to Tune

1. Select the target note you want to tune to using the arrow keys
2. Optionally adjust the octave if needed
3. Optionally set your preferred A4 reference frequency (default: 440 Hz)
4. Play the corresponding string on your guitar
5. Watch the circular indicator:
   - **Green**: Perfectly in tune (±5 cents)
   - **Yellow**: Close (±20 cents)
   - **Red**: Out of tune (>20 cents)
   - **Gray**: No signal detected

The frequency display shows:
- Current detected frequency in Hz
- Detected note name and octave
- Deviation from target in cents

## Technical Details

- **Sample Rate**: 44100 Hz (or device default)
- **FFT Size**: 4096 samples for optimal frequency resolution
- **Window Function**: Hann window for reduced spectral leakage
- **Frequency Range**: Detects frequencies from 20 Hz to 5000 Hz
- **Update Rate**: ~60 FPS for smooth visual feedback

## Dependencies

- `cpal`: Cross-platform audio I/O
- `ratatui`: Terminal UI framework
- `crossterm`: Terminal manipulation
- `rustfft`: FFT implementation for pitch detection
- `crossbeam-channel`: Inter-thread communication
- `hann`: Window function

## License

This project is open source and available for personal and educational use.

---

"예수께서 대답하여 이르시되 기록되었으되 사람이 떡으로만 살 것이 아니요 하나님의 입으로부터 나오는 모든 말씀으로 살 것이라 하였느니라 하시니"

(마태복음 4:4)

