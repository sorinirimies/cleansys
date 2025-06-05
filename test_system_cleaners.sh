#!/bin/bash

# Test script for system cleaners in cleansys
# This script tests the fixed system cleaner functionality

echo "🔧 Testing System Cleaners - Progress Tracking Fix"
echo "=================================================="
echo ""

# Check if we're running as root
if [ "$EUID" -eq 0 ]; then
    echo "✅ Running as root - system cleaners will work"
    ROOT_STATUS="✅ ROOT"
else
    echo "⚠️  Running as regular user - system cleaners will show sudo requirement"
    ROOT_STATUS="👤 USER"
fi

echo "Current user status: $ROOT_STATUS"
echo ""

# Check if cleansys binary exists
if [ ! -f "target/debug/cleansys" ] && [ ! -f "target/release/cleansys" ]; then
    echo "Building cleansys..."
    cargo build
    if [ $? -ne 0 ]; then
        echo "❌ Build failed!"
        exit 1
    fi
fi

# Determine which binary to use
if [ -f "target/release/cleansys" ]; then
    CLEANSYS_BIN="target/release/cleansys"
else
    CLEANSYS_BIN="target/debug/cleansys"
fi

echo "🧪 Testing System Cleaner Progress Tracking:"
echo ""
echo "What to expect:"
echo "  1. Start cleansys TUI"
echo "  2. Navigate to 'System Cleaners' category (Tab key)"
echo "  3. Select one or more system cleaners (Space key)"
echo "  4. Press Enter to start cleaning"
echo "  5. Watch the 'Operation Progress' window (bottom section)"
echo "  6. See detailed logs like:"
echo "     🔄 Executing: Package Manager Caches"
echo "     ✅ Completed Package Manager Caches: 15.2 MB freed"
echo "  7. Timer should show realistic progress and stop when complete"
echo "  8. Status should change from CLEANING to FINISHED"
echo "  9. Total freed bytes should accumulate correctly"
echo ""

if [ "$EUID" -eq 0 ]; then
    echo "💡 Root User Tips:"
    echo "  • System cleaners will actually run (real cleaning)"
    echo "  • You'll see actual package manager commands executed"
    echo "  • Real bytes will be freed from your system"
    echo "  • Progress logs will show real command outputs"
else
    echo "💡 Regular User Tips:"
    echo "  • System cleaners will show 'Requires sudo' messages"
    echo "  • You'll see helpful guidance about running 'sudo cleansys'"
    echo "  • Progress logs will explain why operations failed"
    echo "  • User cleaners will still work normally"
fi

echo ""
echo "🎯 Key Features to Test:"
echo "  ✓ Detailed progress logging in bottom window"
echo "  ✓ Realistic operation timing (not instant)"
echo "  ✓ Proper sudo requirement handling"
echo "  ✓ Timer stops at completion"
echo "  ✓ Status changes to FINISHED"
echo "  ✓ Total bytes accumulate correctly"
echo ""

echo "🚀 Starting cleansys..."
echo "Press 'q' to quit when done testing"
echo ""
echo "Press Enter to continue..."
read

# Run cleansys
$CLEANSYS_BIN

echo ""
echo "✅ System cleaner test completed!"
echo ""
echo "📊 Results Summary:"
echo "  • Progress tracking: Should show detailed logs during operations"
echo "  • Timer behavior: Should start/run/stop properly"
echo "  • Status display: Should show READY → CLEANING → FINISHED"
echo "  • Sudo handling: Should show clear messages for permission requirements"
echo "  • Byte accumulation: Should show realistic totals, not 0"
echo ""
echo "🔧 If you saw issues:"
echo "  • Check that progress logs appeared in bottom window"
echo "  • Verify timer stopped when operations completed"
echo "  • Confirm status changed to FINISHED"
echo "  • Look for detailed operation messages"
echo ""