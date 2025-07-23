# TuiType Project Task Runner
# Usage: just <command>
# Run `just --list` to see all available commands

# Default recipe - show available commands
default:
    @just --list

# Build the project in debug mode
build:
    cargo build

# Build the project in release mode  
build-release:
    cargo build --release

# Run the application
run *args:
    cargo run {{ args }}

# Run tests
test:
    cargo test

# Check the project (faster than build, just checks for errors)
check:
    cargo check

# Run clippy linter
clippy:
    cargo clippy

# Format code with rustfmt
fmt:
    cargo fmt

# Check if code is formatted correctly
fmt-check:
    cargo fmt -- --check

# Clean build artifacts
clean:
    cargo clean

# Auto-fix linting issues where possible
fix:
    cargo fix --allow-dirty --allow-staged
    cargo clippy --fix --allow-dirty --allow-staged

# Install the binary to ~/.cargo/bin
install:
    cargo install --path .

# Run all checks (useful for CI)
ci: fmt-check check clippy test

# Update dependencies
update:
    cargo update

# Show cargo tree of dependencies
deps:
    cargo tree

# Build for multiple platforms (consolidated from build_release.sh)
build-multi:
    #!/usr/bin/env bash
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
    cross build --release --target x86_64-unknown-linux-gnu
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
      cat > "$RELEASE_DIR/wasm-web/index.html" <<'EOL'
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

# Show project info
info:
    @echo "TuiType - Terminal-based typing test application"
    @echo "Version: $(grep '^version =' Cargo.toml | cut -d '"' -f 2)"
    @echo "Build targets available via build-multi:"
    @echo "  - Linux x86_64"
    @echo "  - macOS x86_64" 
    @echo "  - macOS ARM (Apple Silicon)"
    @echo "  - Windows x86_64"
    @echo "  - WebAssembly (WASI)"
    @echo "  - WebAssembly (Web)"

# Development workflow - format, check, test
dev: fmt check test

# Release workflow - all checks plus release build
release: ci build-release

# Build a .deb package from the binary
build-deb:
    #!/usr/bin/env bash
    set -e
    
    # Get version from Cargo.toml
    VERSION=$(grep '^version =' Cargo.toml | cut -d '"' -f 2)
    PACKAGE_NAME="tuitype"
    DEB_DIR="./dist/deb"
    PACKAGE_DIR="$DEB_DIR/${PACKAGE_NAME}_${VERSION}_amd64"
    
    echo "Building .deb package for tuitype v$VERSION"
    
    # Clean up previous builds
    rm -rf "$DEB_DIR"
    
    # Create directory structure
    mkdir -p "$PACKAGE_DIR/DEBIAN"
    mkdir -p "$PACKAGE_DIR/usr/bin"
    mkdir -p "$PACKAGE_DIR/usr/share/doc/$PACKAGE_NAME"
    mkdir -p "$PACKAGE_DIR/usr/share/man/man1"
    
    # Build the binary
    echo "Building release binary..."
    cargo build --release
    
    # Copy binary
    cp target/release/tuitype "$PACKAGE_DIR/usr/bin/"
    chmod 755 "$PACKAGE_DIR/usr/bin/tuitype"
    
    # Create control file
    cat > "$PACKAGE_DIR/DEBIAN/control" <<EOF
    Package: $PACKAGE_NAME
    Version: $VERSION
    Section: games
    Priority: optional
    Architecture: amd64
    Depends: libc6 (>= 2.17)
    Maintainer: RobbyV2 <robbyv2@example.com>
    Description: A terminal-based typing test application
     TuiType is a terminal-based typing test application similar to MonkeyType.
     It provides an interactive typing experience directly in your terminal
     with customizable settings and detailed statistics.
    Homepage: https://github.com/RobbyV2/TuiType
    EOF
    
    # Create postinst script
    cat > "$PACKAGE_DIR/DEBIAN/postinst" <<'EOF'
    #!/bin/bash
    set -e
    
    # Update man database if available
    if command -v mandb > /dev/null 2>&1; then
        mandb -q || true
    fi
    
    echo "TuiType has been installed successfully!"
    echo "Run 'tuitype' to start the application."
    
    exit 0
    EOF
    chmod 755 "$PACKAGE_DIR/DEBIAN/postinst"
    
    # Create prerm script
    cat > "$PACKAGE_DIR/DEBIAN/prerm" <<'EOF'
    #!/bin/bash
    set -e
    
    # Nothing to do before removal
    exit 0
    EOF
    chmod 755 "$PACKAGE_DIR/DEBIAN/prerm"
    
    # Create postrm script
    cat > "$PACKAGE_DIR/DEBIAN/postrm" <<'EOF'
    #!/bin/bash
    set -e
    
    case "$1" in
        remove|purge)
            # Remove any user config directories if purging
            if [ "$1" = "purge" ]; then
                echo "Purging TuiType configuration files..."
                rm -rf /home/*/.config/tuitype || true
                rm -rf /root/.config/tuitype || true
            fi
            ;;
        upgrade|failed-upgrade|abort-install|abort-upgrade|disappear)
            # Nothing to do
            ;;
        *)
            echo "postrm called with unknown argument \`$1'" >&2
            exit 1
            ;;
    esac
    
    exit 0
    EOF
    chmod 755 "$PACKAGE_DIR/DEBIAN/postrm"
    
    # Create copyright file
    cat > "$PACKAGE_DIR/usr/share/doc/$PACKAGE_NAME/copyright" <<EOF
    Format: https://www.debian.org/doc/packaging-manuals/copyright-format/1.0/
    Upstream-Name: TuiType
    Upstream-Contact: RobbyV2 <robbyv2@example.com>
    Source: https://github.com/RobbyV2/TuiType
    
    Files: *
    Copyright: 2024 RobbyV2
    License: MIT
    
    License: MIT
     Permission is hereby granted, free of charge, to any person obtaining a copy
     of this software and associated documentation files (the "Software"), to deal
     in the Software without restriction, including without limitation the rights
     to use, copy, modify, merge, publish, distribute, sublicense, and/or sell
     copies of the Software, and to permit persons to whom the Software is
     furnished to do so, subject to the following conditions:
     .
     The above copyright notice and this permission notice shall be included in all
     copies or substantial portions of the Software.
     .
     THE SOFTWARE IS PROVIDED "AS IS", WITHOUT WARRANTY OF ANY KIND, EXPRESS OR
     IMPLIED, INCLUDING BUT NOT LIMITED TO THE WARRANTIES OF MERCHANTABILITY,
     FITNESS FOR A PARTICULAR PURPOSE AND NONINFRINGEMENT. IN NO EVENT SHALL THE
     AUTHORS OR COPYRIGHT HOLDERS BE LIABLE FOR ANY CLAIM, DAMAGES OR OTHER
     LIABILITY, WHETHER IN AN ACTION OF CONTRACT, TORT OR OTHERWISE, ARISING FROM,
     OUT OF OR IN CONNECTION WITH THE SOFTWARE OR THE USE OR OTHER DEALINGS IN THE
     SOFTWARE.
    EOF
    
    # Create simple man page
    cat > "$PACKAGE_DIR/usr/share/man/man1/tuitype.1" <<'EOF'
    .TH TUITYPE 1 "2024" "TuiType" "User Commands"
    .SH NAME
    tuitype \- terminal-based typing test application
    .SH SYNOPSIS
    .B tuitype
    .SH DESCRIPTION
    TuiType is a terminal-based typing test application similar to MonkeyType.
    It provides an interactive typing experience directly in your terminal
    with customizable settings and detailed statistics.
    .SH OPTIONS
    No command line options are currently supported.
    .SH FILES
    .TP
    .I ~/.config/tuitype/
    User configuration directory
    .SH AUTHOR
    Written by RobbyV2.
    .SH "REPORTING BUGS"
    Report bugs to: https://github.com/RobbyV2/TuiType/issues
    .SH COPYRIGHT
    Copyright \(co 2024 RobbyV2.
    License MIT: <https://opensource.org/licenses/MIT>.
    This is free software: you are free to change and redistribute it.
    There is NO WARRANTY, to the extent permitted by law.
    EOF
    
    # Compress man page
    gzip -9n "$PACKAGE_DIR/usr/share/man/man1/tuitype.1"
    
    # Create changelog
    cat > "$PACKAGE_DIR/usr/share/doc/$PACKAGE_NAME/changelog.Debian.gz" <<EOF
    $PACKAGE_NAME ($VERSION) unstable; urgency=medium
    
      * New upstream release.
    
     -- RobbyV2 <robbyv2@example.com>  $(date -R)
    EOF
    gzip -9n "$PACKAGE_DIR/usr/share/doc/$PACKAGE_NAME/changelog.Debian.gz"
    
    # Build the .deb package
    echo "Building .deb package..."
    dpkg-deb --build "$PACKAGE_DIR"
    
    # Move to releases directory
    mkdir -p ./releases
    mv "$DEB_DIR/${PACKAGE_NAME}_${VERSION}_amd64.deb" "./releases/"
    
    echo "Successfully created: ./releases/${PACKAGE_NAME}_${VERSION}_amd64.deb"
    echo ""
    echo "To install: sudo dpkg -i ./releases/${PACKAGE_NAME}_${VERSION}_amd64.deb"
    echo "To remove:  sudo apt remove $PACKAGE_NAME"
    echo "To purge:   sudo apt purge $PACKAGE_NAME"

# Quick check - just build and clippy (fastest feedback)
quick: check clippy