#!/bin/bash

# Test script for verifying the new "stay on details screen" behavior
# This tests that users remain on the cleaning details screen after operations complete

set -e

# Colors for output
RED='\033[0;31m'
GREEN='\033[0;32m'
YELLOW='\033[1;33m'
BLUE='\033[0;34m'
CYAN='\033[0;36m'
WHITE='\033[1;37m'
NC='\033[0m' # No Color

# Session name for tmux
SESSION_NAME="cleansys_stay_details_test"

# Helper functions
print_header() {
    echo -e "\n${CYAN}=================================${NC}"
    echo -e "${CYAN}$1${NC}"
    echo -e "${CYAN}=================================${NC}\n"
}

print_test() {
    echo -e "${YELLOW}🧪 Test:${NC} $1"
}

print_status() {
    echo -e "${BLUE}📋 Status:${NC} $1"
}

print_success() {
    echo -e "${GREEN}✅ Success:${NC} $1"
}

print_instruction() {
    echo -e "${WHITE}📝 Instruction:${NC} $1"
}

print_feature() {
    echo -e "${GREEN}  ✓${NC} $1"
}

# Start test
print_header "Testing Stay On Details Screen Behavior"

echo -e "${WHITE}This test verifies the new behavior where:${NC}"
print_feature "Users stay on the cleaning details screen after operations complete"
print_feature "ESC key returns to main menu only when manually pressed"
print_feature "All navigation controls work on the completed details screen"
print_feature "Footer shows appropriate controls for completed state"

echo

print_test "Setting up test environment..."

# Clean up any existing session
tmux kill-session -t "$SESSION_NAME" 2>/dev/null || true

print_test "Starting Cleansys in tmux session for testing..."

# Start cleansys in a tmux session
tmux new-session -d -s "$SESSION_NAME" -x 120 -y 30
tmux send-keys -t "$SESSION_NAME" "cd $(pwd) && ./target/debug/cleansys" Enter

# Wait for application to start
sleep 2

print_status "Cleansys should now be running in tmux session '$SESSION_NAME'"

echo
print_instruction "Manual Test Steps:"
echo "1. Connect to the test session:"
echo "   ${YELLOW}tmux attach-session -t $SESSION_NAME${NC}"
echo
echo "2. Select some cleaners:"
echo "   • Use Space to select 2-3 user cleaners"
echo "   • Press Enter to start cleaning"
echo
echo "3. Observe the cleaning process:"
echo "   • Operations should start and progress automatically"
echo "   • Watch status change from READY → CLEANING → FINISHED"
echo "   • Notice the detailed list showing cleaned items"
echo
echo "4. Verify post-completion behavior:"
echo "   • Operations complete but screen stays on details view"
echo "   • Footer should show 'ESC: Return to Menu' instead of 'ESC: Cancel'"
echo "   • Status should show 'FINISHED' instead of 'CLEANING'"
echo "   • All navigation should still work (↑/↓, j/k, PgUp/PgDn)"
echo
echo "5. Test manual return to menu:"
echo "   • Press ESC - should return to main menu"
echo "   • Verify you're back at the category/cleaner selection screen"
echo
echo "6. Exit:"
echo "   • Press 'q' to quit the application"
echo "   • Exit the tmux session with 'exit' or Ctrl+D"

echo
print_status "=== EXPECTED BEHAVIOR ==="
echo "✓ Operations complete automatically"
echo "✓ User stays on details screen after completion"
echo "✓ Footer changes to show 'Return to Menu' instead of 'Cancel'"
echo "✓ Status shows 'FINISHED' when operations are done"
echo "✓ Navigation works normally on completed details screen"
echo "✓ ESC manually returns to main menu when ready"
echo "✓ No automatic return to main menu"

echo
print_status "=== KEY CHANGES IMPLEMENTED ==="
echo "• Added 'show_progress_screen' flag to App struct"
echo "• Modified UI logic to show progress screen when flag is true"
echo "• Updated ESC key handling for completed operations"
echo "• Modified footer controls for completed state"
echo "• Updated navigation controls to work on completed screen"
echo "• Enhanced help text to clarify ESC behavior"

echo
print_status "=== TECHNICAL DETAILS ==="
echo "• is_running: Controls active operations (true during cleaning)"
echo "• show_progress_screen: Controls UI display (stays true after completion)"
echo "• ESC behavior: Cancel → Return to Menu → Quit (context-dependent)"
echo "• Footer dynamically shows appropriate controls"
echo "• All scroll/navigation controls work in completed state"

echo
echo -e "${GREEN}🚀 Ready to test!${NC}"
echo -e "Run: ${YELLOW}tmux attach-session -t $SESSION_NAME${NC}"
echo
echo -e "${BLUE}To clean up after testing:${NC}"
echo -e "  ${YELLOW}tmux kill-session -t $SESSION_NAME${NC}"

echo
echo -e "${WHITE}💡 Test Tips:${NC}"
echo "• Try different navigation keys to verify they all work"
echo "• Check that the footer text changes appropriately"
echo "• Verify ESC only returns to menu when you want it to"
echo "• Test with both user and system cleaners if available"
echo "• Check that completion message includes ESC instruction"