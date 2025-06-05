#!/bin/bash

# Test script for Cleansys terminal resize functionality
# This script tests the application with various terminal sizes

set -e

echo "🧪 Testing Cleansys Terminal Resize Functionality"
echo "================================================="

# Colors for output
RED='\033[0;31m'
GREEN='\033[0;32m'
YELLOW='\033[1;33m'
BLUE='\033[0;34m'
NC='\033[0m' # No Color

# Function to print colored output
print_status() {
    echo -e "${BLUE}[INFO]${NC} $1"
}

print_success() {
    echo -e "${GREEN}[SUCCESS]${NC} $1"
}

print_warning() {
    echo -e "${YELLOW}[WARNING]${NC} $1"
}

print_error() {
    echo -e "${RED}[ERROR]${NC} $1"
}

# Check if tmux is available
if ! command -v tmux &> /dev/null; then
    print_error "tmux is required for this test script but not installed."
    print_status "Install tmux with: sudo apt install tmux (Ubuntu/Debian) or brew install tmux (macOS)"
    exit 1
fi

# Build the application
print_status "Building Cleansys..."
if cargo build --release; then
    print_success "Build completed successfully"
else
    print_error "Build failed"
    exit 1
fi

# Test configurations: [width, height, description]
declare -a test_configs=(
    "60:15:Very Small Terminal"
    "80:20:Small Terminal"
    "100:30:Medium Terminal"
    "120:40:Large Terminal"
    "150:50:Very Large Terminal"
    "40:10:Extremely Narrow"
    "200:60:Ultra Wide"
)

print_status "Starting resize tests..."

# Create a tmux session for testing
SESSION_NAME="cleansys_resize_test"

# Kill existing session if it exists
tmux kill-session -t "$SESSION_NAME" 2>/dev/null || true

for config in "${test_configs[@]}"; do
    IFS=':' read -r width height description <<< "$config"
    
    print_status "Testing: $description (${width}x${height})"
    
    # Create new tmux session with specific size
    tmux new-session -d -s "$SESSION_NAME" -x "$width" -y "$height"
    
    # Send command to run cleansys
    tmux send-keys -t "$SESSION_NAME" "cd $(pwd) && ./target/release/cleansys" Enter
    
    # Wait a moment for the app to start
    sleep 2
    
    # Send 'q' to quit the application
    tmux send-keys -t "$SESSION_NAME" 'q'
    
    # Wait for clean exit
    sleep 1
    
    # Kill the session
    tmux kill-session -t "$SESSION_NAME" 2>/dev/null || true
    
    print_success "Completed test: $description"
done

print_status "Testing dynamic resize behavior..."

# Test dynamic resizing
print_status "Creating session for dynamic resize test..."
tmux new-session -d -s "$SESSION_NAME" -x 100 -y 30

# Start the application
tmux send-keys -t "$SESSION_NAME" "cd $(pwd) && ./target/release/cleansys" Enter
sleep 2

print_status "Resizing window dynamically..."

# Resize to small
tmux resize-window -t "$SESSION_NAME" -x 60 -y 20
sleep 2
print_success "Resized to 60x20"

# Resize to large
tmux resize-window -t "$SESSION_NAME" -x 150 -y 45
sleep 2
print_success "Resized to 150x45"

# Resize to very narrow
tmux resize-window -t "$SESSION_NAME" -x 40 -y 25
sleep 2
print_success "Resized to 40x25"

# Resize back to normal
tmux resize-window -t "$SESSION_NAME" -x 100 -y 30
sleep 2
print_success "Resized back to 100x30"

# Quit the application
tmux send-keys -t "$SESSION_NAME" 'q'
sleep 1

# Clean up
tmux kill-session -t "$SESSION_NAME" 2>/dev/null || true

print_success "All resize tests completed!"
print_status "Manual testing recommendations:"
echo "  1. Run ./target/release/cleansys in your terminal"
echo "  2. Manually resize your terminal window"
echo "  3. Verify that:"
echo "     - The cleanup distribution chart appears/disappears appropriately"
echo "     - Layout adjusts to terminal width and height"
echo "     - Text truncates properly on narrow terminals"
echo "     - Terminal dimensions are shown for very small windows"
echo "     - No UI elements are cut off or overlapping"

print_warning "Note: If you see any layout issues, the responsive design may need further adjustment."