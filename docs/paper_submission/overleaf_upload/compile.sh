#!/bin/bash
#
# LaTeX Compilation Script for ProvChain Paper
# Target Journal: Computers and Electronics in Agriculture (Elsevier)
#
# Usage:
#   ./compile.sh              # Compile with bibtex
#   ./compile.sh clean        # Clean auxiliary files
#   ./compile.sh watch        # Watch mode (auto-recompile on changes)
#

set -e  # Exit on error

# Configuration
MAIN_FILE="main"
OUTPUT_DIR="build"
PDF_VIEWER=""  # Set to "evince", "okular", "xdg-open", etc. or leave empty

# Colors for output
RED='\033[0;31m'
GREEN='\033[0;32m'
YELLOW='\033[1;33m'
BLUE='\033[0;34m'
NC='\033[0m' # No Color

# Function to print colored messages
info() {
    echo -e "${BLUE}[INFO]${NC} $1"
}

success() {
    echo -e "${GREEN}[SUCCESS]${NC} $1"
}

warning() {
    echo -e "${YELLOW}[WARNING]${NC} $1"
}

error() {
    echo -e "${RED}[ERROR]${NC} $1"
}

# Function to check if command exists
command_exists() {
    command -v "$1" >/dev/null 2>&1
}

# Check dependencies
check_dependencies() {
    info "Checking dependencies..."
    
    if ! command_exists pdflatex; then
        error "pdflatex not found! Please install TeX Live."
        echo "  Ubuntu/Debian: sudo apt-get install texlive-full"
        echo "  macOS: brew install --cask mactex"
        echo "  Or download from: https://tug.org/texlive/"
        exit 1
    fi
    
    if ! command_exists bibtex; then
        warning "bibtex not found! Trying bibtex8..."
        if ! command_exists bibtex8; then
            error "No bibtex found!"
            exit 1
        fi
    fi
    
    success "All dependencies found!"
}

# Clean auxiliary files
clean() {
    info "Cleaning auxiliary files..."
    rm -f *.aux *.bbl *.blg *.log *.out *.toc *.lof *.lot
    rm -f *.fls *.fdb_latexmk *.synctex.gz
    rm -f *.nav *.snm *.vrb *.run.xml
    rm -f *.bcf *.glo *.gls *.idx *.ind *.ilg
    rm -f *.nlo *.nls *.bak *.backup
    rm -rf "$OUTPUT_DIR"
    success "Clean complete!"
}

# Create build directory
setup_build() {
    if [ ! -d "$OUTPUT_DIR" ]; then
        mkdir -p "$OUTPUT_DIR"
    fi
}

# Compile LaTeX document
compile() {
    info "Starting LaTeX compilation..."
    info "Main file: $MAIN_FILE.tex"
    
    # Run pdflatex first time
    info "Running pdflatex (1st pass)..."
    pdflatex -interaction=nonstopmode -halt-on-error "$MAIN_FILE.tex" 2>&1 | tee "$OUTPUT_DIR/compile.log" || {
        error "pdflatex failed on first pass!"
        error "Check $OUTPUT_DIR/compile.log for details"
        exit 1
    }
    
    # Run bibtex
    info "Running bibtex..."
    if command_exists bibtex; then
        bibtex "$MAIN_FILE" 2>&1 | tee -a "$OUTPUT_DIR/compile.log" || {
            warning "bibtex had warnings, continuing..."
        }
    else
        bibtex8 "$MAIN_FILE" 2>&1 | tee -a "$OUTPUT_DIR/compile.log" || {
            warning "bibtex8 had warnings, continuing..."
        }
    fi
    
    # Run pdflatex second time
    info "Running pdflatex (2nd pass)..."
    pdflatex -interaction=nonstopmode -halt-on-error "$MAIN_FILE.tex" 2>&1 | tee -a "$OUTPUT_DIR/compile.log" || {
        error "pdflatex failed on second pass!"
        exit 1
    }
    
    # Run pdflatex third time (for references)
    info "Running pdflatex (3rd pass)..."
    pdflatex -interaction=nonstopmode -halt-on-error "$MAIN_FILE.tex" 2>&1 | tee -a "$OUTPUT_DIR/compile.log" || {
        error "pdflatex failed on third pass!"
        exit 1
    }
    
    success "Compilation successful!"
    
    # Move PDF to build directory
    if [ -f "$MAIN_FILE.pdf" ]; then
        mv "$MAIN_FILE.pdf" "$OUTPUT_DIR/"
        success "PDF created: $OUTPUT_DIR/$MAIN_FILE.pdf"
        
        # Show PDF info
        info "PDF Information:"
        pdfinfo "$OUTPUT_DIR/$MAIN_FILE.pdf" 2>/dev/null || true
        
        # Open PDF if viewer is set
        if [ -n "$PDF_VIEWER" ] && command_exists "$PDF_VIEWER"; then
            info "Opening PDF with $PDF_VIEWER..."
            "$PDF_VIEWER" "$OUTPUT_DIR/$MAIN_FILE.pdf" &
        fi
    fi
}

# Watch mode (using inotifywait or fswatch)
watch_mode() {
    if command_exists inotifywait; then
        info "Watch mode started. Monitoring *.tex and *.bib files..."
        info "Press Ctrl+C to stop"
        while true; do
            inotifywait -e modify,create,delete -r . --include='.*\\.(tex|bib)$' 2>/dev/null
            info "File changed, recompiling..."
            compile
        done
    elif command_exists fswatch; then
        info "Watch mode started. Monitoring *.tex and *.bib files..."
        info "Press Ctrl+C to stop"
        fswatch -o *.tex *.bib 2>/dev/null | while read; do
            info "File changed, recompiling..."
            compile
        done
    else
        error "Watch mode requires inotifywait (Linux) or fswatch (macOS)"
        echo "  Ubuntu: sudo apt-get install inotify-tools"
        echo "  macOS: brew install fswatch"
        exit 1
    fi
}

# Show help
show_help() {
    echo "ProvChain Paper Compilation Script"
    echo "=================================="
    echo ""
    echo "Usage: ./compile.sh [command]"
    echo ""
    echo "Commands:"
    echo "  (none)     Compile the paper with bibtex"
    echo "  clean      Remove all auxiliary files"
    echo "  watch      Watch mode (auto-recompile on changes)"
    echo "  help       Show this help message"
    echo ""
    echo "Configuration:"
    echo "  Edit PDF_VIEWER variable to auto-open PDF"
    echo "  Current viewer: ${PDF_VIEWER:-'(not set)'}}"
    echo ""
    echo "Examples:"
    echo "  ./compile.sh              # Compile paper"
    echo "  ./compile.sh clean        # Clean auxiliary files"
    echo "  ./compile.sh watch        # Auto-recompile on changes"
}

# Main script
main() {
    # Check if in correct directory
    if [ ! -f "$MAIN_FILE.tex" ]; then
        error "$MAIN_FILE.tex not found!"
        error "Please run this script from the paper_submission directory"
        exit 1
    fi
    
    case "${1:-}" in
        clean)
            clean
            ;;
        watch)
            check_dependencies
            compile
            watch_mode
            ;;
        help|--help|-h)
            show_help
            ;;
        "")
            check_dependencies
            setup_build
            compile
            success "All done! PDF is ready at: $OUTPUT_DIR/$MAIN_FILE.pdf"
            ;;
        *)
            error "Unknown command: $1"
            show_help
            exit 1
            ;;
    esac
}

# Run main function
main "$@"
