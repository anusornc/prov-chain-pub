#!/bin/bash

# Build technical documentation with Typst
# This script builds the comprehensive technical documentation

set -e

# Colors for output
RED='\033[0;31m'
GREEN='\033[0;32m'
YELLOW='\033[1;33m'
BLUE='\033[0;34m'
NC='\033[0m' # No Color

echo -e "${BLUE}📚 Building OWL2 Reasoner Technical Documentation${NC}"

# Check if Typst is installed
if ! command -v typst &> /dev/null; then
    echo -e "${RED}❌ Typst is not installed. Please install it first:${NC}"
    echo -e "${YELLOW}📦 curl -fsSL https://github.com/typst/typst/releases/download/v0.10.0/typst-x86_64-unknown-linux-musl.tar.gz | tar -xz${NC}"
    echo -e "${YELLOW}📦 sudo mv typst /usr/local/bin/${NC}"
    exit 1
fi

# Create output directory
mkdir -p docs/technical-documentation/output

echo -e "${YELLOW}🔧 Building technical documentation...${NC}"

# Build PDF documentation
cd docs/technical-documentation
typst compile OWL2_Reasoner_Technical_Documentation.typ output/OWL2_Reasoner_Technical_Documentation.pdf

echo -e "${GREEN}✅ Technical documentation built successfully!${NC}"
echo -e "${BLUE}📄 Output: docs/technical-documentation/output/OWL2_Reasoner_Technical_Documentation.pdf${NC}"

# Also build HTML version if requested
if [[ "$1" == "--html" ]]; then
    echo -e "${YELLOW}🌐 Building HTML version...${NC}"
    # Note: Typst doesn't have native HTML export yet
    # This is a placeholder for future HTML generation
    echo -e "${YELLOW}⚠️  HTML export not yet supported by Typst${NC}"
fi

# Show file info
echo -e "${BLUE}📊 Documentation statistics:${NC}"
if command -v wc &> /dev/null; then
    echo -e "${GREEN}📄 Pages: $(pdfinfo output/OWL2_Reasoner_Technical_Documentation.pdf 2>/dev/null | grep Pages | cut -d: -f2 | xargs)${NC}"
fi

if command -v du &> /dev/null; then
    echo -e "${GREEN}💾 File size: $(du -h output/OWL2_Reasoner_Technical_Documentation.pdf | cut -f1)${NC}"
fi

echo -e "${GREEN}🎉 Technical documentation complete!${NC}"