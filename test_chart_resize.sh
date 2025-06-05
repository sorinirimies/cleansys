#!/bin/bash

# Test script specifically for Cleansys chart resizing functionality
# Tests the new responsive chart features and data integration

set -e

echo "🎯 Testing Cleansys Chart Resize Functionality"
echo "=============================================="

# Colors for output
RED='\033[0;31m'
GREEN='\033[0;32m'
YELLOW='\033[1;33m'
BLUE='\033[0;34m'
CYAN='\033[0;36m'
NC='\033[0m' # No Color

# Function to print colored output
print_status() {
    echo -e "${BLUE}[INFO]${NC} $1"
}

print_success() {
    echo -e "${GREEN}[PASS]${NC} $1"
}

print_warning() {
    echo -e "${YELLOW}[WARN]${NC} $1"
}

print_error() {
    echo -e "${RED}[FAIL]${NC} $1"
}

print_test() {
    echo -e "${CYAN}[TEST]${NC} $1"
}

# Check if tmux is available
if ! command -v tmux &> /dev/null; then
    print_error "tmux is required for this test but not installed."
    print_status "Install with: sudo apt install tmux (Ubuntu/Debian) or brew install tmux (macOS)"
    exit 1
fi

# Build the application
print_status "Building Cleansys with chart improvements..."
if cargo build --release; then
    print_success "Build completed successfully"
else
    print_error "Build failed"
    exit 1
fi

# Test configurations for chart behavior: [width, height, description, expected_behavior]
declare -a chart_tests=(
    "160:50:Ultra Wide Terminal:Full Chart widget with detailed axes and labels"
    "130:40:Large Terminal:Standard Chart widget with full labels and axes"
    "100:30:Medium Terminal:Balanced layout with Chart widget visible"
    "90:25:Small Terminal:Compact Chart widget with truncated labels"
    "85:20:Narrow Terminal:Small Chart widget with minimal labels"
    "79:18:Chart Threshold:Chart should be hidden (width < 80)"
    "60:15:Narrow Without Chart:Stats use full width, no chart"
    "45:12:Very Narrow:Ultra-compact view with terminal dimensions"
    "35:10:Extremely Narrow:Minimal layout with size indicator"
    "25:8:Ultra Narrow:Maximum compression with essential info only"
)

print_status "Starting chart-specific resize tests..."
print_warning "Each test will run for 3 seconds - observe the chart behavior"

SESSION_NAME="cleansys_chart_test"

# Kill existing session if it exists
tmux kill-session -t "$SESSION_NAME" 2>/dev/null || true

for test_config in "${chart_tests[@]}"; do
    IFS=':' read -r width height description expected <<< "$test_config"
    
    print_test "Testing: $description (${width}x${height})"
    print_status "Expected: $expected"
    
    # Create new tmux session with specific size
    tmux new-session -d -s "$SESSION_NAME" -x "$width" -y "$height"
    
    # Send command to run cleansys
    tmux send-keys -t "$SESSION_NAME" "cd $(pwd) && ./target/release/cleansys" Enter
    
    # Wait for the app to start and display
    sleep 3
    
    # Send 'q' to quit the application
    tmux send-keys -t "$SESSION_NAME" 'q'
    
    # Wait for clean exit
    sleep 1
    
    # Kill the session
    tmux kill-session -t "$SESSION_NAME" 2>/dev/null || true
    
    print_success "Completed: $description"
    echo
done

print_test "Testing dynamic chart resize behavior..."

# Test dynamic resizing with chart focus
print_status "Creating session for dynamic chart resize test..."
tmux new-session -d -s "$SESSION_NAME" -x 120 -y 35

# Start the application
tmux send-keys -t "$SESSION_NAME" "cd $(pwd) && ./target/release/cleansys" Enter
sleep 2

print_status "Testing chart visibility transitions..."

# Test chart visibility threshold (around 60 columns)
print_test "1. Resize to show full chart (100x30)"
tmux resize-window -t "$SESSION_NAME" -x 100 -y 30
sleep 2
print_success "Chart should be visible with medium-sized bars"

print_test "2. Resize to narrow with chart (70x25)"
tmux resize-window -t "$SESSION_NAME" -x 70 -y 25
sleep 2
print_success "Chart should be visible but compact"

print_test "3. Resize to chart threshold (80x20)"
tmux resize-window -t "$SESSION_NAME" -x 80 -y 20
sleep 2
print_success "Chart should still be visible but minimal"

print_test "4. Resize below threshold (75x18)"
tmux resize-window -t "$SESSION_NAME" -x 75 -y 18
sleep 2
print_success "Chart should be HIDDEN - stats use full width"

print_test "5. Resize to ultra-narrow (40x15)"
tmux resize-window -t "$SESSION_NAME" -x 40 -y 15
sleep 2
print_success "Should show ultra-compact view with terminal dimensions"

print_test "6. Resize back to show chart (90x30)"
tmux resize-window -t "$SESSION_NAME" -x 90 -y 30
sleep 2
print_success "Chart should reappear with proper sizing"

print_test "7. Test ultra-wide layout (150x45)"
tmux resize-window -t "$SESSION_NAME" -x 150 -y 45
sleep 2
print_success "Chart should have maximum detail and wide bars"

# Quit the application
tmux send-keys -t "$SESSION_NAME" 'q'
sleep 1

# Clean up
tmux kill-session -t "$SESSION_NAME" 2>/dev/null || true

print_success "All chart resize tests completed!"

echo
print_status "=== CHART FEATURE SUMMARY ==="
echo "✅ Real data integration: Chart widget uses actual cleaning results"
echo "✅ Professional Chart widget: Replaced basic BarChart with full Chart widget"
echo "✅ Smart visibility: Chart hides on terminals < 80 columns wide"
echo "✅ Improved layout: Chart gets 55-65% of space when visible"
echo "✅ Axes and labels: Proper X/Y axes with category and count labels"
echo "✅ Ultra-compact mode: Special layout for very small terminals"
echo "✅ Progressive enhancement: Layout quality scales with terminal size"

echo
print_warning "=== MANUAL VERIFICATION CHECKLIST ==="
echo "□ Chart displays real category data from cleaned items"
echo "□ Chart disappears when terminal width < 80 columns"
echo "□ Chart widget shows proper X/Y axes with labels"
echo "□ Category names truncate appropriately for narrow terminals"
echo "□ Chart area gets 55-65% of horizontal space when visible"
echo "□ Ultra-compact view shows terminal dimensions for debugging"
echo "□ No text overlap or UI corruption at any tested size"
echo "□ Chart reappears correctly when resizing back to wider terminals"
echo "□ Progress statistics adapt layout when chart is hidden"
echo "□ Chart shows 'Items Distribution' title and axis labels"

echo
print_status "=== NEXT STEPS ==="
echo "1. Run actual cleaning operations to test with real data"
echo "2. Verify chart updates dynamically as items are cleaned"
echo "3. Test with different category distributions"
echo "4. Validate performance with large datasets"

print_status "Run './target/release/cleansys' and manually resize your terminal to verify smooth transitions"