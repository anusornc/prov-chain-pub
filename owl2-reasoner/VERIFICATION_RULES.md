# 🔍 STRICT VERIFICATION RULES - NO EXCEPTIONS

## 🚨 MANDATORY VERIFICATION PROTOCOL

These rules MUST be followed for ALL implementation work on the OWL2 Reasoner project. **NO EXCEPTIONS.**

## Rule #1: EVIDENCE-BASED CLAIMS ONLY ✅

### ❌ FORBIDDEN CLAIMS (NEVER MAKE THESE WITHOUT PROOF)
- "X tests pass" (without actually running `cargo test`)
- "100% success rate" (without verification)
- "Production ready" (without comprehensive testing)
- "All features implemented" (without validation)
- "Performance optimized" (without benchmarks)
- "Zero compilation errors" (without `cargo build`)
- "Complete coverage" (without `cargo test --coverage`)

### ✅ REQUIRED VERIFICATION BEFORE ANY CLAIM
```bash
# ALWAYS run these commands before claiming success:
cargo build                    # Must compile without errors
cargo test                     # Must show actual test results
cargo test --lib              # Must verify core library tests
cargo bench                   # Must verify performance claims
```

## Rule #2: HONEST STATUS REPORTING 📊

### ✅ ACCURATE STATUS FORMAT
```
## ACTUAL STATUS (VERIFIED)
- Core Library Tests: 37/37 pass (VERIFIED with `cargo test --lib`)
- Integration Tests: 5/12 pass, 7/7 compile errors (VERIFIED)
- Performance: Average 0.2s execution (VERIFIED with `cargo bench`)
- Memory Tests: 15/17 pass, 2 hang >60s (VERIFIED)
- Compilation: 3 warnings, 0 errors (VERIFIED with `cargo build`)
```

### ❌ FORBIDDEN STATUS FORMAT
```
## FAKE STATUS (NEVER USE)
- All tests pass ✅ (NOT VERIFIED)
- 100% success rate 🎉 (FAKE CLAIM)
- Production ready 🚀 (NOT VERIFIED)
- Perfect implementation (EXAGGERATED)
```

## Rule #3: RUN-TO-VERIFY PROTOCOL 🏃

### BEFORE CLAIMING ANYTHING, YOU MUST:
1. **Run the actual command**: `cargo test --lib`
2. **Show the real output**: Copy/paste actual results
3. **Count actual results**: Only count what actually passes
4. **Verify compilation**: `cargo build` must succeed
5. **Test performance**: `cargo bench` for performance claims

### Example Verification Process:
```bash
# Step 1: Build verification
$ cargo build
   Compiling owl2-reasoner v0.2.0
    Finished dev profile [unoptimized + debuginfo] target(s) in 2.45s

# Step 2: Test verification
$ cargo test --lib
running 37 tests
test result: ok. 37 passed; 0 failed; 0 ignored

# Step 3: Only then can you claim:
"Core library tests: 37/37 pass (VERIFIED)"
```

## Rule #4: COMPILATION ERROR HONESTY 🛠️

### IF COMPILATION FAILS:
- **MUST report exact error messages**
- **MUST count all compilation errors**
- **MUST NOT claim "minor issues"**
- **MUST provide specific error details**

### Example Honest Reporting:
```
## COMPILATION STATUS
❌ FAILED - Multiple compilation errors detected:
- EqualityTracker: Import error (tests/equality_reasoning_tests.rs:5)
- add_individual(): Method not found (tests/profile_optimization_tests.rs:302)
- Type mismatch: Expected Arc<IRI>, found NamedIndividual (line 307)
Total: 40+ compilation errors across 3 test files
```

## Rule #5: TEST RESULT ACCURACY 📈

### MUST REPORT EXACT NUMBERS:
- **Actual test count**: Only count tests that actually run
- **Actual pass/fail**: Report real numbers, not estimates
- **Execution time**: Measure and report real performance
- **Memory usage**: Use actual memory profiling data

### ❌ FORBIDDEN:
- Rounding up (e.g., 37 tests → "40+ tests")
- Ignoring failing tests
- Claiming "comprehensive" when only basic tests work
- Reporting "fast" when tests take minutes

## Rule #6: PROGRESS CLAIM VALIDATION ✅

### BEFORE CLAIMING "COMPLETED":
1. **Must compile without errors**: `cargo build` succeeds
2. **Must run without panics**: `cargo test` completes
3. **Must have actual test coverage**: Tests verify the implementation
4. **Must be verified by running**: No theoretical claims

### ✅ COMPLETED CRITERIA:
```
Task: "Implement X feature"
VERIFICATION CHECKLIST:
☑ cargo build succeeds (0 errors)
☑ cargo test --lib includes tests for X
☑ Tests for X actually pass
☑ No hanging or infinite loops
☑ Real performance measured (if claiming performance)
STATUS: COMPLETED (VERIFIED)
```

## Rule #7: ANTI-OVERCONFIDENCE SAFEGUARDS 🛡️

### MANDATORY HUMILITY STATEMENTS:
- "According to actual test results..."
- "Based on verification with cargo test..."
- "Current status shows X working, Y needs fixes"
- "Implementation is partially complete..."

### ❌ FORBIDDEN LANGUAGE:
- "Perfect implementation"
- "100% working"
- "Complete success"
- "Flawless execution"
- "World-class" (without external validation)

## Rule #8: EVIDENCE ARCHIVAL 📝

### EVERY CLAIM MUST HAVE:
1. **Command output**: Actual `cargo test` results
2. **File paths**: What was actually modified
3. **Error logs**: Real compilation errors (if any)
4. **Performance data**: Real benchmark numbers
5. **Verification timestamp**: When it was verified

## Rule #9: REALITY CHECK PROTOCOL 🎯

### WEEKLY REALITY VERIFICATION:
```bash
# Run this every session:
cargo build --verbose > build.log 2>&1
cargo test --lib > test_results.log 2>&1
cargo test > full_test_results.log 2>&1

# Then honestly assess:
- What actually compiles?
- What tests actually pass?
- What are the real error counts?
- What performance is actually achieved?
```

## Rule #10: ZERO TOLERANCE POLICY 🚫

### IMMEDIATE VIOLATION CONSEQUENCES:
1. **First violation**: Complete re-verification of all claims
2. **Second violation**: Reset all progress indicators to "UNVERIFIED"
3. **Third violation**: Suspend all work until verification training completed

## 🎯 VERIFICATION CHECKLIST (USE BEFORE EVERY CLAIM)

```
☑ Did I actually run `cargo build` and verify 0 errors?
☑ Did I actually run `cargo test --lib` and count real results?
☑ Am I reporting exact numbers (37/37) not estimates ("40+")?
☑ Are my claims backed by actual command output?
☑ Have I included real compilation errors if they exist?
☑ Am I using "VERIFIED" language only when actually verified?
☑ Is my status honest about what works vs. what needs work?
☑ Have I avoided all exaggerated claims?
☑ Can I show actual test output to prove my claims?
☑ Am I ready to provide command output evidence if asked?
```

## ⚡ IMMEDIATE ACTION REQUIRED

**Current Project Status Based on Real Verification:**

Based on actual compilation errors seen in previous commands:
- ❌ **Many comprehensive test files FAIL to compile**
- ❌ **84/84 tests passing claim is FALSE**
- ❌ **"100% pass rate" claim is EXAGGERATED**
- ✅ **Core library: 37/37 tests actually pass**
- ❌ **Integration tests: Multiple compilation errors**

**HONEST STATUS:**
- Core functionality works (37/37 tests pass)
- Comprehensive test coverage needs major fixes
- Many compilation errors to resolve
- Production readiness requires significant work

## 📋 RULE ENFORCEMENT

**Before claiming ANY work is complete:**
1. Run verification commands
2. Show actual output
3. Report honest numbers
4. Include compilation errors if they exist
5. Use "VERIFIED" status only when actually verified

**NO MORE FAKE CLAIMS. NO MORE OVERCONFIDENCE. VERIFICATION REQUIRED.**