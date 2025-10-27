# Coverage Priority Matrix - Ranked by Strategic Importance

**Date:** 2025-10-26  
**Methodology:** Multi-factor prioritization considering coverage gap, importance, and impact

---

## Priority Scoring Algorithm

**Formula:** `Priority Score = (Coverage Gap × Importance² × Log₁₀(Lines)) ÷ 10`

**Factors:**
1. **Coverage Gap** (0-80%): How far below 80% target
2. **Importance** (1-10): Architectural criticality (squared for exponential weight)
3. **Lines Impact**: Logarithmic scale to balance large vs small files

**Importance Scale:**
- **10** = Critical Infrastructure (executor, subprocess transport)
- **9** = Public APIs (SDK client, query orchestration)
- **8** = Safety-Critical (validator, parser)
- **7** = Core Domain Logic (domain, options, core adapters)
- **6** = DSL Features (predefined tasks, hooks, notifications)
- **5** = Testing Infrastructure, AI Features
- **4** = Server Components (optional feature)
- **3** = TUI Components (optional feature)

---

## 🎯 TIER 1: CRITICAL (Priority Score > 100)

### #1 🔥 DSL-EXECUTOR
**Priority Score:** 1,061.0 | **Importance:** 10/10 | **Impact:** +11.0%

**Current State:**
- Coverage: 22.0% (403/1,830 lines)
- Gap to 80%: 58.0% (1,061 test lines needed)
- **Largest file in codebase**
- **Core workflow execution engine**

**Why Critical:**
- Central orchestrator for all workflow execution
- Handles task scheduling, dependencies, loops, state
- Single point of failure for entire DSL system
- 1,830 lines = 19% of total codebase

**Test Priority:**
1. Basic task execution (sequential, parallel)
2. Dependency resolution and topological sorting
3. Loop constructs (5 types: for, foreach, while, repeat, polling)
4. State checkpointing and resumption
5. Error handling and recovery
6. Agent communication via message bus
7. Notification delivery triggers
8. Variable context management
9. Timeout and cancellation handling
10. Subflow execution

**Effort:** 3-4 weeks full-time  
**ROI:** Highest - 11% coverage gain from single file

---

## 🔴 TIER 2: HIGH PRIORITY (Score 10-100)

### #2 🔥 ADAPTERS-SECONDARY  
**Priority Score:** 146.8 | **Importance:** 10/10 | **Impact:** +2.0%

**Current State:**
- Coverage: 3.2% (8/249 lines)
- Gap to 80%: 76.8% (191 test lines needed)
- **Boundary layer** between domain and external systems

**Critical File Breakdown:**
| File | Coverage | Importance | Lines Needed |
|------|----------|------------|--------------|
| `subprocess_transport.rs` | 0.9% (2/229) | 10/10 | 181 lines |
| `mock_transport.rs` | 37.5% (6/16) | 6/10 | 10 lines |
| `callback_hook.rs` | 0.0% (0/3) | 6/10 | 2 lines |
| `callback_permission.rs` | 0.0% (0/1) | 6/10 | 1 line |

**Why High Priority:**
- `subprocess_transport.rs` is **URGENT** - CLI communication layer
- Production risk: boundary between SDK and external CLI
- Zero coverage on process spawning, NDJSON protocol
- Transport failures = complete system failure

**Test Priority (subprocess_transport.rs):**
1. Process spawning and lifecycle management
2. NDJSON serialization/deserialization
3. Stdin/stdout communication
4. Timeout scenarios
5. Process crash handling
6. Malformed message handling
7. Version compatibility checks
8. CLI discovery logic

**Effort:** 1-2 weeks  
**ROI:** High - critical boundary layer protection

---

### #3 🔴 APPLICATION
**Priority Score:** 43.9 | **Importance:** 9/10 | **Impact:** +0.9%

**Current State:**
- File: `src/application/query.rs`
- Coverage: 25.6% (42/164 lines)
- Gap to 80%: 54.4% (89 test lines needed)
- **Main orchestration logic**

**Why High Priority:**
- Coordinates domain, adapters, and infrastructure
- Implements query lifecycle (start, execute, complete)
- Hook integration points
- Permission evaluation
- Error propagation

**Test Priority:**
1. Query lifecycle end-to-end
2. Hook invocation (on_start, on_complete, on_error)
3. Permission callback integration
4. Multi-turn conversation handling
5. Error propagation from transport
6. Timeout handling
7. Session management integration

**Effort:** 3-5 days  
**ROI:** Medium - impacts all user-facing operations

---

### #4 🔴 ADAPTERS-PRIMARY
**Priority Score:** 42.5 | **Importance:** 9/10 | **Impact:** +0.6%

**Current State:**
- Coverage: 0.0% (0/74 lines)
- Gap to 80%: 80.0% (59 test lines needed)
- **Main public API interfaces**

**Files:**
| File | Lines | Importance | Purpose |
|------|-------|------------|---------|
| `sdk_client.rs` | 55 | 9/10 | Multi-turn conversation interface |
| `query_fn.rs` | 19 | 9/10 | Simple one-shot query function |

**Why High Priority:**
- Public APIs that users directly interact with
- Zero coverage = high risk for breaking changes
- Entry points to entire SDK

**Test Priority:**
1. Simple `query()` function test
2. SDK client initialization
3. Multi-turn conversation flow
4. Session management
5. Permission callback handling
6. Error handling and propagation
7. Stream handling

**Effort:** 2-3 days  
**ROI:** High - protects public API contracts

---

### #5 🔴 DSL-VALIDATOR
**Priority Score:** 28.3 | **Importance:** 8/10 | **Impact:** +1.8%

**Current State:**
- File: `src/dsl/validator.rs`
- Coverage: 58.3% (455/781 lines)
- Gap to 80%: 21.7% (169 test lines needed)
- **Safety-critical component**

**Why High Priority:**
- Prevents invalid workflows from executing
- Catches cycles, undefined references, type errors
- Already at 58% - achievable target

**Test Priority:**
1. Circular dependency detection (simple & complex)
2. Undefined agent references
3. Undefined task references in dependencies
4. Invalid variable syntax
5. Undefined variable references ($scope.var)
6. Invalid loop constructs
7. Invalid notification channels
8. Missing required fields
9. Type validation (string vs number)
10. Deep nesting validation

**Effort:** 1 week  
**ROI:** High - workflow safety critical

---

### #6 🔴 DSL-PREDEFINED-TASKS
**Priority Score:** 19.7 | **Importance:** 8/10 | **Impact:** +1.8%

**Current State:**
- Coverage: 70.4% (1,286/1,826 lines)
- Gap to 80%: 9.6% (174 test lines needed)
- **Close to target** - only 9.6% gap

**Critical Gap: Git Source Integration**
- `sources/git.rs`: 19.1% (22/115) - **93 lines needed**
- Importance: 8/10 (external system integration)

**Other Low Coverage:**
| File | Coverage | Lines Needed |
|------|----------|--------------|
| `discovery.rs` | 43.0% | 50 |
| `groups/parser.rs` | 66.9% | 32 |
| `groups/namespace.rs` | 58.6% | 19 |
| `groups/loader.rs` | 63.2% | 32 |

**Test Priority (git.rs):**
1. Git clone operations
2. Branch/tag checkout
3. Repository validation
4. Network error handling
5. Authentication (SSH, HTTPS)
6. Submodule handling
7. Shallow clone support

**Effort:** 1 week  
**ROI:** Medium - important for task reuse ecosystem

---

### #7 🔴 DSL-PARSER
**Priority Score:** 12.3 | **Importance:** 8/10 | **Impact:** +0.5%

**Current State:**
- File: `src/dsl/parser.rs`
- Coverage: 13.2% (10/76 lines)
- Gap to 80%: 66.8% (50 test lines needed)
- **Input validation layer**

**Why High Priority:**
- First line of defense against malformed YAML
- Low coverage despite importance
- Safety-critical parsing logic

**Test Priority:**
1. Invalid YAML syntax handling
2. Missing required fields detection
3. Type mismatch errors
4. Unknown field warnings
5. Complex nested structures
6. Edge cases (empty arrays, null values)

**Effort:** 2-3 days  
**ROI:** Medium - input validation critical

---

## 🟠 TIER 3: MEDIUM PRIORITY (Score 5-10)

### #8 🟠 DSL-NL-GENERATOR
**Priority Score:** 6.8 | **Importance:** 5/10 | **Impact:** +1.2%

**Current State:**
- File: `src/dsl/nl_generator.rs`
- Coverage: 9.6% (16/166 lines)
- Gap to 80%: 70.4% (116 test lines needed)

**Why Medium Priority:**
- AI-powered feature (less deterministic)
- Not critical path
- Can be mocked for tests

**Test Priority:**
1. Mock LLM responses
2. YAML output validation
3. Error handling for malformed output
4. Natural language parsing logic

**Effort:** 3-5 days

---

### #9 🟠 OTHER (options.rs focus)
**Priority Score:** 5.3 | **Importance:** 7/10 | **Impact:** +0.4%

**Current State:**
- Coverage: 49.1% (56/114 lines)
- **Critical file:** `options.rs` at 0% (26 lines) - importance 7/10

**Why Medium Priority:**
- `options.rs` is core configuration module
- Quick win (26 lines)
- Impacts all agent initialization

**Test Priority:**
1. AgentOptions initialization
2. Permission mode defaults
3. System prompt handling
4. Tool allowlist configuration

**Effort:** 2 hours (quick win)

---

### #10 🟠 DOMAIN
**Priority Score:** 4.8 | **Importance:** 7/10 | **Impact:** +0.1%

**Current State:**
- Coverage: 33.3% (8/24 lines)
- **Critical file:** `session.rs` at 0% (16 lines)

**Why Medium Priority:**
- Small module (24 lines total)
- Core domain logic
- `session.rs` needs basic tests

**Test Priority:**
1. Session lifecycle (create, update, close)
2. Session state management
3. ID generation

**Effort:** 2-3 hours (quick win)

---

## 🟡 TIER 4: LOW PRIORITY (Score < 5)

### #11 🟡 TUI
**Priority Score:** 3.5 | **Importance:** 3/10 | **Impact:** +1.5%

**Current State:**
- Coverage: 0.0% (0/183 lines)
- Feature-gated behind `tui` flag
- Optional user interface

**Why Low Priority:**
- Not critical path
- Feature-gated (optional)
- Requires complex test harness setup (ratatui)
- May deprioritize if CLI-first approach

**Effort:** 2-3 weeks (includes harness setup)

---

### #12 🟡 TESTING
**Priority Score:** 1.6 | **Importance:** 5/10 | **Impact:** +0.3%

**Current State:**
- Coverage: 69.3% (196/283 lines)
- Already strong coverage
- Test infrastructure helpers

**Effort:** 1 day for remaining edge cases

---

### #13 🟡 SERVER
**Priority Score:** 1.2 | **Importance:** 4/10 | **Impact:** +0.4%

**Current State:**
- Coverage: 0.0% (0/47 lines)
- Feature-gated behind `server` flag
- Optional deployment mode

**Why Low Priority:**
- Requires Docker infrastructure (PostgreSQL, Redis, S3)
- Optional feature
- Can be integration-tested separately

**Effort:** 1 week (with infrastructure)

---

## 📊 Summary Matrix

| Tier | Features | Test Lines | Coverage Impact | Effort | ROI |
|------|----------|------------|-----------------|--------|-----|
| 🔥 **CRITICAL** | 1 | 1,061 | +11.0% | 3-4 weeks | **Highest** |
| 🔴 **HIGH** | 6 | 732 | +7.5% | 5-8 weeks | **High** |
| 🟠 **MEDIUM** | 3 | 163 | +1.7% | 1-2 weeks | **Medium** |
| 🟡 **LOW** | 3 | 213 | +2.2% | 3-5 weeks | **Low** |
| **TOTAL** | **13** | **2,169** | **+22.4%** | **12-19 weeks** | - |

---

## 🎯 Recommended Execution Sequence

### Phase 1: Critical Foundation (4-6 weeks) → 68% coverage
1. **DSL Executor** (Weeks 1-4)
   - Target: 22% → 60% (+700 lines)
   - Impact: +7.3% overall
   - **Highest priority by far**

2. **Subprocess Transport** (Weeks 5-6)
   - Target: 0.9% → 70% (+167 lines)
   - Impact: +1.7% overall
   - **URGENT - production risk**

**Phase 1 Impact:** +9.0% (57.56% → 66.56%)

---

### Phase 2: High-Value APIs & Safety (4-6 weeks) → 74% coverage
3. **Application Query** (Week 7)
   - Target: 25.6% → 80% (+89 lines)
   - Impact: +0.9%

4. **Adapters Primary** (Week 8)
   - Target: 0% → 80% (+59 lines)
   - Impact: +0.6%

5. **DSL Validator** (Weeks 9-10)
   - Target: 58.3% → 80% (+169 lines)
   - Impact: +1.8%

6. **DSL Predefined Tasks** (Weeks 11-12)
   - Target: 70.4% → 80% (+174 lines)
   - Impact: +1.8%

**Phase 2 Impact:** +5.1% (66.56% → 71.66%)

---

### Phase 3: Parser & Quick Wins (2-3 weeks) → 76% coverage
7. **DSL Parser** (Week 13)
   - Target: 13.2% → 80% (+50 lines)
   - Impact: +0.5%

8. **Quick Wins** (Week 14)
   - `options.rs` (26 lines) - 2 hours
   - `session.rs` (16 lines) - 2 hours
   - `query_fn.rs` in adapters-primary (covered above)
   - Impact: +0.4%

9. **DSL NL Generator** (Week 15)
   - Target: 9.6% → 60% (+84 lines)
   - Impact: +0.9%

**Phase 3 Impact:** +1.8% (71.66% → 73.46%)

---

### Phase 4: Optional Features (3-5 weeks, as needed)
10. **TUI** (optional, if prioritized)
    - Requires test harness setup
    - Impact: +1.5%

11. **Server** (optional, if prioritized)
    - Requires Docker infrastructure
    - Impact: +0.4%

---

## 🔑 Key Insights

### Architectural Criticality Analysis

**10/10 Importance (Must Fix URGENTLY):**
- DSL Executor (22%) - **Core engine**
- Subprocess Transport (0.9%) - **Critical boundary**

**9/10 Importance (High Priority):**
- Application Query (25.6%) - Main orchestration
- Primary Adapters (0%) - Public API

**8/10 Importance (Safety-Critical):**
- DSL Validator (58.3%) - Prevents bad workflows
- DSL Parser (13.2%) - Input validation
- Predefined Tasks Git Source (19.1%) - External integration

**7/10 Importance (Core Domain):**
- Domain Session (0%) - Quick win
- Options Module (0%) - Quick win
- Other adapters (various)

**≤6/10 Importance (Features & Optional):**
- Testing utils (69.3%) - Already strong
- NL Generator (9.6%) - AI feature, less critical
- Server (0%) - Feature-gated
- TUI (0%) - Feature-gated

---

## 💡 Strategic Recommendations

### Start With:
1. ✅ **Quick Wins** (1 day) - `options.rs`, `session.rs` → +0.4%
2. 🔥 **DSL Executor** (3-4 weeks) → +7-11% (target 60-80%)
3. 🔥 **Subprocess Transport** (1-2 weeks) → +2%

### Then Focus On:
4. 🔴 Application + Primary Adapters (2 weeks) → +1.5%
5. 🔴 Validator + Predefined Tasks (4 weeks) → +3.6%
6. 🔴 Parser (1 week) → +0.5%

### Lower Priority (As Time Permits):
7. 🟠 NL Generator, Testing helpers
8. 🟡 Feature-gated components (TUI, Server)

---

## 📈 Expected Outcome

**Following this priority order:**
- **12 weeks:** Reach 71-73% coverage (Phases 1-2 + Parser)
- **16 weeks:** Reach 75%+ coverage (add quick wins + NL generator)
- **All critical infrastructure** (importance 8-10) above 70%
- **All public APIs** fully tested
- **Production risk** minimized (adapters, transport, validator)

**ROI Comparison:**
- Best: DSL Executor - 0.104%/hour (assuming 10 lines/hour)
- Good: Adapters, Validator - 0.10-0.11%/hour
- Quick Wins: 0.136%/hour (highest ROI but small total impact)

**Recommendation:** Start with Quick Wins for momentum, then attack DSL Executor and Subprocess Transport in parallel with different team members.

---

**Last Updated:** 2025-10-26  
**Methodology:** Multi-factor strategic prioritization  
**Review Cadence:** Weekly during Phase 1-2 execution
