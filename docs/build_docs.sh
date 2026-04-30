#!/bin/bash

# ProvChainOrg Documentation Build Script
# This script builds the complete documentation with PlantUML diagrams

set -e  # Exit on any error

echo "🚀 Building ProvChainOrg Documentation..."

# Check if we're in the docs directory
if [ ! -f "conf.py" ]; then
    echo "❌ Error: Please run this script from the docs/ directory"
    exit 1
fi

# Check for required tools
echo "🔍 Checking dependencies..."

# Check Python and pip
if ! command -v python3 &> /dev/null; then
    echo "❌ Error: Python 3 is required but not installed"
    exit 1
fi

if ! command -v pip &> /dev/null; then
    echo "❌ Error: pip is required but not installed"
    exit 1
fi

# Check for PlantUML (optional but recommended)
if ! command -v plantuml &> /dev/null; then
    echo "⚠️  Warning: PlantUML not found. Diagrams will not be generated."
    echo "   Install with: sudo apt-get install plantuml (Ubuntu/Debian)"
    echo "   Or: brew install plantuml (macOS)"
    echo "   Or download from: https://plantuml.com/download"
else
    echo "✅ PlantUML found"
fi

# Install Python dependencies
echo "📦 Installing Python dependencies..."
if [ -f "requirements.txt" ]; then
    pip install -r requirements.txt
else
    echo "❌ Error: requirements.txt not found"
    exit 1
fi

# Generate PlantUML diagrams if PlantUML is available
if command -v plantuml &> /dev/null && [ -d "diagrams" ]; then
    echo "🎨 Generating PlantUML diagrams..."
    cd diagrams
    for puml_file in *.puml; do
        if [ -f "$puml_file" ]; then
            echo "   Generating diagram: $puml_file"
            plantuml "$puml_file"
        fi
    done
    cd ..
    echo "✅ Diagrams generated successfully"
fi

# Clean previous build
echo "🧹 Cleaning previous build..."
if [ -d "_build" ]; then
    rm -rf _build
fi

# Build HTML documentation
echo "📚 Building HTML documentation..."
sphinx-build -b html . _build/html

# Check if build was successful
if [ $? -eq 0 ]; then
    echo "✅ HTML documentation built successfully!"
    echo "📂 Output location: _build/html/"
    echo "🌐 Open _build/html/index.html in your browser"
else
    echo "❌ Error: HTML build failed"
    exit 1
fi

# Build PDF documentation (optional) - DISABLED due to Unicode character issues
echo "⚠️  Skipping PDF generation (Unicode characters not compatible with LaTeX)"
echo "   HTML documentation provides full functionality"

# Build EPUB documentation (optional)
echo "📱 Building EPUB documentation..."
sphinx-build -b epub . _build/epub
if [ $? -eq 0 ]; then
    echo "✅ EPUB documentation built successfully!"
    echo "📱 EPUB location: _build/epub/ProvChainOrg.epub"
else
    echo "⚠️  Warning: EPUB build failed"
fi

# Generate link check report
echo "🔗 Checking for broken links..."
sphinx-build -b linkcheck . _build/linkcheck
if [ $? -eq 0 ]; then
    echo "✅ Link check completed successfully!"
    if [ -f "_build/linkcheck/output.txt" ]; then
        echo "📋 Link check report: _build/linkcheck/output.txt"
    fi
else
    echo "⚠️  Warning: Link check found issues"
fi

echo ""
echo "🎉 Documentation build complete!"
echo ""
echo "📂 Available outputs:"
echo "   HTML: _build/html/index.html"
if [ -f "_build/latex/provchainorg.pdf" ]; then
    echo "   PDF:  _build/latex/provchainorg.pdf"
fi
if [ -f "_build/epub/ProvChainOrg.epub" ]; then
    echo "   EPUB: _build/epub/ProvChainOrg.epub"
fi
echo ""
echo "🚀 To serve the documentation locally:"
echo "   cd _build/html && python -m http.server 8000"
echo "   Then open: http://localhost:8000"
echo ""
echo "📝 To rebuild with live reload during development:"
echo "   make livehtml"
echo "   Then open: http://localhost:8000"
