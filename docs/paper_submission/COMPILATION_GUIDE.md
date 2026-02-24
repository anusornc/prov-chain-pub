# LaTeX Compilation Guide

## Quick Start

```bash
cd docs/paper_submission

# Method 1: Using the bash script
./compile.sh

# Method 2: Using make
make

# Method 3: Manual compilation
pdflatex main.tex
bibtex main
pdflatex main.tex
pdflatex main.tex
```

## Prerequisites

### Install TeX Live

**Ubuntu/Debian:**
```bash
sudo apt-get update
sudo apt-get install texlive-full
# OR minimal installation:
sudo apt-get install texlive-latex-base texlive-latex-extra texlive-fonts-recommended texlive-bibtex-extra biber
```

**macOS:**
```bash
brew install --cask mactex
# OR minimal:
brew install --cask mactex-no-gui
```

**Windows:**
Download from: https://tug.org/texlive/windows.html

### Verify Installation

```bash
pdflatex --version
bibtex --version
```

## Compilation Scripts

### Option 1: Bash Script (compile.sh)

```bash
# Full compilation
./compile.sh

# Clean auxiliary files
./compile.sh clean

# Watch mode (auto-recompile on changes)
./compile.sh watch

# Show help
./compile.sh help
```

**Features:**
- Automatic dependency checking
- Colored output messages
- Error handling with helpful messages
- Three-pass compilation for proper references
- Optional PDF auto-open
- Watch mode for development

### Option 2: Makefile

```bash
# Full compilation
make

# Quick compile (no bibtex)
make quick

# Clean files
make clean

# Watch mode
make watch

# Word count
make wordcount

# Check for issues
make check

# View PDF
make view        # Linux
make view-mac    # macOS

# Create submission package
make submit
```

## Troubleshooting

### Error: "File `elsarticle.cls' not found"

Install the missing package:
```bash
# Ubuntu/Debian
sudo apt-get install texlive-publishers

# With tlmgr (TeX Live Manager)
tlmgr install elsarticle
```

### Error: "File `biblatex.sty' not found"

```bash
sudo apt-get install texlive-bibtex-extra biber
```

### Error: "Citation undefined"

Make sure to run bibtex:
```bash
pdflatex main.tex
bibtex main
pdflatex main.tex
pdflatex main.tex
```

### Error: "Dimension too large" in TikZ

This is usually a coordinate issue. Check the TikZ diagram for invalid coordinates.

### Font warnings

These are usually harmless but can be fixed by installing additional fonts:
```bash
sudo apt-get install texlive-fonts-recommended texlive-fonts-extra
```

## Output Files

After successful compilation:
```
docs/paper_submission/
├── build/
│   ├── main.pdf              ← Final PDF
│   └── compile.log           ← Compilation log
├── main.tex                  ← Source file
└── ... (auxiliary files)
```

## Journal Submission Checklist

Before submitting, verify:

- [ ] PDF compiles without errors
- [ ] All citations resolve correctly
- [ ] All figures appear correctly
- [ ] Tables are properly formatted
- [ ] No overfull/underfull hbox warnings
- [ ] Page count appropriate (typically 15-25 pages)

### Final Build

```bash
make clean
make
make check
make submit
```

## VS Code Integration

If using VS Code with LaTeX Workshop extension:

```json
// .vscode/settings.json
{
    "latex-workshop.latex.outDir": "./build",
    "latex-workshop.latex.recipes": [
        {
            "name": "pdflatex -> bibtex -> pdflatex*2",
            "tools": [
                "pdflatex",
                "bibtex",
                "pdflatex",
                "pdflatex"
            ]
        }
    ]
}
```

## Overleaf

To use on Overleaf:
1. Zip the paper_submission directory
2. Upload to Overleaf
3. Set compiler to pdfLaTeX
4. bibliography tool to BibTeX

## Performance Tips

### Fast Compilation (for drafting)
```bash
# Draft mode (faster, lower quality images)
pdflatex -draftmode main.tex

# Or use the quick option
make quick
```

### Final Compilation (for submission)
```bash
# Full compilation with optimization
make clean
make
```

## Getting Help

If compilation fails:
1. Check the error message at the end of the output
2. Look at `build/compile.log` for details
3. Common fixes:
   - Missing packages: Install with `tlmgr` or `apt-get`
   - Citation issues: Run `bibtex` then `pdflatex` twice
   - Figure issues: Check file paths and formats

## References

- elsarticle documentation: https://www.elsevier.com/authors/policies-and-guidelines/latex-instructions
- TeX Live: https://tug.org/texlive/
- TeX Stack Exchange: https://tex.stackexchange.com/
