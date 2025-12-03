#!/bin/bash
# Test script for validating telemetry configuration
# This script demonstrates how to use telemetry features for debugging

set -e

echo "=========================================="
echo "Scarab Telemetry Test Script"
echo "=========================================="
echo ""

# Colors for output
GREEN='\033[0;32m'
BLUE='\033[0;34m'
YELLOW='\033[1;33m'
NC='\033[0m' # No Color

echo -e "${BLUE}Test 1: Default configuration (no telemetry)${NC}"
echo "Running: cargo check -p scarab-daemon"
cargo check -p scarab-daemon --quiet
echo -e "${GREEN}✓ Build successful${NC}"
echo ""

echo -e "${BLUE}Test 2: Config tests${NC}"
echo "Running: cargo test -p scarab-config --quiet"
cargo test -p scarab-config --quiet
echo -e "${GREEN}✓ All config tests pass${NC}"
echo ""

echo -e "${BLUE}Test 3: Environment variable configuration${NC}"
echo ""
echo -e "${YELLOW}Example: Enable FPS logging every 5 seconds${NC}"
echo "  SCARAB_LOG_FPS=5 cargo run -p scarab-daemon"
echo ""
echo -e "${YELLOW}Example: Enable pane lifecycle logging${NC}"
echo "  SCARAB_LOG_PANES=1 cargo run -p scarab-daemon"
echo ""
echo -e "${YELLOW}Example: Enable sequence number tracking${NC}"
echo "  RUST_LOG=debug SCARAB_LOG_SEQUENCE=1 cargo run -p scarab-daemon"
echo ""
echo -e "${YELLOW}Example: Enable all telemetry${NC}"
echo "  RUST_LOG=debug SCARAB_LOG_FPS=5 SCARAB_LOG_SEQUENCE=1 SCARAB_LOG_PANES=1 cargo run -p scarab-daemon"
echo ""

echo -e "${BLUE}Test 4: Configuration file examples${NC}"
echo ""
echo "Available example configs:"
ls -1 examples/fusabi-config/*.fsx | while read file; do
    echo "  - $(basename $file)"
done
echo ""

echo -e "${BLUE}Test 5: Documentation${NC}"
echo ""
echo "Telemetry documentation: docs/TELEMETRY.md"
if [ -f "docs/TELEMETRY.md" ]; then
    echo -e "${GREEN}✓ Documentation exists${NC}"
    wc -l docs/TELEMETRY.md | awk '{print "  Lines:", $1}'
else
    echo -e "${YELLOW}⚠ Documentation not found${NC}"
fi
echo ""

echo "=========================================="
echo -e "${GREEN}All telemetry tests completed!${NC}"
echo "=========================================="
echo ""
echo "Next steps:"
echo "1. Copy example config: cp examples/fusabi-config/telemetry.fsx ~/.config/scarab/config.fsx"
echo "2. Run daemon with telemetry: RUST_LOG=info cargo run -p scarab-daemon"
echo "3. View documentation: cat docs/TELEMETRY.md"
echo ""
