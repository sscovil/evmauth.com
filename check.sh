#!/bin/bash
set -e

# Colors for output
GREEN='\033[0;32m'
BLUE='\033[0;34m'
NC='\033[0m' # No Color

# Check function for TypeScript
check_typescript() {
    local dir=$1
    local rel_path="${dir#./}"
    echo -e "${BLUE}=== Checking TypeScript: $rel_path ===${NC}"
    (
        cd "$dir"
        pnpm check
    )
    echo ""
}

# Check function for Rust
check_rust() {
    local dir=$1
    local rel_path="${dir#./}"
    echo -e "${BLUE}=== Checking Rust: $rel_path ===${NC}"
    (
        cd "$dir"
        cargo fmt --check 2>/dev/null || cargo fmt
        cargo clippy -- -D warnings
        cargo test
    )
    echo ""
}

# Parse arguments
TARGET="${1:-.}"

# Validate target exists
if [ ! -e "$TARGET" ]; then
    echo "Error: Target path '$TARGET' does not exist"
    exit 1
fi

# Track if any projects were found
PROJECTS_FOUND=false

# Find and check TypeScript projects
while IFS= read -r ts_dir; do
    PROJECTS_FOUND=true
    check_typescript "$ts_dir"
done < <(find "$TARGET" -name "package.json" -not -path "*/node_modules/*" -not -path "*/.next/*" -not -path "*/build/*" -not -path "*/dist/*" -exec dirname {} \; 2>/dev/null | sort)

# Find and check Rust projects
while IFS= read -r cargo_dir; do
    PROJECTS_FOUND=true
    check_rust "$cargo_dir"
done < <(find "$TARGET" -name "Cargo.toml" -not -path "*/target/*" -not -path "*/.template/*" -exec dirname {} \; 2>/dev/null | sort)

# Summary
if [ "$PROJECTS_FOUND" = false ]; then
    echo "No projects found in '$TARGET'"
    exit 1
fi

echo -e "${GREEN}✓ All checks passed${NC}"
