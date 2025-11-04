#!/bin/bash

# Trust Construction System - Launch Script

# Get the directory where this script is located
SCRIPT_DIR="$( cd "$( dirname "${BASH_SOURCE[0]}" )" && pwd )"

# Change to the script directory
cd "$SCRIPT_DIR"

case "$1" in
    import)
        echo "🗄️  Running import..."
        cargo run --release import
        ;;
    ui|"")
        echo "🖥️  Launching UI..."
        cargo run --release
        ;;
    *)
        echo "Usage:"
        echo "  ./run.sh import  - Import transactions from CSV"
        echo "  ./run.sh ui      - Launch terminal UI (default)"
        echo "  ./run.sh         - Launch terminal UI"
        ;;
esac
