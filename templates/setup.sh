#!/bin/bash
# SuperHarness Setup Script
# Run: ./setup.sh

set -e

# Colors
GREEN='\033[32m'
YELLOW='\033[33m'
CYAN='\033[36m'
RESET='\033[0m'

# Default values
PROVIDER="${PROVIDER:-anthropic}"
API_KEY="${API_KEY:-}"
FORCE="${FORCE:-}"

# Functions
header() {
    echo ""
    echo -e "${CYAN}========================================${RESET}"
    echo -e "${CYAN}  $1${RESET}"
    echo -e "${CYAN}========================================${RESET}"
    echo ""
}

success() {
    echo -e "${GREEN}[OK]${RESET} $1"
}

info() {
    echo -e "${YELLOW}[INFO]${RESET} $1"
}

# Main
header "SuperHarness Configuration Setup"

# 1. Create config directory
CONFIG_DIR="$HOME/.superharness"
if [ ! -d "$CONFIG_DIR" ]; then
    mkdir -p "$CONFIG_DIR"
    success "Created config directory: $CONFIG_DIR"
else
    info "Config directory exists: $CONFIG_DIR"
fi

# 2. Copy config template
CONFIG_FILE="$CONFIG_DIR/config.toml"
SCRIPT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
TEMPLATE_FILE="$SCRIPT_DIR/config.toml"

if [ -f "$CONFIG_FILE" ] && [ -z "$FORCE" ]; then
    info "Config file exists: $CONFIG_FILE"
    info "Use FORCE=1 to overwrite"
else
    if [ -f "$TEMPLATE_FILE" ]; then
        cp "$TEMPLATE_FILE" "$CONFIG_FILE"
        success "Copied config template to: $CONFIG_FILE"
    else
        info "Template not found, creating default config"
    fi
fi

# 3. Set environment variables
header "Environment Variables"

if [ -n "$API_KEY" ]; then
    export SUPERHARNESS_API_KEY="$API_KEY"
    export SUPERHARNESS_PROVIDER="$PROVIDER"
    
    # Add to shell config
    SHELL_RC=""
    if [ -f "$HOME/.bashrc" ]; then
        SHELL_RC="$HOME/.bashrc"
    elif [ -f "$HOME/.zshrc" ]; then
        SHELL_RC="$HOME/.zshrc"
    fi
    
    if [ -n "$SHELL_RC" ]; then
        echo "" >> "$SHELL_RC"
        echo "# SuperHarness configuration" >> "$SHELL_RC"
        echo "export SUPERHARNESS_API_KEY='$API_KEY'" >> "$SHELL_RC"
        echo "export SUPERHARNESS_PROVIDER='$PROVIDER'" >> "$SHELL_RC"
        success "Added environment variables to $SHELL_RC"
    fi
else
    info "No API key provided. Set it manually:"
    echo ""
    echo "  export SUPERHARNESS_API_KEY='your-api-key'"
    echo "  export SUPERHARNESS_PROVIDER='anthropic'"
    echo ""
fi

# 4. Verify
header "Verification"
echo "Config directory: $CONFIG_DIR"
echo "Config file: $CONFIG_FILE"
echo "Provider: $PROVIDER"

# 5. Test
header "Quick Test"
echo "Testing Python SDK..."
python3 -c "from superharness_sdk import Agent, Config; c = Config.from_default(); print(f'Config: {c}')"

header "Setup Complete!"
echo "Next steps:"
echo "  1. Set your API key:"
echo "     export SUPERHARNESS_API_KEY='your-key'"
echo ""
echo "  2. Run SuperHarness:"
echo "     superharness run 'your task'"
