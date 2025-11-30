# Documentation Update Guidelines

This document provides comprehensive guidelines for updating and maintaining documentation for the OWL2 Reasoner library.

## 📝 When to Update Documentation

Update documentation when you:
- ✅ Add new features or functionality
- ✅ Change existing APIs or behavior  
- ✅ Fix bugs that affect usage
- ✅ Add new examples or use cases
- ✅ Improve performance or change behavior
- ✅ Deprecate features

## 📋 Documentation Update Checklist

### A. Code Documentation (Rustdoc)
```bash
# Update source code doc comments
# 1. Add/update module-level documentation
# 2. Add/update struct/enum documentation
# 3. Add/update function documentation with examples
# 4. Update version-specific notes
```

### B. User Guide (mdbook)
```bash
# Update mdbook documentation
# 1. Update getting-started.md for new features
# 2. Update user-guide/ sections with new usage patterns
# 3. Update examples/ with new examples
# 4. Update architecture.md for new components
# 5. Update reference/ for version changes
```

### C. Examples
```bash
# Create or update examples
# 1. Add new example for new feature
# 2. Update existing examples if API changed
# 3. Test that all examples compile and run
```

### D. Technical Documentation (Typst)
```bash
# Update technical documentation
# 1. Update docs/technical-documentation/OWL2_Reasoner_Technical_Documentation.typ
# 2. Update performance benchmarks in Appendix B
# 3. Update API reference in Appendix A
# 4. Update configuration options in Appendix D
# 5. Build updated PDF documentation
```

## 🔄 Step-by-Step Update Process

### Step 1: Update Source Code Documentation
```rust
// Example: Adding new feature documentation
/// New feature explanation
/// 
/// ## What it does
/// - Feature capability 1
/// - Feature capability 2
/// 
/// ## Performance
/// - Time complexity: O(n)
/// - Memory usage: ~100 bytes
/// 
/// ## Examples
/// 
/// ```rust
/// use owl2_reasoner::*;
/// 
/// let result = ontology.new_feature()?;
/// println!("Feature result: {:?}", result);
/// # Ok::<(), owl2_reasoner::OwlError>(())
/// ```
pub fn new_feature(&self) -> OwlResult<ReturnType> {
    // implementation
}
```

### Step 2: Update mdbook Documentation
```bash
# Edit relevant mdbook files
vim docs/src/getting-started.md      # Add new feature to quick start
vim docs/src/user-guide/advanced.md  # Add detailed usage guide
vim docs/src/examples/examples.md   # Add example documentation
vim docs/src/reference/changelog.md  # Update changelog
```

### Step 3: Create or Update Examples
```bash
# Create new example
cargo new --bin examples/new_feature_example

# Or update existing example
vim examples/simple_example.rs
```

### Step 4: Update Version Information
```bash
# Update version in Cargo.toml
vim Cargo.toml

# Update changelog
vim docs/src/reference/changelog.md
```

## 📏 Documentation Standards

### A. Doc Comment Format
```rust
/// Brief description (one line)
/// 
/// Detailed description with multiple paragraphs if needed.
/// 
/// ## Arguments
/// - `param1`: Description of first parameter
/// - `param2`: Description of second parameter
/// 
/// ## Returns
/// - `Ok(Value)`: Description of successful return
/// - `Err(OwlError)`: Description of error conditions
/// 
/// ## Examples
/// 
/// ```rust
/// # use owl2_reasoner::*;
/// let result = your_function()?;
/// println!("Result: {:?}", result);
/// # Ok::<(), owl2_reasoner::OwlError>(())
/// ```
/// 
/// ## Performance
/// - Time complexity: O(log n)
/// - Memory usage: O(1) additional space
/// 
/// ## Panics
/// - Never panics under normal conditions
pub fn your_function(param1: Type1, param2: Type2) -> OwlResult<ReturnType> {
    // implementation
}
```

### B. Example Requirements
```rust
// Examples must:
// 1. Compile without errors
// 2. Run successfully  
// 3. Demonstrate the feature clearly
// 4. Include error handling with # Ok::<(), owl2_reasoner::OwlError>(())
// 5. Be realistic and practical
```

## 🚀 Automation Script

Use the provided `update_docs.sh` script to automate documentation updates:

```bash
# For small changes
./update_docs.sh "Added new reasoning feature"

# For major features
./update_docs.sh "Major feature X release"
```

The script will:
1. Build Rustdoc documentation
2. Test all examples
3. Build mdbook documentation
4. Run tests
5. Show documentation locations

## 🔄 Documentation Update Workflows

### For Small Changes
```bash
# 1. Update source code documentation
# 2. Update relevant mdbook files
# 3. Run the update script
./update_docs.sh "Added new reasoning feature"

# 4. Review and commit
git add .
git commit -m "docs: Added new reasoning feature documentation"
git push
```

### For Major Features
```bash
# 1. Plan documentation updates
# 2. Update source code docs
# 3. Create new examples
# 4. Update user guide sections
# 5. Update API reference
# 6. Update changelog
# 7. Run full documentation build
./update_docs.sh "Major feature X release"

# 8. Review all documentation
# 9. Commit and push
```

## ✅ Documentation Quality Checklist

### Before Committing
- [ ] All examples compile and run successfully
- [ ] All doc comments follow the format standard
- [ ] Code examples in doc comments compile
- [ ] Links in documentation work correctly
- [ ] Performance information is accurate
- [ ] Error conditions are documented
- [ ] Version information is updated

### After Building
- [ ] Open generated documentation in browser
- [ ] Verify all sections display correctly
- [ ] Test code examples from documentation
- [ ] Check for broken links or formatting issues
- [ ] Verify navigation works properly

## 📝 Version Management

### Changelog Format
```markdown
## [Unreleased]

### Added
- New feature description with usage example
- Performance improvement details
- API changes with migration guide

### Changed
- Updated behavior description
- Performance characteristics
- Default values or settings

### Deprecated
- Features being deprecated with timeline
- Migration path for deprecated features

### Fixed
- Bug fixes that affect usage
- Documentation corrections

### Removed
- Features removed in this version
```

## 🤖 Continuous Integration

Add to your CI pipeline (.github/workflows/docs.yml):

```yaml
name: Documentation

on:
  push:
    branches: [main]
  pull_request:
    branches: [main]

jobs:
  build-docs:
    runs-on: ubuntu-latest
    steps:
      - uses: actions/checkout@v3
      
      - name: Install Rust
        uses: actions-rs/toolchain@v1
        with:
          toolchain: stable
          
      - name: Build documentation
        run: |
          cargo doc --no-deps
          mdbook build docs
          
      - name: Test examples
        run: |
          cargo check --example simple_example
          cargo check --example family_ontology
          
      - name: Deploy documentation
        if: github.ref == 'refs/heads/main'
        run: |
          # Deploy to GitHub Pages or other hosting
```

## ⚡ Quick Reference Commands

```bash
# Build all documentation
./update_docs.sh "Update description"

# Build only Rustdoc
cargo doc --no-deps

# Build only mdbook
mdbook build docs

# Test specific example
cargo check --example simple_example

# Run documentation lints (if available)
cargo doc --no-deps -- -D warnings

# Open documentation in browser
open target/doc/owl2_reasoner/index.html
open docs/book/index.html
```

## 📚 Documentation Locations

After building, documentation is available at:

- **mdbook**: `docs/book/index.html` - User guides and tutorials
- **Rustdoc**: `target/doc/owl2_reasoner/index.html` - API reference
- **Examples**: `examples/` directory - Working code examples

## 🎯 Best Practices

1. **Document as You Code**: Write documentation alongside implementation
2. **Keep Examples Simple**: Focus on demonstrating one concept at a time
3. **Update Changelog**: Document all changes that affect users
4. **Test Documentation**: Ensure all examples compile and run
5. **Be Consistent**: Follow the established documentation format
6. **Think About Users**: Write documentation from the user's perspective
7. **Include Performance**: Document performance characteristics
8. **Handle Errors**: Show proper error handling in examples

This ensures that your documentation stays current, accurate, and helpful as your OWL2 Reasoner library evolves! 🚀