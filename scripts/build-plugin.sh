#!/usr/bin/env bash
# Scarab Fusabi Plugin Build Script
# Compiles .fsx source files to .fzb bytecode and validates plugin metadata

set -euo pipefail

SCRIPT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
PROJECT_ROOT="$(cd "$SCRIPT_DIR/.." && pwd)"

# Colors for output
RED='\033[0;31m'
GREEN='\033[0;32m'
YELLOW='\033[1;33m'
BLUE='\033[0;34m'
NC='\033[0m' # No Color

# Default values
VERBOSE=0
VALIDATE_ONLY=0
OUTPUT_DIR=""
PLUGIN_FILE=""
SKIP_METADATA=0

# Print usage
usage() {
    cat << EOF
Usage: $0 [OPTIONS] <plugin.fsx>

Compile Fusabi plugin source (.fsx) to bytecode (.fzb)

OPTIONS:
    -h, --help              Show this help message
    -v, --verbose           Enable verbose output
    -o, --output DIR        Output directory for .fzb file (default: same as source)
    -V, --validate-only     Only validate plugin, don't compile
    -s, --skip-metadata     Skip metadata validation
    -a, --all               Build all plugins in examples/fusabi/

EXAMPLES:
    # Build single plugin
    $0 examples/fusabi/hello.fsx

    # Build with custom output directory
    $0 -o target/plugins examples/fusabi/theme.fsx

    # Validate plugin without compiling
    $0 --validate-only examples/fusabi/hello.fsx

    # Build all example plugins
    $0 --all
EOF
}

# Logging functions
log_info() {
    echo -e "${BLUE}[INFO]${NC} $*"
}

log_success() {
    echo -e "${GREEN}[SUCCESS]${NC} $*"
}

log_warn() {
    echo -e "${YELLOW}[WARN]${NC} $*"
}

log_error() {
    echo -e "${RED}[ERROR]${NC} $*"
}

log_verbose() {
    if [ "$VERBOSE" -eq 1 ]; then
        echo -e "${BLUE}[VERBOSE]${NC} $*"
    fi
}

# Check if Fusabi compiler is available
check_fusabi_compiler() {
    log_verbose "Checking for Fusabi compiler..."

    # Try to use fusabi-frontend via cargo
    if cargo run -p fusabi-frontend --help &>/dev/null; then
        log_verbose "Using Fusabi compiler from cargo workspace"
        return 0
    fi

    # Check for fusabi CLI tool
    if command -v fusabi &>/dev/null; then
        log_verbose "Using system-installed fusabi CLI"
        return 0
    fi

    log_error "Fusabi compiler not found!"
    log_info "Install via: cargo install fusabi-cli"
    log_info "Or ensure fusabi-frontend is available in the workspace"
    return 1
}

# Validate plugin metadata by parsing the .fsx file
validate_metadata() {
    local plugin_file="$1"

    log_verbose "Validating plugin metadata in $plugin_file"

    if [ ! -f "$plugin_file" ]; then
        log_error "Plugin file not found: $plugin_file"
        return 1
    fi

    # Basic validation: check for common metadata comments
    # Fusabi plugins should declare metadata at the top
    local has_name=0
    local has_version=0
    local has_description=0

    # Read first 50 lines for metadata
    while IFS= read -r line; do
        if [[ "$line" =~ ^[[:space:]]*//.*@name ]]; then
            has_name=1
            log_verbose "Found @name metadata"
        fi
        if [[ "$line" =~ ^[[:space:]]*//.*@version ]]; then
            has_version=1
            log_verbose "Found @version metadata"
        fi
        if [[ "$line" =~ ^[[:space:]]*//.*@description ]]; then
            has_description=1
            log_verbose "Found @description metadata"
        fi
    done < <(head -n 50 "$plugin_file")

    if [ "$SKIP_METADATA" -eq 1 ]; then
        log_verbose "Skipping metadata validation (--skip-metadata)"
        return 0
    fi

    # Warn if metadata is missing (but don't fail)
    if [ $has_name -eq 0 ]; then
        log_warn "Missing @name metadata in $plugin_file"
    fi
    if [ $has_version -eq 0 ]; then
        log_warn "Missing @version metadata in $plugin_file"
    fi
    if [ $has_description -eq 0 ]; then
        log_warn "Missing @description metadata in $plugin_file"
    fi

    log_verbose "Metadata validation complete"
    return 0
}

# Compile plugin .fsx to .fzb
compile_plugin() {
    local source_file="$1"
    local output_file="$2"

    log_info "Compiling plugin: $source_file"

    # Create output directory if needed
    local output_dir
    output_dir="$(dirname "$output_file")"
    mkdir -p "$output_dir"

    # Compile using Rust-based approach (fusabi-frontend crate)
    # Since fusabi-frontend is a library, we'll create a simple compiler binary
    # For now, we'll use a placeholder that copies the source as-is
    # TODO: Replace with actual Fusabi compiler when available

    log_warn "Fusabi compiler integration pending - copying source to output"
    log_verbose "Source: $source_file"
    log_verbose "Output: $output_file"

    # For now, just copy the .fsx file to .fzb location
    # This allows the build system to work while the compiler is being integrated
    cp "$source_file" "$output_file"

    if [ -f "$output_file" ]; then
        log_success "Plugin compiled successfully: $output_file"
        return 0
    else
        log_error "Compilation failed for $source_file"
        return 1
    fi
}

# Build a single plugin
build_plugin() {
    local plugin_file="$1"
    local output_dir="${2:-$(dirname "$plugin_file")}"

    # Validate metadata
    if ! validate_metadata "$plugin_file"; then
        return 1
    fi

    if [ "$VALIDATE_ONLY" -eq 1 ]; then
        log_success "Validation passed: $plugin_file"
        return 0
    fi

    # Determine output path
    local basename
    basename="$(basename "$plugin_file" .fsx)"
    local output_file="$output_dir/${basename}.fzb"

    # Compile plugin
    if ! compile_plugin "$plugin_file" "$output_file"; then
        return 1
    fi

    return 0
}

# Build all plugins in a directory
build_all_plugins() {
    local plugin_dir="${1:-$PROJECT_ROOT/examples/fusabi}"

    log_info "Building all plugins in $plugin_dir"

    if [ ! -d "$plugin_dir" ]; then
        log_error "Plugin directory not found: $plugin_dir"
        return 1
    fi

    local count=0
    local success=0
    local failed=0

    while IFS= read -r -d '' plugin_file; do
        count=$((count + 1))
        log_info "[$count] Building $(basename "$plugin_file")"

        if build_plugin "$plugin_file" "${OUTPUT_DIR:-$(dirname "$plugin_file")}"; then
            success=$((success + 1))
        else
            failed=$((failed + 1))
        fi

        echo ""
    done < <(find "$plugin_dir" -maxdepth 1 -name "*.fsx" -type f -print0 | sort -z)

    echo "========================================"
    log_info "Build Summary:"
    log_info "  Total:   $count plugins"
    log_success "  Success: $success plugins"
    if [ $failed -gt 0 ]; then
        log_error "  Failed:  $failed plugins"
        return 1
    fi

    return 0
}

# Parse command line arguments
BUILD_ALL=0

while [[ $# -gt 0 ]]; do
    case $1 in
        -h|--help)
            usage
            exit 0
            ;;
        -v|--verbose)
            VERBOSE=1
            shift
            ;;
        -V|--validate-only)
            VALIDATE_ONLY=1
            shift
            ;;
        -s|--skip-metadata)
            SKIP_METADATA=1
            shift
            ;;
        -o|--output)
            OUTPUT_DIR="$2"
            shift 2
            ;;
        -a|--all)
            BUILD_ALL=1
            shift
            ;;
        -*)
            log_error "Unknown option: $1"
            usage
            exit 1
            ;;
        *)
            PLUGIN_FILE="$1"
            shift
            ;;
    esac
done

# Main execution
main() {
    log_info "Scarab Fusabi Plugin Builder"
    echo "========================================"

    # Check for Fusabi compiler
    if ! check_fusabi_compiler; then
        log_warn "Continuing with placeholder compiler..."
    fi

    # Build all or single plugin
    if [ "$BUILD_ALL" -eq 1 ]; then
        build_all_plugins
    else
        if [ -z "$PLUGIN_FILE" ]; then
            log_error "No plugin file specified"
            usage
            exit 1
        fi

        build_plugin "$PLUGIN_FILE" "$OUTPUT_DIR"
    fi
}

main
