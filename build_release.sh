#!/bin/bash
# Build script for tuitype - builds for multiple platforms

# Exit immediately if a command exits with a non-zero status
set -e

# Build directory
RELEASE_DIR="./releases"
mkdir -p "$RELEASE_DIR"

# Output version
VERSION=$(grep '^version =' Cargo.toml | cut -d '"' -f 2)
echo "Building tuitype v$VERSION"

# Ensure all dependencies are available
echo "Updating dependencies..."
cargo update

# --------------------------------
# Native platform builds
# --------------------------------

# Linux x86_64
echo "Building for Linux (x86_64)..."
cargo build --release --target x86_64-unknown-linux-gnu
mkdir -p "$RELEASE_DIR/linux-x86_64"
cp target/x86_64-unknown-linux-gnu/release/tuitype "$RELEASE_DIR/linux-x86_64/"
tar -czf "$RELEASE_DIR/tuitype-v$VERSION-linux-x86_64.tar.gz" -C "$RELEASE_DIR/linux-x86_64" tuitype

# macOS x86_64
echo "Building for macOS (x86_64)..."
cargo build --release --target x86_64-apple-darwin
mkdir -p "$RELEASE_DIR/macos-x86_64"
cp target/x86_64-apple-darwin/release/tuitype "$RELEASE_DIR/macos-x86_64/"
tar -czf "$RELEASE_DIR/tuitype-v$VERSION-macos-x86_64.tar.gz" -C "$RELEASE_DIR/macos-x86_64" tuitype

# macOS ARM (Apple Silicon)
echo "Building for macOS (ARM)..."
cargo build --release --target aarch64-apple-darwin
mkdir -p "$RELEASE_DIR/macos-arm"
cp target/aarch64-apple-darwin/release/tuitype "$RELEASE_DIR/macos-arm/"
tar -czf "$RELEASE_DIR/tuitype-v$VERSION-macos-arm.tar.gz" -C "$RELEASE_DIR/macos-arm" tuitype

# Windows x86_64
echo "Building for Windows (x86_64)..."
cargo build --release --target x86_64-pc-windows-msvc
mkdir -p "$RELEASE_DIR/windows-x86_64"
cp target/x86_64-pc-windows-msvc/release/tuitype.exe "$RELEASE_DIR/windows-x86_64/"
pushd "$RELEASE_DIR/windows-x86_64"
zip -r "../tuitype-v$VERSION-windows-x86_64.zip" tuitype.exe
popd

# --------------------------------
# WebAssembly builds
# --------------------------------

# WASI (WebAssembly System Interface)
echo "Building for WASI (WebAssembly)..."
cargo build --release --target wasm32-wasi
mkdir -p "$RELEASE_DIR/wasm-wasi"
cp target/wasm32-wasi/release/tuitype.wasm "$RELEASE_DIR/wasm-wasi/"
tar -czf "$RELEASE_DIR/tuitype-v$VERSION-wasi.tar.gz" -C "$RELEASE_DIR/wasm-wasi" tuitype.wasm

# Web (using wasm-bindgen)
echo "Building for Web (WebAssembly)..."
cargo build --release --target wasm32-unknown-unknown

# Check if wasm-bindgen-cli is installed
if command -v wasm-bindgen &> /dev/null; then
  mkdir -p "$RELEASE_DIR/wasm-web"
  wasm-bindgen --target web \
    --out-dir "$RELEASE_DIR/wasm-web" \
    ./target/wasm32-unknown-unknown/release/tuitype.wasm
  
  # Create simple HTML example
  cat > "$RELEASE_DIR/wasm-web/index.html" <<EOL
<!DOCTYPE html>
<html>
<head>
  <meta charset="UTF-8">
  <title>TuiType - Web Demo</title>
  <style>
    body { font-family: sans-serif; max-width: 800px; margin: 0 auto; padding: 20px; }
    #tuitype-container { border: 1px solid #ccc; padding: 10px; }
    .typing-text { font-family: monospace; font-size: 18px; line-height: 1.5; }
    .stats { display: flex; gap: 20px; margin-top: 20px; }
    .stat-box { border: 1px solid #ccc; padding: 10px; flex: 1; }
  </style>
</head>
<body>
  <h1>TuiType - Web Demo</h1>
  
  <div id="tuitype-container">
    <div class="typing-text" id="text-display"></div>
    <div class="stats">
      <div class="stat-box">WPM: <span id="wpm">0</span></div>
      <div class="stat-box">Accuracy: <span id="accuracy">100%</span></div>
      <div class="stat-box">Time: <span id="time">0s</span></div>
    </div>
  </div>
  
  <script type="module">
    import init, { WasmTuiType } from './tuitype.js';
    
    async function run() {
      await init();
      const tuitype = new WasmTuiType();
      
      // Initialize UI elements
      const textDisplay = document.getElementById('text-display');
      const wpmElement = document.getElementById('wpm');
      const accuracyElement = document.getElementById('accuracy');
      const timeElement = document.getElementById('time');
      
      // Display text
      textDisplay.textContent = tuitype.text();
      
      // Update stats
      setInterval(() => {
        wpmElement.textContent = tuitype.wpm().toFixed(1);
        accuracyElement.textContent = tuitype.accuracy().toFixed(1) + '%';
      }, 1000);
      
      // Handle keypresses
      document.addEventListener('keydown', (event) => {
        if (event.key.length === 1 || event.key === 'Backspace' || 
            event.key === 'Tab' || event.key === 'Enter') {
          tuitype.keypress(event.key);
          textDisplay.textContent = tuitype.text();
        }
      });
    }
    
    run();
  </script>
</body>
</html>
EOL
  
  # Create a zip with web files
  pushd "$RELEASE_DIR/wasm-web"
  zip -r "../tuitype-v$VERSION-web.zip" *
  popd
else
  echo "Warning: wasm-bindgen-cli not found, skipping Web build packaging"
  echo "Install with: cargo install wasm-bindgen-cli"
fi

echo "All builds completed!"
echo "Release files are available in the $RELEASE_DIR directory:"
ls -la "$RELEASE_DIR"/*.{tar.gz,zip}