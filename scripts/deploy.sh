#!/usr/bin/env bash
# ============================================
# MediGate - Soroban Contract Deploy Script
# ============================================
# Deploy the AccessLog contract to Stellar testnet.
#
# Prerequisites:
#   1. Install Stellar CLI: https://soroban.stellar.org/docs/getting-started/setup
#   2. Have a Stellar testnet account funded with friendbot
#      (or use an existing identity)
#
# Usage:
#   chmod +x deploy.sh
#   ./deploy.sh                              # Deploy with default identity
#   ./deploy.sh --identity my-wallet          # Deploy with a specific identity
#   ./deploy.sh --network mainnet             # Deploy to mainnet (not recommended)
#
# Output:
#   Prints the deployed contract ID to stdout.
#   Saves contract IDs to ../.contract-ids file.
# ============================================

set -euo pipefail

# Defaults
NETWORK="testnet"
IDENTITY="medigate-dev"
SCRIPT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
CONTRACTS_DIR="$(dirname "$SCRIPT_DIR")"
OUTPUT_FILE="$CONTRACTS_DIR/.contract-ids"

# Colors for output
GREEN='\033[0;32m'
YELLOW='\033[1;33m'
BLUE='\033[0;34m'
RED='\033[0;31m'
NC='\033[0m' # No Color

# Parse arguments
while [[ $# -gt 0 ]]; do
  case $1 in
    --identity)
      IDENTITY="$2"
      shift 2
      ;;
    --network)
      NETWORK="$2"
      shift 2
      ;;
    --help)
      echo "Usage: $0 [--identity <name>] [--network <testnet|mainnet>]"
      exit 0
      ;;
    *)
      echo -e "${RED}Unknown option: $1${NC}"
      exit 1
      ;;
  esac
done

echo -e "${BLUE}============================================${NC}"
echo -e "${BLUE}  MediGate Soroban Contract Deployer${NC}"
echo -e "${BLUE}============================================${NC}"
echo ""
echo -e "  Network:   ${YELLOW}$NETWORK${NC}"
echo -e "  Identity:  ${YELLOW}$IDENTITY${NC}"
echo ""

# Check prerequisites
if ! command -v stellar &> /dev/null; then
    echo -e "${RED}Error: Stellar CLI is not installed.${NC}"
    echo "Install it from: https://soroban.stellar.org/docs/getting-started/setup"
    exit 1
fi

# Check if identity exists (for mainnet, this is critical)
if ! stellar keys ls 2>/dev/null | grep -q "$IDENTITY"; then
    echo -e "${YELLOW}Identity '$IDENTITY' not found. Creating a new one...${NC}"
    stellar keys generate "$IDENTITY"
    echo -e "${GREEN}✓ Identity '$IDENTITY' created${NC}"
fi

# Fund the identity on testnet
if [ "$NETWORK" = "testnet" ]; then
    echo -e "${YELLOW}Funding identity '$IDENTITY' on testnet...${NC}"
    ADDRESS=$(stellar keys public-key "$IDENTITY")
    curl -s "https://friendbot.stellar.org?addr=$ADDRESS" > /dev/null
    echo -e "${GREEN}✓ Funded $ADDRESS${NC}"
fi

echo ""
echo -e "${BLUE}--- Building contracts ---${NC}"
cd "$CONTRACTS_DIR"

# Build all contracts (with wasm compat flag for Rust >=1.82)
echo -e "${YELLOW}Building with WASM compatibility flags...${NC}"
RUSTFLAGS="-Ctarget-feature=-reference-types" cargo build --target wasm32-unknown-unknown --release 2>&1 | tail -5

# Check if wasm-opt is available for WASM optimization
WASM_OPT=""
if command -v wasm-opt &> /dev/null; then
    WASM_OPT="wasm-opt"
    echo -e "${GREEN}✓ Using wasm-opt for WASM optimization${NC}"
else
    echo -e "${YELLOW}⚠️  wasm-opt not found. Install binaryen for WASM optimization.${NC}"
fi

echo ""
echo -e "${BLUE}--- Deploying AccessLog Contract ---${NC}"

# Deploy the AccessLog contract
ACCESS_LOG_WASM="target/wasm32-unknown-unknown/release/access_log.wasm"
if [ ! -f "$ACCESS_LOG_WASM" ]; then
    echo -e "${RED}Error: WASM file not found at $ACCESS_LOG_WASM${NC}"
    echo "Make sure the contract compiled successfully."
    exit 1
fi

# Optimize WASM if wasm-opt is available
if [ -n "$WASM_OPT" ]; then
    ACCESS_LOG_WASM="/tmp/access_log_opt.wasm"
    wasm-opt -O3 --strip-producers "target/wasm32-unknown-unknown/release/access_log.wasm" -o "$ACCESS_LOG_WASM"
fi

ACCESS_LOG_ID=$(stellar contract deploy \
    --wasm "$ACCESS_LOG_WASM" \
    --network "$NETWORK" \
    --source "$IDENTITY" 2>&1 | grep -E '^C[0-9A-Z]{55}' | head -1)

echo -e "${GREEN}✓ AccessLog deployed at: $ACCESS_LOG_ID${NC}"

# Save contract IDs
echo "# MediGate Contract IDs ($NETWORK)" > "$OUTPUT_FILE"
echo "# Deployed at: $(date -u +"%Y-%m-%dT%H:%M:%SZ")" >> "$OUTPUT_FILE"
echo "ACCESS_LOG_CONTRACT_ID=$ACCESS_LOG_ID" >> "$OUTPUT_FILE"
echo "" >> "$OUTPUT_FILE"

echo ""
echo -e "${GREEN}============================================${NC}"
echo -e "${GREEN}  Deployment Complete!${NC}"
echo -e "${GREEN}============================================${NC}"
echo ""
echo -e "  AccessLog Contract ID: ${BLUE}$ACCESS_LOG_ID${NC}"
echo ""
echo -e "  Contract IDs saved to: ${YELLOW}$OUTPUT_FILE${NC}"
echo ""
echo -e "  View on Stellar Testnet Explorer:"
echo -e "  https://stellar.expert/explorer/testnet/contract/$ACCESS_LOG_ID"
echo ""
