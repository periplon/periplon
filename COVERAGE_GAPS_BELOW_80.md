# Coverage Gaps Below 80% - Action Plan

**Generated:** 2025-10-26  
**Overall Coverage:** 57.56%  
**Features Below 80%:** 13 out of 14 (92.9%)  
**Total Test Lines Needed:** 2,168 to reach 80% across all features

---

## Executive Summary

**13 out of 14 features** (92.9%) are below the 80% coverage target, requiring **2,168 lines of additional tests**. 

### Priority Classification:
- 🔥 **Critical Impact (Top 2):** 1,252 lines needed (57.8% of total effort)
- 🔴 **High Priority (Next 3):** 489 lines needed (22.5% of total effort)  
- 🟡 **Medium Priority (Remaining 8):** 427 lines needed (19.7% of total effort)

---

## 🔥 CRITICAL PRIORITY (Top 2 by Impact)

### 1. DSL-EXECUTOR (22.0% coverage)

**Impact:** 🔥 **HIGHEST** - 1,061 lines needed (48.9% of total effort)

**Current State:**
- File: `src/dsl/executor.rs`
- Coverage: 22.0% (403/1,830 lines)
- Largest file in codebase
- Core workflow execution engine

**Gap Analysis:**
- Target: 80% (1,464 lines covered)
- Current: 403 lines covered
- **Need: 1,061 additional test lines**

**What's Missing:**
- Task execution paths (sequential, parallel, dependent)
- Loop execution (for, foreach, while, repeat, polling)
- State checkpointing and resumption
- Error handling and recovery
- Notification delivery integration
- Agent orchestration scenarios
- Subflow execution
- Conditional execution
- Variable interpolation in execution context

**Recommended Test Strategy:**
```rust
// Priority test cases:
1. Basic task execution (sequential)
2. Parallel task execution
3. Task dependency resolution
4. Loop constructs (all 5 types)
5. State persistence/resumption
6. Error propagation
7. Agent communication via message bus
8. Notification triggers
9. Variable context management
10. Timeout handling
```

**Effort Estimate:** 3-4 weeks (full-time)

**Impact on Overall Coverage:** +11.0% (1,061/9,623)

---

### 2. ADAPTERS-SECONDARY (3.2% coverage)

**Impact:** 🔥 **CRITICAL** - 191 lines needed (8.8% of total effort)

**Current State:**
- Files: 4 files, 249 total lines
- Coverage: 3.2% (8/249 lines)
- **Boundary layer** between domain and external systems

**Files Breakdown:**
| File | Coverage | Lines | Priority |
|------|----------|-------|----------|
| `subprocess_transport.rs` | 0.9% | 2/229 | 🔥 URGENT |
| `mock_transport.rs` | 37.5% | 6/16 | 🟡 Medium |
| `callback_hook.rs` | 0.0% | 0/3 | 🔴 High |
| `callback_permission.rs` | 0.0% | 0/1 | 🔴 High |

**Gap Analysis:**
- Target: 80% (199 lines covered)
- Current: 8 lines covered
- **Need: 191 additional test lines**

**What's Missing:**

**subprocess_transport.rs (URGENT - 181 lines needed):**
- Process spawning and lifecycle
- NDJSON protocol handling (send/receive)
- Stdin/stdout communication
- Timeout scenarios
- Error handling (process crash, invalid JSON)
- Version checking
- CLI discovery logic

**mock_transport.rs (10 lines needed):**
- Edge case scenarios
- Error injection
- Response queuing

**callback_hook.rs (2 lines needed):**
- Hook invocation
- Async callback execution

**callback_permission.rs (1 line needed):**
- Permission callback test

**Recommended Test Strategy:**
```rust
// subprocess_transport.rs tests:
1. Mock subprocess with controlled responses
2. Test NDJSON serialization/deserialization
3. Test timeout handling
4. Test process crash scenarios
5. Test malformed message handling
6. Test version compatibility checks
7. Test CLI discovery paths

// Integration tests:
8. Full round-trip communication
9. Multi-message sequences
10. Stream handling
```

**Effort Estimate:** 1-2 weeks

**Impact on Overall Coverage:** +2.0% (191/9,623)

---

## 🔴 HIGH PRIORITY (Next 3 by Impact)

### 3. DSL-PREDEFINED-TASKS (70.4% coverage)

**Impact:** 🔴 High - 174 lines needed (8.0% of total effort)

**Current State:**
- Files: 19 files
- Coverage: 70.4% (1,286/1,826 lines)
- **Close to target** - only 9.6% gap

**Critical Gap:**
- `sources/git.rs`: 19.1% (22/115) - **93 lines needed**

**Other Low Coverage Files:**
| File | Coverage | Lines Needed |
|------|----------|--------------|
| `discovery.rs` | 43.0% | 50 lines |
| `sources/mod.rs` | 58.3% | 3 lines |
| `groups/namespace.rs` | 58.6% | 19 lines |
| `groups/loader.rs` | 63.2% | 32 lines |

**Focus Area: Git Source Integration**

**What's Missing in git.rs:**
- Git clone operations
- Branch/tag checkout
- Repository validation
- Network error handling
- Authentication scenarios
- Submodule handling
- Shallow clone support

**Effort Estimate:** 1 week

**Impact on Overall Coverage:** +1.8% (174/9,623)

---

### 4. DSL-VALIDATOR (58.3% coverage)

**Impact:** 🔴 High - 169 lines needed (7.8% of total effort)

**Current State:**
- File: `src/dsl/validator.rs`
- Coverage: 58.3% (455/781 lines)
- Critical for workflow safety

**Gap Analysis:**
- Target: 80% (624 lines covered)
- Current: 455 lines covered
- **Need: 169 additional test lines**

**What's Missing:**
- Cycle detection in task dependencies
- Undefined variable reference checks
- Agent reference validation
- Tool availability validation
- Permission mode validation
- Loop condition validation
- Notification channel validation
- Complex nested structure validation

**Recommended Test Strategy:**
```yaml
# Test cases needed:
1. Circular dependency detection (simple & complex)
2. Undefined agent references
3. Undefined task references in dependencies
4. Invalid variable syntax
5. Undefined variable references
6. Invalid loop constructs
7. Invalid notification channels
8. Missing required fields
9. Type validation (string vs number)
10. Deep nesting validation
```

**Effort Estimate:** 1 week

**Impact on Overall Coverage:** +1.8% (169/9,623)

---

### 5. TUI (0.0% coverage)

**Impact:** 🔴 High - 146 lines needed (6.7% of total effort)

**Current State:**
- Files: 4 view files
- Coverage: 0.0% (0/183 lines)
- Feature-gated behind `tui` flag

**Files:**
| File | Lines | Priority |
|------|-------|----------|
| `state_browser.rs` | 94 | High |
| `file_manager.rs` | 31 | Medium |
| `editor.rs` | 29 | Medium |
| `generator.rs` | 29 | Medium |

**Gap Analysis:**
- Target: 80% (146 lines covered)
- Current: 0 lines covered
- **Need: 146 additional test lines**

**Challenge:** Requires ratatui test harness setup

**What's Missing:**
- View rendering tests
- Keyboard event handling
- State transitions
- Modal interactions
- Navigation flows
- Input validation

**Recommended Test Strategy:**
```rust
// Setup: Create ratatui test harness
1. Virtual terminal backend
2. Snapshot testing framework

// Test cases:
3. View rendering (snapshot tests)
4. Keyboard navigation (up/down/enter/esc)
5. State transitions between views
6. Modal open/close/confirm
7. Text input handling
8. Error display
9. Help system navigation
```

**Effort Estimate:** 2-3 weeks (includes harness setup)

**Impact on Overall Coverage:** +1.5% (146/9,623)

**Note:** May deprioritize if CLI-first approach taken.

---

## 🟡 MEDIUM PRIORITY (Remaining 8 Features)

### 6. DSL-NL-GENERATOR (9.6% coverage) - 116 lines needed

**Current:** 16/166 lines  
**File:** `src/dsl/nl_generator.rs`  

**What's Missing:**
- LLM integration tests (mock responses)
- YAML output validation
- Error handling for malformed LLM output
- Natural language parsing logic

**Effort:** 3-5 days  
**Impact:** +1.2% overall coverage

---

### 7. APPLICATION (25.6% coverage) - 89 lines needed

**Current:** 42/164 lines  
**File:** `src/application/query.rs`  

**What's Missing:**
- Query lifecycle end-to-end tests
- Hook integration (on_start, on_complete, on_error)
- Permission evaluation during execution
- Multi-turn conversation scenarios

**Effort:** 3-5 days  
**Impact:** +0.9% overall coverage

---

### 8. ADAPTERS-PRIMARY (0.0% coverage) - 59 lines needed

**Current:** 0/74 lines  
**Files:**
- `query_fn.rs`: 0/19
- `sdk_client.rs`: 0/55

**What's Missing:**
- Simple query function tests
- SDK client integration tests
- Session management tests
- Multi-turn conversation tests
- Permission callback tests

**Effort:** 2-3 days  
**Impact:** +0.6% overall coverage

---

### 9. DSL-PARSER (13.2% coverage) - 50 lines needed

**Current:** 10/76 lines  
**File:** `src/dsl/parser.rs`  

**What's Missing:**
- Invalid YAML format handling
- Missing required field detection
- Type validation
- Complex nested structure parsing

**Effort:** 2-3 days  
**Impact:** +0.5% overall coverage

---

### 10. SERVER (0.0% coverage) - 37 lines needed

**Current:** 0/47 lines  
**Files:**
- `migrations.rs`: 0/13
- `s3.rs`: 0/32
- `user_storage.rs`: 0/2

**What's Missing:**
- Database migration tests
- S3 storage operations (mock client)
- User storage interface tests

**Challenge:** Requires Docker infrastructure

**Effort:** 1 week (with infrastructure setup)  
**Impact:** +0.4% overall coverage

---

### 11. OTHER (49.1% coverage) - 35 lines needed

**Current:** 56/114 lines  
**Files:**
- `options.rs`: 0/26 (HIGH PRIORITY)
- `data_fetcher.rs`: 56/88

**What's Missing:**
- **options.rs:** AgentOptions tests, permission mode defaults
- **data_fetcher.rs:** HTTP auth, error handling edge cases

**Effort:** 1-2 days  
**Impact:** +0.4% overall coverage

---

### 12. TESTING (69.3% coverage) - 30 lines needed

**Current:** 196/283 lines  

**What's Missing:**
- Test helper edge cases
- Mock service edge cases

**Effort:** 1 day  
**Impact:** +0.3% overall coverage

---

### 13. DOMAIN (33.3% coverage) - 11 lines needed

**Current:** 8/24 lines  
**Files:**
- `session.rs`: 0/16 (needs tests)
- `message.rs`: 8/8 (complete)

**What's Missing:**
- Session lifecycle tests
- Session state management

**Effort:** 2-3 hours  
**Impact:** +0.1% overall coverage

---

## Prioritized Action Plan

### Phase 1: Critical Infrastructure (4-6 weeks)
**Goal:** Reach 65% overall coverage (+7.5%)

1. **DSL Executor - Week 1-4**
   - Focus on basic task execution (sequential, parallel)
   - Loop constructs (5 types)
   - State management
   - **Target:** Improve from 22% → 60% (+700 test lines)
   - **Impact:** +7.3% overall

2. **Adapters Secondary - Week 5-6**
   - Focus on subprocess_transport.rs
   - Mock-based testing
   - **Target:** Improve from 3% → 70% (+167 test lines)
   - **Impact:** +1.7% overall

**Phase 1 Total Impact:** +9.0% overall (867 test lines)

---

### Phase 2: High-Value Modules (4-6 weeks)
**Goal:** Reach 72% overall coverage (+14.5%)

3. **DSL Validator - Week 7-8**
   - Cycle detection
   - Variable validation
   - **Target:** 58% → 80% (+169 test lines)
   - **Impact:** +1.8% overall

4. **DSL Predefined Tasks - Week 9-10**
   - Focus on git.rs
   - Discovery and loader edge cases
   - **Target:** 70% → 80% (+174 test lines)
   - **Impact:** +1.8% overall

5. **Application Layer - Week 11**
   - Query orchestration
   - Hook integration
   - **Target:** 26% → 80% (+89 test lines)
   - **Impact:** +0.9% overall

6. **Adapters Primary - Week 12**
   - SDK client tests
   - Query function tests
   - **Target:** 0% → 80% (+59 test lines)
   - **Impact:** +0.6% overall

**Phase 2 Total Impact:** +5.1% overall (491 test lines)

---

### Phase 3: Remaining Gaps (3-4 weeks)
**Goal:** Reach 78% overall coverage (+20.5%)

7. **DSL Parser & NL Generator - Week 13-14**
   - Parser edge cases (+50 lines)
   - NL generator mocks (+116 lines)
   - **Impact:** +1.7% overall

8. **Quick Wins - Week 15**
   - options.rs (+26 lines)
   - session.rs (+16 lines)
   - callback adapters (+4 lines)
   - **Impact:** +0.5% overall

9. **Testing Utils - Week 16**
   - Test helpers edge cases (+30 lines)
   - **Impact:** +0.3% overall

**Phase 3 Total Impact:** +2.5% overall (242 test lines)

---

### Phase 4: Feature-Gated (Optional - 3-5 weeks)

10. **TUI Testing Infrastructure**
    - Set up ratatui test harness
    - View rendering tests
    - **Target:** 0% → 80% (+146 lines)
    - **Impact:** +1.5% overall

11. **Server Testing Infrastructure**
    - Docker setup (PostgreSQL/Redis/S3)
    - Integration tests
    - **Target:** 0% → 80% (+37 lines)
    - **Impact:** +0.4% overall

**Phase 4 Total Impact:** +1.9% overall (183 test lines)

---

## Summary Timeline & Impact

| Phase | Duration | Test Lines | Coverage Gain | Target Coverage |
|-------|----------|------------|---------------|-----------------|
| **Current** | - | 5,539 | - | 57.56% |
| **Phase 1** | 4-6 weeks | +867 | +9.0% | 66.56% |
| **Phase 2** | 4-6 weeks | +491 | +5.1% | 71.66% |
| **Phase 3** | 3-4 weeks | +242 | +2.5% | 74.16% |
| **Phase 4** | 3-5 weeks | +183 | +1.9% | 76.06% |
| **Total** | **14-21 weeks** | **+1,783** | **+18.5%** | **76.06%** |

**Remaining gap to 80%:** 385 test lines (DSL executor edge cases, end-to-end tests)

---

## Quick Wins (1-2 Days, High ROI)

These provide immediate coverage improvement with minimal effort:

1. **options.rs** (26 lines) - 2 hours
   - AgentOptions tests
   - Permission mode tests
   - **Impact:** +0.27%

2. **session.rs** (16 lines) - 1 hour
   - Session lifecycle tests
   - **Impact:** +0.17%

3. **Callback adapters** (4 lines) - 1 hour
   - Permission callback test
   - Hook callback test
   - **Impact:** +0.04%

4. **query_fn.rs** (19 lines) - 1 hour
   - Simple query function test
   - **Impact:** +0.20%

**Total Quick Wins:** 5 hours → +0.68% coverage (65 lines)

---

## Automation & CI Integration

### Recommended CI Checks:

1. **Coverage Gate:**
   - Block PRs that decrease overall coverage
   - Require 70%+ coverage for new files
   - Warn on files below 50%

2. **Feature Coverage Thresholds:**
   ```yaml
   core: 60%
   dsl: 70%
   adapters: 60%
   testing: 75%
   ```

3. **Coverage Reports:**
   - Generate HTML report on every PR
   - Comment PR with coverage diff
   - Track coverage trend over time

4. **Weekly Full Runs:**
   - Test all features (cli + server + tui)
   - Generate comprehensive report
   - Update coverage badge

---

## Metrics to Track

### Weekly Monitoring:

1. **Overall Coverage Trend**
   - Target: +0.5-1.0% per week during active test development

2. **Critical File Coverage**
   - executor.rs: Track weekly progress toward 60%
   - subprocess_transport.rs: Track toward 70%
   - validator.rs: Track toward 80%

3. **Feature Balance**
   - No feature below 50% (except feature-gated)
   - Core modules (domain, application, adapters) above 60%
   - DSL system above 70%

4. **New Code Coverage**
   - All new PRs: 70%+ coverage
   - Critical paths: 90%+ coverage

---

## Conclusion

**Current State:**
- 13/14 features below 80% (92.9%)
- 2,168 test lines needed to reach 80% everywhere

**Realistic Short-Term Goal (3 months):**
- Focus on Phase 1 & 2
- Reach 70-72% overall coverage
- Cover critical infrastructure (executor, adapters)
- **Effort:** 8-12 weeks

**Long-Term Goal (6 months):**
- Complete Phase 1-3
- Reach 75%+ overall coverage
- All core features above 70%
- **Effort:** 14-21 weeks

**Highest Impact Actions:**
1. 🔥 DSL Executor: 1,061 lines → +11.0% coverage
2. 🔥 Adapters Secondary: 191 lines → +2.0% coverage
3. 🔴 DSL Validator: 169 lines → +1.8% coverage

**Start with Quick Wins:** 5 hours of work for +0.68% coverage boost to build momentum!

---

**Generated:** 2025-10-26  
**Next Review:** Weekly during active test development  
**Coverage Goal:** 80% across all features
