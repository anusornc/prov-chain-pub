# Makefile for ProvChain Paper
# Target Journal: Computers and Electronics in Agriculture

# Configuration
MAIN = main
BUILD_DIR = build
PDF = $(BUILD_DIR)/$(MAIN).pdf

# Programs
LATEX = pdflatex
BIBTEX = bibtex
# Alternative: BIBTEX = biber

# Flags
LATEX_FLAGS = -interaction=nonstopmode -halt-on-error

# Default target
.PHONY: all clean watch help

all: $(PDF)

# Create build directory
$(BUILD_DIR):
	mkdir -p $(BUILD_DIR)

# Main compilation rule
$(PDF): $(MAIN).tex references.bib $(BUILD_DIR)
	@echo "=== Compiling LaTeX (1st pass) ==="
	$(LATEX) $(LATEX_FLAGS) $(MAIN).tex
	
	@echo "=== Running BibTeX ==="
	$(BIBTEX) $(MAIN) || true
	
	@echo "=== Compiling LaTeX (2nd pass) ==="
	$(LATEX) $(LATEX_FLAGS) $(MAIN).tex
	
	@echo "=== Compiling LaTeX (3rd pass) ==="
	$(LATEX) $(LATEX_FLAGS) $(MAIN).tex
	
	@mv $(MAIN).pdf $(BUILD_DIR)/
	@echo "=== Build complete: $(PDF) ==="

# Quick compile (no bibtex)
quick:
	$(LATEX) $(LATEX_FLAGS) $(MAIN).tex
	@mv $(MAIN).pdf $(BUILD_DIR)/ 2>/dev/null || true

# Clean auxiliary files
clean:
	@echo "=== Cleaning auxiliary files ==="
	rm -f *.aux *.bbl *.blg *.log *.out *.toc *.lof *.lot
	rm -f *.fls *.fdb_latexmk *.synctex.gz
	rm -f *.nav *.snm *.vrb *.run.xml *.bcf
	rm -f *.glo *.gls *.idx *.ind *.ilg *.nlo *.nls
	rm -f *.bak *.backup
	rm -rf $(BUILD_DIR)
	@echo "=== Clean complete ==="

# Watch mode (requires inotifywait or fswatch)
watch:
	@echo "=== Starting watch mode ==="
	@echo "Press Ctrl+C to stop"
	@which inotifywait >/dev/null 2>&1 && \
		inotifywait -m -e modify -r . --format '%f' --include='.*\\.(tex|bib)$$' 2>/dev/null | \
		while read file; do echo "File changed: $$file"; make; done || \
	(fswatch -o *.tex *.bib 2>/dev/null | while read; do make; done)

# Show word count
wordcount:
	@echo "=== Word Count ==="
	@detex $(MAIN).tex | wc -w
	@echo "Note: Approximately 4000-5000 words for abstract + main text"

# Check for common issues
check:
	@echo "=== Checking for common issues ==="
	@echo "Checking for TODO/FIXME..."
	@grep -n "TODO\|FIXME\|XXX\|HACK" *.tex || echo "No TODOs found"
	@echo ""
	@echo "Checking for empty citations..."
	@grep -n "cite{}" *.tex || echo "No empty citations found"
	@echo ""
	@echo "Checking for placeholder text..."
	@grep -n "\\[.*To be.*\\]\|\\[.*Fill in.*\\]" *.tex || echo "No placeholders found"

# View PDF (Linux)
view: $(PDF)
	xdg-open $(PDF)

# View PDF (macOS)
view-mac: $(PDF)
	open $(PDF)

# Submit preparation (final clean build)
submit: clean $(PDF)
	@echo "=== Preparing submission package ==="
	@cp $(PDF) ProvChain_Paper_$(shell date +%Y%m%d).pdf
	@echo "Submission PDF created: ProvChain_Paper_$(shell date +%Y%m%d).pdf"
	@echo ""
	@echo "Submission checklist:"
	@echo "  [ ] Author information filled in"
	@echo "  [ ] Funding acknowledgment added"
	@echo "  [ ] Competing interests declared"
	@echo "  [ ] Graphical abstract created"
	@echo "  [ ] All figures high resolution"
	@echo "  [ ] Supplementary materials prepared"

# Help
help:
	@echo "ProvChain Paper - Makefile Commands"
	@echo "===================================="
	@echo ""
	@echo "make          - Compile paper with bibtex (full build)"
	@echo "make quick    - Quick compile (no bibtex)"
	@echo "make clean    - Remove all auxiliary files"
	@echo "make watch    - Watch mode (auto-recompile on changes)"
	@echo "make wordcount- Count words in document"
	@echo "make check    - Check for common issues"
	@echo "make view     - Open PDF (Linux)"
	@echo "make view-mac - Open PDF (macOS)"
	@echo "make submit   - Create submission package"
	@echo "make help     - Show this help"
