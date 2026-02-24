# ProvChain Paper Submission

Complete journal paper submission package for **Computers and Electronics in Agriculture** (Elsevier).

## Paper Details

**Title:** ProvChain: A GS1 EPCIS-Compliant Blockchain Framework for Ultra-High Temperature (UHT) Milk Supply Chain Traceability with Semantic Web Integration

**Target Journal:** Computers and Electronics in Agriculture (Elsevier)
- Impact Factor: ~8.3
- Focus: ICT in agriculture, food supply chains, traceability

## Files

| File | Description |
|------|-------------|
| `main.tex` | Complete LaTeX manuscript (~29KB) |
| `references.bib` | BibTeX references (22 citations) |
| `README.md` | This file |

## Paper Structure

1. **Abstract** - Summary of contributions
2. **Highlights** - 5 bullet points of key contributions
3. **Introduction** - Background, motivation, research gaps, contributions
4. **Related Work** - Literature review, comparison table
5. **System Architecture** - Technical design, ontologies, blockchain layer
6. **Implementation** - Technology stack, code examples
7. **Evaluation** - Performance metrics, standards compliance, fault tolerance
8. **Discussion** - Industry implications, limitations, future work
9. **Conclusion** - Summary of contributions

## Key Contributions Highlighted

1. Full GS1 EPCIS 2.0 integration with blockchain
2. UHT-specific ontology with SHACL validation
3. OWL2 reasoning support
4. Write-Ahead Logging (WAL) persistence
5. Interactive demonstration (8 phases, 7 event types)

## Performance Metrics

- 65ms average event latency
- 128 events/second throughput
- 100% EPCIS 2.0 compliant
- Zero data loss on crash recovery

## How to Compile

### Requirements
- LaTeX distribution (TeX Live or MiKTeX)
- BibTeX or Biber

### Compile Commands

```bash
cd docs/paper_submission

# Method 1: Manual compilation
pdflatex main.tex
bibtex main
pdflatex main.tex
pdflatex main.tex

# Method 2: Using latexmk (recommended)
latexmk -pdf main.tex

# Clean auxiliary files
latexmk -c
```

## Pre-Submission Checklist

### Author Information (Section to complete)
- [ ] Add author names in `\author{}` commands
- [ ] Add corresponding author email in `\ead{}`
- [ ] Add ORCID numbers if available
- [ ] Add author affiliations in `\address{}`

### Content (Optional additions)
- [ ] Insert graphical abstract in `\begin{graphicalabstract}`
- [ ] Create high-resolution figures (architecture diagram)
- [ ] Verify all citations are correct
- [ ] Check for any funding acknowledgments

### Submission Requirements
- [ ] Add competing interests statement
- [ ] Add data availability statement (GitHub link)
- [ ] Prepare cover letter for editors
- [ ] Suggest 3-5 potential reviewers

## Journal Submission Guidelines

**Computers and Electronics in Agriculture** submission portal:
https://www.editorialmanager.com/compag

### Article Type
Select "Research Paper"

### Highlights
The paper includes 5 highlights in the `highlights` environment (already formatted).

### Keywords
10 keywords included in `keywords` environment.

### Graphical Abstract
Placeholder included. Create a visual summary showing:
- Farm to retail supply chain flow
- Blockchain layer with EPCIS events
- Semantic web layer with ontologies
- Key performance metrics

## Citation Style

This paper uses **Harvard (author-year)** citation style:
- `\cite{key}` produces: (Author, Year)
- `\citep{key}` produces: (Author, Year)
- `\citet{key}` produces: Author (Year)

## Word Count

Approximate word counts:
- Abstract: ~250 words
- Main text: ~6,000-7,000 words
- Total (including references): ~8,000 words

## LaTeX Class Options

Current options: `preprint,12pt,authoryear`

For final submission, change to: `final,5p,times,authoryear`

## Contact

[To be filled by corresponding author]
