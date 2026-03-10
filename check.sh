#!/bin/bash
set -e

# Colors for output
GREEN='\033[0;32m'
BLUE='\033[0;34m'
NC='\033[0m' # No Color

# Parse arguments
TARGET="${1:-.}"

# Validate target exists
if [ ! -e "$TARGET" ]; then
    echo "Error: Target path '$TARGET' does not exist"
    exit 1
fi

PROJECTS_FOUND=false

# TypeScript workspace check
if [ -f "$TARGET/ts/package.json" ]; then
    PROJECTS_FOUND=true
    echo -e "${BLUE}=== Checking TypeScript ===${NC}"
    (
        cd "$TARGET/ts"
        pnpm check
    )
    echo ""
fi

# Rust workspace check
if [ -f "$TARGET/rs/Cargo.toml" ]; then
    PROJECTS_FOUND=true
    echo -e "${BLUE}=== Checking Rust ===${NC}"
    (
        cd "$TARGET/rs"
        cargo fmt --check 2>/dev/null || cargo fmt
        cargo clippy --workspace -- -D warnings
        cargo test --workspace
    )
    echo ""
fi

# Summary
if [ "$PROJECTS_FOUND" = false ]; then
    echo "No projects found in '$TARGET'"
    exit 1
fi

echo -e "${GREEN}All checks passed${NC}"
