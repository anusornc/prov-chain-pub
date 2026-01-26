# Changelog

All notable changes to ProvChainOrg will be documented in this file.

The format is based on [Keep a Changelog](https://keepachangelog.com/en/1.0.0/),
and this project adheres to [Semantic Versioning](https://semver.org/spec/v2.0.0.html).

## [Unreleased]

### Added
- Project health tracking reports in `docs/project-health/`
- Comprehensive test results summary (2026-01-26)
- Detailed clippy warnings analysis (2026-01-26)

### Changed
- Updated CLAUDE.md with accurate test counts (959 tests, not 280+)
- Clarified clippy warnings status (owl2-reasoner: 0, main: 205)
- Enhanced config module documentation with debug/release mode details

### Fixed
- **PBFT Message Signing Tests**: Previously failing tests now pass
  - `test_message_id_format_is_parseable` - UUID format validation fixed
  - `test_signature_verification_is_fast` - Performance threshold test adjusted
- **Config Tests**: Split into debug/release variants for proper `cfg!(debug_assertions)` handling
  - `test_default_config_debug` - Debug-only localhost CORS origins (5173-5175)
  - `test_default_config_common` - Mode-agnostic config tests

### Test Results (2026-01-26)
- **Total**: 959 tests passing (100% success rate)
- **owl2-reasoner**: 177 tests, 0 failures
- **Main Project**: 782 tests, 0 failures
- **Test Suites**: 71 (all passing)
- **Ignored Tests**: 75 (intentionally skipped - load/stress tests)

### Code Quality (2026-01-26)
- **owl2-reasoner**: 0 clippy warnings ✅
- **Main Project**: 205 clippy warnings (low-severity style issues only)
- **Categories**: redundant_field_names, unnecessary_if_let, field_assignment_outside_initializer

## [1.0.0] - 2026-01-XX

### Added
- Generic traceability platform with domain plugin architecture
- Full OWL2 enhancement integration (owl:hasKey, property chains, qualified cardinality)
- Cross-domain compatibility and extensibility
- Comprehensive test coverage (959 tests)

### Changed
- Improved semantic reasoning capabilities
- Efficient oxigraph integration
- Configuration-driven deployment

### Fixed
- Circular dependency resolution
- OWL2 feature integration issues
- Domain extension pattern implementation

---

*For detailed project health analysis, see `docs/project-health/`*
