#!/bin/bash
################################################################################
# ProvChain-Org Benchmark Toolkit - Packaging Script
################################################################################
# Creates a distributable tarball of the entire benchmark toolkit
# Can be deployed on any machine with Docker installed
################################################################################

set -e

# Colors
GREEN='\033[0;32m'
BLUE='\033[0;34m'
YELLOW='\033[1;33m'
NC='\033[0m'

TOOLKIT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
VERSION="1.0.0"
DATE=$(date +%Y%m%d)
PACKAGE_NAME="provchain-benchmark-toolkit-v${VERSION}-${DATE}"
OUTPUT_DIR="${TOOLKIT_DIR}/../dist"

echo -e "${BLUE}╔═══════════════════════════════════════════════════════════════╗${NC}"
echo -e "${BLUE}║  ProvChain-Org Benchmark Toolkit - Package Creator           ║${NC}"
echo -e "${BLUE}╚═══════════════════════════════════════════════════════════════╝${NC}"
echo ""

# Create output directory
mkdir -p "$OUTPUT_DIR"

# Create temporary directory for packaging
TEMP_DIR="/tmp/${PACKAGE_NAME}"
rm -rf "$TEMP_DIR"
mkdir -p "$TEMP_DIR"

echo -e "${YELLOW}Step 1: Copying toolkit files...${NC}"

# Copy all toolkit files
cp -r "$TOOLKIT_DIR"/* "$TEMP_DIR/"

# Remove package script itself from package
rm -f "$TEMP_DIR/package.sh"

# Remove unnecessary files
rm -rf "$TEMP_DIR/results"/*
rm -rf "$TEMP_DIR/logs"/*

echo -e "${GREEN}✓ Files copied${NC}"

echo -e "${YELLOW}Step 2: Creating distribution package...${NC}"

# Create tarball
cd /tmp
tar -czf "${OUTPUT_DIR}/${PACKAGE_NAME}.tar.gz" "${PACKAGE_NAME}"

# Calculate checksum
cd "$OUTPUT_DIR"
sha256sum "${PACKAGE_NAME}.tar.gz" > "${PACKAGE_NAME}.tar.gz.sha256"

# Get file size
SIZE=$(du -h "${PACKAGE_NAME}.tar.gz" | cut -f1)

echo -e "${GREEN}✓ Package created${NC}"

# Clean up temp directory
rm -rf "$TEMP_DIR"

echo ""
echo -e "${BLUE}╔═══════════════════════════════════════════════════════════════╗${NC}"
echo -e "${BLUE}║                    Package Complete!                         ║${NC}"
echo -e "${BLUE}╚═══════════════════════════════════════════════════════════════╝${NC}"
echo ""
echo -e "${GREEN}Package Location:${NC} ${OUTPUT_DIR}/${PACKAGE_NAME}.tar.gz"
echo -e "${GREEN}Package Size:${NC}    ${SIZE}"
echo -e "${GREEN}Checksum:${NC}        ${PACKAGE_NAME}.tar.gz.sha256"
echo ""
echo -e "${YELLOW}To deploy on another machine:${NC}"
echo ""
echo "  1. Copy the tarball to target machine:"
echo "     scp ${PACKAGE_NAME}.tar.gz user@server:/path/to/"
echo ""
echo "  2. Extract on target machine:"
echo "     tar -xzf ${PACKAGE_NAME}.tar.gz"
echo "     cd ${PACKAGE_NAME}"
echo ""
echo "  3. Run benchmark:"
echo "     chmod +x run.sh"
echo "     ./run.sh"
echo ""
echo -e "${BLUE}╔═══════════════════════════════════════════════════════════════╗${NC}"
echo -e "${BLUE}║  Quick Deploy Command (one-line):                            ║${NC}"
echo -e "${BLUE}╚═══════════════════════════════════════════════════════════════╝${NC}"
echo ""
echo -e "tar -xzf ${PACKAGE_NAME}.tar.gz && cd ${PACKAGE_NAME} && chmod +x run.sh && ./run.sh"
echo ""
