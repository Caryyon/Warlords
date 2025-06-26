#!/bin/bash
# Build the game quietly
echo "Building Warlords..."
cargo build --bin warlords --quiet 2>/dev/null

# Check if build succeeded
if [ $? -eq 0 ]; then
    # Clear the screen before running
    clear
    # Run the game
    ./target/debug/warlords
else
    echo "Build failed. Running with full output:"
    cargo build --bin warlords
fi