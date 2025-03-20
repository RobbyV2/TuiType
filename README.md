# TuiType

A terminal-based typing test application similar to MonkeyType, built with Rust and the Ratatui library.

## Features

### Typing Modes
- Words Per Minute (WPM) Test
- Accuracy Test
- Time-based Tests (30-seconds, 1-minute, 5-minute)
- Custom Tests (user-defined text)

### Customization Options
- Theme Selection (light, dark, sepia, matrix, ocean)
- Text Difficulty Levels (easy, medium, hard)

### User Interface
- Clean and Minimalistic Design
- Real-time Display of Typing Metrics (WPM, accuracy, time remaining)
- Progress Bar/Graph
- Easy keyboard navigation (tab + enter to restart test)

### Progress Tracking
- Detailed Statistics (average WPM, best WPM, accuracy over time)
- Progress visualization with charts

### Feedback and Corrections
- Immediate Feedback on Mistakes
- Error Highlighting

## Installation

### From Source

1. Clone the repository:
```
git clone https://github.com/RobbyV2/tuitype.git
cd tuitype
```

2. Build the application:
```
cargo build --release
```

3. Run the application:
```
cargo run --release
```

## WebAssembly Support

TuiType can be compiled to WebAssembly for running in a browser or other WASI-compatible environments.

### Building for WASI

```
cargo build --target wasm32-wasi --release
```

### Building for Web with wasm-bindgen

```
cargo build --target wasm32-unknown-unknown --release
wasm-bindgen --target web --out-dir ./web/pkg ./target/wasm32-unknown-unknown/release/tuitype.wasm
```

## Usage

- Use keyboard to type the displayed text
- Press `Tab` to restart a test
- Press `Esc` to exit the application

## Controls

- Type the text displayed on the screen
- `Backspace` to delete characters
- `Tab` to restart test (after completion)
- `Esc` to exit

## Configuration

TuiType saves configuration in your system's config directory:
- Windows: `%APPDATA%\tuitype\config.json`
- macOS: `~/Library/Application Support/tuitype/config.json`
- Linux: `~/.config/tuitype/config.json`

## License

MIT