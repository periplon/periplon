# Coverage Analysis Summary

**Date:** 2025-10-26  
**Overall Coverage:** 57.56% (5,539 / 9,623 lines)

---

## Quick Reference

### Current State
- ✅ 1 feature above 80%: **DSL-CORE** (only template.rs at 100%)
- 🟡 13 features below 80% (92.9% of all features)
- 🔴 6 features below 20% (critical)

### Test Lines Needed
- **To reach 65%:** ~867 test lines (Phase 1: 4-6 weeks)
- **To reach 70%:** ~1,358 test lines (Phase 1-2: 8-12 weeks)
- **To reach 80%:** ~2,168 test lines (All phases: 14-21 weeks)

---

## Features Ranked by Priority

| Rank | Feature | Coverage | Gap to 80% | Lines Needed | Impact | Priority |
|------|---------|----------|------------|--------------|--------|----------|
| 1 | DSL-EXECUTOR | 22.0% | 58.0% | 1,061 | +11.0% | 🔥 CRITICAL |
| 2 | ADAPTERS-SECONDARY | 3.2% | 76.8% | 191 | +2.0% | 🔥 CRITICAL |
| 3 | DSL-PREDEFINED-TASKS | 70.4% | 9.6% | 174 | +1.8% | 🔴 HIGH |
| 4 | DSL-VALIDATOR | 58.3% | 21.7% | 169 | +1.8% | 🔴 HIGH |
| 5 | TUI | 0.0% | 80.0% | 146 | +1.5% | 🔴 HIGH |
| 6 | DSL-NL-GENERATOR | 9.6% | 70.4% | 116 | +1.2% | 🟡 MEDIUM |
| 7 | APPLICATION | 25.6% | 54.4% | 89 | +0.9% | 🟡 MEDIUM |
| 8 | ADAPTERS-PRIMARY | 0.0% | 80.0% | 59 | +0.6% | 🟡 MEDIUM |
| 9 | DSL-PARSER | 13.2% | 66.8% | 50 | +0.5% | 🟡 MEDIUM |
| 10 | SERVER | 0.0% | 80.0% | 37 | +0.4% | 🟡 MEDIUM |
| 11 | OTHER | 49.1% | 30.9% | 35 | +0.4% | 🟡 MEDIUM |
| 12 | TESTING | 69.3% | 10.7% | 30 | +0.3% | 🟡 MEDIUM |
| 13 | DOMAIN | 33.3% | 46.7% | 11 | +0.1% | 🟡 MEDIUM |

**Total:** 2,168 lines needed

---

## Top 5 Most Critical Files

| File | Current | Lines | Gap to 80% | Impact |
|------|---------|-------|------------|--------|
| `src/dsl/executor.rs` | 22.0% | 403/1,830 | 1,061 lines | 🔥 Highest |
| `src/adapters/secondary/subprocess_transport.rs` | 0.9% | 2/229 | 181 lines | 🔥 Critical |
| `src/dsl/validator.rs` | 58.3% | 455/781 | 169 lines | 🔴 High |
| `src/dsl/nl_generator.rs` | 9.6% | 16/166 | 116 lines | 🔴 High |
| `src/dsl/predefined_tasks/sources/git.rs` | 19.1% | 22/115 | 93 lines | 🔴 High |

---

## Recommended Action Plan

### 🎯 Quick Wins (5 hours → +0.68%)
Start here for immediate momentum:

1. `options.rs` - 2 hours → +0.27%
2. `session.rs` - 1 hour → +0.17%
3. `query_fn.rs` - 1 hour → +0.20%
4. Callback adapters - 1 hour → +0.04%

**Total: 5 hours, 65 test lines, +0.68% coverage**

---

### 🔥 Phase 1: Critical Infrastructure (4-6 weeks → 66.5% coverage)

**Focus:** DSL Executor + Adapters Secondary

1. **DSL Executor (Weeks 1-4)**
   - Basic task execution (sequential, parallel)
   - Loop constructs (5 types)
   - State persistence/resumption
   - **Target:** 22% → 60% (+700 lines)
   - **Impact:** +7.3% overall

2. **Subprocess Transport (Weeks 5-6)**
   - Process spawning and NDJSON protocol
   - Mock-based testing
   - **Target:** 0.9% → 70% (+167 lines)
   - **Impact:** +1.7% overall

**Phase 1 Total: +9.0% coverage (57.56% → 66.56%)**

---

### 🔴 Phase 2: High-Value Modules (4-6 weeks → 71.7% coverage)

**Focus:** Validator + Predefined Tasks + Application + Primary Adapters

3. **DSL Validator (Weeks 7-8)** - +1.8%
4. **Predefined Tasks (Weeks 9-10)** - +1.8%
5. **Application Layer (Week 11)** - +0.9%
6. **Primary Adapters (Week 12)** - +0.6%

**Phase 2 Total: +5.1% coverage (66.56% → 71.66%)**

---

### 🟡 Phase 3: Remaining Gaps (3-4 weeks → 74.2% coverage)

**Focus:** Parser, NL Generator, Quick Wins, Testing Utils

7. **Parser & NL Generator (Weeks 13-14)** - +1.7%
8. **Quick Wins (Week 15)** - +0.5%
9. **Testing Utils (Week 16)** - +0.3%

**Phase 3 Total: +2.5% coverage (71.66% → 74.16%)**

---

## Files with Zero Coverage (Highest Priority)

### Adapters (URGENT - Production Risk)
- 🔥 `subprocess_transport.rs` (229 lines) - CLI communication
- 🔥 `sdk_client.rs` (55 lines) - Main public API
- 🔴 `query_fn.rs` (19 lines) - Simple query interface

### Core Configuration
- 🔴 `options.rs` (26 lines) - Agent options
- 🔴 `session.rs` (16 lines) - Session management

### Server (Feature-Gated)
- 🟡 `s3.rs` (32 lines)
- 🟡 `migrations.rs` (13 lines)
- 🟡 `user_storage.rs` (2 lines)

### TUI (Feature-Gated)
- 🟡 `state_browser.rs` (94 lines)
- 🟡 `file_manager.rs` (31 lines)
- 🟡 `editor.rs` (29 lines)
- 🟡 `generator.rs` (29 lines)

---

## Coverage Goals & Timeline

| Milestone | Target | Test Lines Added | Duration | Cumulative Time |
|-----------|--------|------------------|----------|-----------------|
| **Quick Wins** | 58.2% | +65 | 5 hours | 5 hours |
| **Phase 1** | 66.6% | +867 | 4-6 weeks | 4-6 weeks |
| **Phase 2** | 71.7% | +491 | 4-6 weeks | 8-12 weeks |
| **Phase 3** | 74.2% | +242 | 3-4 weeks | 11-16 weeks |
| **Phase 4** | 76.1% | +183 | 3-5 weeks | 14-21 weeks |
| **Final Push** | 80.0% | +385 | 2-4 weeks | 16-25 weeks |

---

## Key Insights

### What's Working Well (Above 70%)
- ✅ **DSL Template System** (100%) - 2,101 lines fully covered
- ✅ **Predefined Task Schema** (100%) - Strong type definitions
- ✅ **Testing Mocks** (69-94%) - Good test infrastructure
- ✅ **DSL Predefined Tasks** (70%) - Close to target

### Critical Gaps (Highest Risk)
- 🔥 **Adapters Layer** (2.5%) - Boundary between domain and external systems
- 🔥 **DSL Executor** (22%) - Core workflow engine, largest file
- 🔴 **Application Query** (25%) - Main orchestration logic

### Feature-Gated (0% Coverage, Lower Priority)
- Server components (requires Docker infrastructure)
- TUI components (requires ratatui test harness)

---

## ROI Analysis

### Highest Impact per Effort Hour

Assuming ~10 test lines per hour:

| Feature | Test Lines | Hours | Coverage Gain | ROI |
|---------|------------|-------|---------------|-----|
| Quick Wins | 65 | 5 | +0.68% | 0.136%/hr |
| DSL Executor | 1,061 | 106 | +11.0% | 0.104%/hr |
| Adapters Secondary | 191 | 19 | +2.0% | 0.105%/hr |
| DSL Validator | 169 | 17 | +1.8% | 0.106%/hr |

**Recommendation:** Start with Quick Wins, then focus on DSL Executor and Adapters in parallel.

---

## CI/CD Integration

### Recommended Coverage Gates

```yaml
# .github/workflows/coverage.yml
coverage:
  overall: 60%  # Current: 57.56%, target: 60%+
  per_feature:
    core: 50%
    dsl: 60%
    adapters: 50%
    testing: 70%
  per_file:
    new_files: 70%
    modified_files: no_decrease
```

### Weekly Monitoring
- Track overall coverage trend (+0.5-1.0% per week target)
- Monitor critical files (executor, transport, validator)
- Alert on coverage decreases in PRs

---

## Next Steps

### Immediate Actions (This Week)
1. ✅ Complete Quick Wins (5 hours)
2. Set up coverage CI gates
3. Create issue tracking for Phase 1 tasks
4. Assign DSL executor testing (Week 1-4)

### Short-Term (Next Month)
1. Complete Phase 1: DSL Executor + Subprocess Transport
2. Reach 65-67% overall coverage
3. Establish weekly coverage review cadence

### Long-Term (Next Quarter)
1. Complete Phase 1-2
2. Reach 70%+ overall coverage
3. All core features above 60%
4. Set up automated coverage reporting

---

## Documentation

Full detailed reports available:
- **`COVERAGE_ANALYSIS.md`** - Complete baseline analysis with module breakdown
- **`COVERAGE_GAPS_BELOW_80.md`** - Detailed action plan with test strategies
- **`coverage/tarpaulin-report.html`** - Interactive HTML report

---

**Last Updated:** 2025-10-26  
**Next Review:** Weekly during Phase 1  
**Owner:** Development Team  
**Coverage Target:** 80% across all features
