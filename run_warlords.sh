#!/bin/bash

# Script to run Warlords with proper terminal setup
echo "🎮 Starting Warlords..."
echo "======================================"
echo "🚀 Setting up terminal environment..."

# Ensure we have a proper terminal
export TERM=${TERM:-xterm-256color}

# Check if running in a proper terminal
if [ ! -t 0 ]; then
    echo "❌ Error: Not running in a proper terminal!"
    echo "Please run this script from:"
    echo "  - Terminal.app on macOS"
    echo "  - A Linux terminal"
    echo "  - Windows Terminal"
    echo "  - NOT from an IDE terminal"
    exit 1
fi

echo "✅ Terminal check passed!"
echo "🎯 Launching Warlords..."
echo ""

# Build and run the game
cargo run --bin warlords 2>&1

# Capture exit code
EXIT_CODE=$?

echo ""
echo "======================================"
if [ $EXIT_CODE -eq 0 ]; then
    echo "✅ Game exited successfully!"
else
    echo "❌ Game exited with error code: $EXIT_CODE"
fi

exit $EXIT_CODE