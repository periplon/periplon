# Periplon Documentation

Complete documentation for the Periplon Rust SDK - A powerful framework for building multi-agent AI workflows with type-safe, async Rust interfaces.

## 🚀 Getting Started

**New to Periplon?** Start here:

1. 📖 **[DSL Quick Start](guides/DSL_QUICKSTART.md)** - Create your first workflow in 5 minutes
2. 💻 **[CLI Guide](guides/CLI_GUIDE.md)** - Master the command-line interface
3. 🔔 **[Notifications Quick Start](guides/NOTIFICATION_QUICK_START.md)** - Set up notifications
4. 🔄 **[Loop Tutorial](features/loop-tutorial.md)** - Learn iteration patterns

## 📚 Documentation Structure

### [guides/](guides/) - User Tutorials & Quick Starts

Perfect for getting started and learning core concepts:

| Document | Description |
|----------|-------------|
| **[DSL_QUICKSTART.md](guides/DSL_QUICKSTART.md)** | Get started with the DSL system |
| **[STDIO_CONTEXT_QUICKSTART.md](guides/STDIO_CONTEXT_QUICKSTART.md)** | Memory management quick start |
| **[NOTIFICATION_QUICK_START.md](guides/NOTIFICATION_QUICK_START.md)** | Notification system quick start |
| **[predefined-tasks-quickstart.md](guides/predefined-tasks-quickstart.md)** | Using predefined tasks |
| **[CLI_GUIDE.md](guides/CLI_GUIDE.md)** | Command-line interface guide |
| **[CLI_USAGE.md](guides/CLI_USAGE.md)** | CLI usage examples |
| **[ci-cd.md](guides/ci-cd.md)** | CI/CD integration guide |
| **[ci-cd-quick-reference.md](guides/ci-cd-quick-reference.md)** | Quick CI/CD reference |
| **[iterative-pattern-implementation.md](guides/iterative-pattern-implementation.md)** | Implementing iterative patterns |

---

### [features/](features/) - Feature Documentation

Detailed documentation for specific features:

#### 🔄 Loop System
Comprehensive loop and iteration capabilities:

| Document | Description |
|----------|-------------|
| **[loop-tutorial.md](features/loop-tutorial.md)** | Step-by-step loop tutorial |
| **[loop-cookbook.md](features/loop-cookbook.md)** | 25+ real-world loop recipes |
| **[loop-patterns.md](features/loop-patterns.md)** | Advanced loop patterns reference |

#### 🔔 Notifications
Multi-channel notification delivery:

| Document | Description |
|----------|-------------|
| **[notifications.md](features/notifications.md)** | Notification system overview |
| **[notifications_delivery.md](features/notifications_delivery.md)** | Delivery mechanisms (Console, Ntfy, Slack, Discord, File) |

#### 📦 Task Groups
Organize and reuse task groups:

| Document | Description |
|----------|-------------|
| **[README.md](features/README.md)** | Task groups overview |
| **[task-groups-guide.md](features/task-groups-guide.md)** | Task groups user guide |
| **[task-groups-loader.md](features/task-groups-loader.md)** | Task group loader documentation |
| **[api-reference.md](features/api-reference.md)** | Task groups API reference |
| **[architecture.md](features/architecture.md)** | Task groups architecture |
| **[tutorial.md](features/tutorial.md)** | Task groups tutorial |

#### ⚙️ Predefined Tasks
Built-in tasks and lockfile management:

| Document | Description |
|----------|-------------|
| **[predefined-tasks-lockfile.md](features/predefined-tasks-lockfile.md)** | Lockfile management |

#### 🎯 Workflow Features
Core workflow capabilities:

| Document | Description |
|----------|-------------|
| **[conditional-tasks.md](features/conditional-tasks.md)** | Conditional task execution |
| **[subflows.md](features/subflows.md)** | Subflow composition |
| **[definition-of-done.md](features/definition-of-done.md)** | Definition of Done criteria |
| **[TASK_OUTPUT_SYNTAX.md](features/TASK_OUTPUT_SYNTAX.md)** | Task output configuration |

#### 💾 Context & Memory Management
Bounded memory and context control:

| Document | Description |
|----------|-------------|
| **[STDIO_CONTEXT_README.md](features/STDIO_CONTEXT_README.md)** | STDIO and context management |

#### 🌐 Data & HTTP
Data fetching and HTTP operations:

| Document | Description |
|----------|-------------|
| **[DATA_FETCHER_README.md](features/DATA_FETCHER_README.md)** | Data fetching utilities |
| **[HTTP_COLLECTION_SUMMARY.md](features/HTTP_COLLECTION_SUMMARY.md)** | HTTP collection features |

#### 🖥️ User Interfaces
UI and server capabilities:

| Document | Description |
|----------|-------------|
| **[EMBEDDED_WEB_UI.md](features/EMBEDDED_WEB_UI.md)** | Embedded web UI |
| **[server-mode.md](features/server-mode.md)** | Server mode operation |

---

### [api/](api/) - Technical Documentation

Implementation details and API references:

#### 🏗️ DSL System
Domain-Specific Language core:

| Document | Description |
|----------|-------------|
| **[dsl.md](api/dsl.md)** | DSL core documentation |
| **[DSL_FEATURES_INVENTORY.md](api/DSL_FEATURES_INVENTORY.md)** | Complete feature inventory |
| **[DSL_IMPLEMENTATION.md](api/DSL_IMPLEMENTATION.md)** | Implementation details |
| **[DSL_NL_GENERATION.md](api/DSL_NL_GENERATION.md)** | Natural language generation |
| **[dsl-plan.md](api/dsl-plan.md)** | DSL planning and design |

#### ⚙️ Implementation Guides
Component implementations:

| Document | Description |
|----------|-------------|
| **[CLI_IMPLEMENTATION_SUMMARY.md](api/CLI_IMPLEMENTATION_SUMMARY.md)** | CLI implementation |
| **[SERVER_IMPLEMENTATION.md](api/SERVER_IMPLEMENTATION.md)** | Server implementation |
| **[WEB_UI_IMPLEMENTATION.md](api/WEB_UI_IMPLEMENTATION.md)** | Web UI implementation |

#### ⚡ Performance & Security
Optimization and security:

| Document | Description |
|----------|-------------|
| **[PERFORMANCE_OPTIMIZATIONS.md](api/PERFORMANCE_OPTIMIZATIONS.md)** | Performance optimization guide |
| **[SECURITY_AUDIT.md](api/SECURITY_AUDIT.md)** | Security audit and best practices |

---

### [internal/](internal/) - Development Documentation

Internal development notes, summaries, and phase reports:

#### 📋 Analysis & Summaries
Development summaries:

| Document | Description |
|----------|-------------|
| **[ANALYSIS_REPORT.md](internal/ANALYSIS_REPORT.md)** | Analysis reports |
| **[IMPLEMENTATION_SUMMARY.md](internal/IMPLEMENTATION_SUMMARY.md)** | Phase 1-4 implementation summary |

#### 🔔 Notification System Development
Notification implementation details:

| Document | Description |
|----------|-------------|
| **[notification_analysis.md](internal/notification_analysis.md)** | Notification system analysis |
| **[notification_implementation_summary.md](internal/notification_implementation_summary.md)** | Implementation summary |
| **[notification_schema_design.md](internal/notification_schema_design.md)** | Schema design details |
| **[notification_test_results.md](internal/notification_test_results.md)** | Test results |

#### 📦 Predefined Tasks Development
Predefined tasks implementation:

| Document | Description |
|----------|-------------|
| **[predefined-tasks-implementation.md](internal/predefined-tasks-implementation.md)** | Full implementation details |
| **[predefined-tasks-implementation-status.md](internal/predefined-tasks-implementation-status.md)** | Status tracking |
| **[predefined-tasks-phase1-summary.md](internal/predefined-tasks-phase1-summary.md)** | Phase 1 summary |

#### 🗓️ Phase Summaries
Development phase reports:

| Document | Description |
|----------|-------------|
| **[PHASE5_SUMMARY.md](internal/PHASE5_SUMMARY.md)** | Phase 5 development summary |
| **[PHASE6_SUMMARY.md](internal/PHASE6_SUMMARY.md)** | Phase 6 development summary |
| **[PHASE7_SUMMARY.md](internal/PHASE7_SUMMARY.md)** | Phase 7 development summary |
| **[PHASE8_SUMMARY.md](internal/PHASE8_SUMMARY.md)** | Phase 8 error recovery features |
| **[PHASE8_FINAL_SUMMARY.md](internal/PHASE8_FINAL_SUMMARY.md)** | Phase 8 loop documentation |

#### 🐛 Bug Fixes & Updates
Fix summaries:

| Document | Description |
|----------|-------------|
| **[TIMEOUT_FIX_SUMMARY.md](internal/TIMEOUT_FIX_SUMMARY.md)** | Timeout handling fixes |
| **[WORKFLOW_VARIABLE_INTERPOLATION_FIX.md](internal/WORKFLOW_VARIABLE_INTERPOLATION_FIX.md)** | Variable interpolation fixes |

---

### [tui/](tui/) - TUI Testing Documentation

Terminal User Interface testing documentation:

| Document | Description |
|----------|-------------|
| **[README.md](tui/README.md)** | TUI overview |
| **[user-guide.md](tui/user-guide.md)** | TUI user guide |
| **[developer-guide.md](tui/developer-guide.md)** | TUI developer guide |
| **[state_browser_guide.md](tui/state_browser_guide.md)** | State browser guide |
| **[TUI_E2E_INTEGRATION_TESTS.md](tui/TUI_E2E_INTEGRATION_TESTS.md)** | End-to-end tests |
| **[TUI_EDITOR_TESTS.md](tui/TUI_EDITOR_TESTS.md)** | Editor component tests |
| **[TUI_EXECUTION_MONITOR_TESTS.md](tui/TUI_EXECUTION_MONITOR_TESTS.md)** | Execution monitor tests |
| **[TUI_GENERATOR_TESTS.md](tui/TUI_GENERATOR_TESTS.md)** | Generator tests |
| **[TUI_HELP_TESTS.md](tui/TUI_HELP_TESTS.md)** | Help system tests |
| **[TUI_MODAL_TESTS.md](tui/TUI_MODAL_TESTS.md)** | Modal dialog tests |
| **[TUI_STATE_BROWSER_TESTS.md](tui/TUI_STATE_BROWSER_TESTS.md)** | State browser tests |
| **[TUI_TEST_UTILITIES.md](tui/TUI_TEST_UTILITIES.md)** | Testing utilities |
| **[TUI_VIEWER_TESTS.md](tui/TUI_VIEWER_TESTS.md)** | Viewer component tests |
| **[TUI_WORKFLOW_LIST_TESTS.md](tui/TUI_WORKFLOW_LIST_TESTS.md)** | Workflow list tests |
| **[IMPLEMENTATION_SUMMARY.md](tui/IMPLEMENTATION_SUMMARY.md)** | TUI implementation summary |

---

### [archive/](archive/) - Historical Documentation

Historical development notes and deprecated documentation (30 files).

---

## 🎯 Quick Reference

### By Use Case

**Building Your First Workflow**
1. [DSL Quick Start](guides/DSL_QUICKSTART.md)
2. [CLI Guide](guides/CLI_GUIDE.md)
3. [Task Groups Tutorial](features/tutorial.md)

**Working with Loops**
1. [Loop Tutorial](features/loop-tutorial.md) - Learn the basics
2. [Loop Cookbook](features/loop-cookbook.md) - Copy-paste patterns
3. [Loop Patterns](features/loop-patterns.md) - Advanced reference

**Setting Up Notifications**
1. [Quick Start](guides/NOTIFICATION_QUICK_START.md) - 5-minute setup
2. [Feature Overview](features/notifications.md) - All channels
3. [Delivery Guide](features/notifications_delivery.md) - Configuration

**Optimizing Performance**
1. [STDIO Context Guide](features/STDIO_CONTEXT_README.md) - Memory management
2. [Performance Optimizations](api/PERFORMANCE_OPTIMIZATIONS.md) - Best practices

**CI/CD Integration**
1. [CI/CD Guide](guides/ci-cd.md) - Full integration guide
2. [Quick Reference](guides/ci-cd-quick-reference.md) - Common patterns

### By Skill Level

**Beginner** (New to Periplon)
- [DSL Quick Start](guides/DSL_QUICKSTART.md)
- [CLI Guide](guides/CLI_GUIDE.md)
- [Loop Tutorial](features/loop-tutorial.md)
- [Notification Quick Start](guides/NOTIFICATION_QUICK_START.md)

**Intermediate** (Building Workflows)
- [Loop Cookbook](features/loop-cookbook.md)
- [Task Groups Guide](features/task-groups-guide.md)
- [Conditional Tasks](features/conditional-tasks.md)
- [Subflows](features/subflows.md)
- [Definition of Done](features/definition-of-done.md)

**Advanced** (Performance & Architecture)
- [STDIO Context Management](features/STDIO_CONTEXT_README.md)
- [Performance Optimizations](api/PERFORMANCE_OPTIMIZATIONS.md)
- [Security Audit](api/SECURITY_AUDIT.md)
- [Server Implementation](api/SERVER_IMPLEMENTATION.md)
- [DSL Implementation](api/DSL_IMPLEMENTATION.md)

**Expert** (Contributing & Extending)
- [DSL Features Inventory](api/DSL_FEATURES_INVENTORY.md)
- [DSL Natural Language](api/DSL_NL_GENERATION.md)
- [Internal Implementation Summaries](internal/)
- [Phase Development Reports](internal/)

---

## 🔍 Search by Topic

### Agent Configuration
- [DSL Quick Start](guides/DSL_QUICKSTART.md) - Agent basics
- [Task Groups](features/task-groups-guide.md) - Organizing agents

### Task Orchestration
- [DSL Implementation](api/DSL_IMPLEMENTATION.md) - Task graph
- [Conditional Tasks](features/conditional-tasks.md) - Dynamic execution
- [Definition of Done](features/definition-of-done.md) - Task validation

### Loops & Iteration
- [Loop Tutorial](features/loop-tutorial.md) - Learning loops
- [Loop Cookbook](features/loop-cookbook.md) - Recipes
- [Loop Patterns](features/loop-patterns.md) - Reference
- [Iterative Patterns](guides/iterative-pattern-implementation.md) - Implementation

### State Management
- [STDIO Context](features/STDIO_CONTEXT_README.md) - Memory bounds
- [STDIO Quick Start](guides/STDIO_CONTEXT_QUICKSTART.md) - Quick setup
- [Task Output Syntax](features/TASK_OUTPUT_SYNTAX.md) - Output config

### Notifications
- [Notification Quick Start](guides/NOTIFICATION_QUICK_START.md) - Setup
- [Notifications Overview](features/notifications.md) - All channels
- [Delivery Guide](features/notifications_delivery.md) - Configuration

### HTTP & Data
- [HTTP Collections](features/HTTP_COLLECTION_SUMMARY.md) - HTTP tasks
- [Data Fetcher](features/DATA_FETCHER_README.md) - Data utilities

### CLI & Tools
- [CLI Guide](guides/CLI_GUIDE.md) - Complete guide
- [CLI Usage](guides/CLI_USAGE.md) - Examples
- [CLI Implementation](api/CLI_IMPLEMENTATION_SUMMARY.md) - Technical

### Server & Production
- [Server Mode](features/server-mode.md) - Production deployment
- [Server Implementation](api/SERVER_IMPLEMENTATION.md) - Architecture
- [CI/CD Integration](guides/ci-cd.md) - Automation

### Testing
- [TUI Tests](tui/) - TUI testing docs
- [Test Results](internal/notification_test_results.md) - Example results

---

## 📖 Documentation Standards

### File Organization
When adding new documentation:

| Directory | Purpose | Audience |
|-----------|---------|----------|
| **guides/** | Tutorials, quick starts, how-tos | End users, beginners |
| **features/** | Feature-specific documentation | Users learning features |
| **api/** | Technical implementation, API reference | Developers, contributors |
| **internal/** | Development notes, phase summaries | Core team, maintainers |
| **tui/** | TUI testing documentation | TUI developers |
| **archive/** | Historical, deprecated docs | Reference only |

### Writing Guidelines

**Good Documentation:**
- ✅ Clear, descriptive titles
- ✅ Code examples with explanations
- ✅ Step-by-step instructions
- ✅ Links to related documentation
- ✅ Searchable keywords in headings

**File Naming:**
- Use lowercase with hyphens: `my-feature-guide.md`
- Be descriptive: `loop-cookbook.md` not `loops.md`
- Indicate purpose: `*-tutorial.md`, `*-guide.md`, `*-quickstart.md`

**Content Structure:**
1. **Title** - Clear, descriptive
2. **Overview** - What this doc covers
3. **Prerequisites** - What you need to know
4. **Main Content** - Step-by-step or reference
5. **Examples** - Real-world usage
6. **Related Docs** - Links to more info

### Maintenance

When updating documentation:
- ✅ Update this README index
- ✅ Check and update cross-references
- ✅ Verify code examples still work
- ✅ Update version numbers if applicable
- ✅ Archive obsolete documentation

---

## 🤝 Contributing

### Adding New Documentation

1. **Choose the right directory** based on content type
2. **Follow naming conventions** (lowercase-with-hyphens.md)
3. **Include all standard sections** (overview, examples, links)
4. **Add entry to this README** in the appropriate section
5. **Test all code examples** before submitting
6. **Link to related documentation**

### Review Checklist

Before submitting documentation:
- [ ] Clear title and overview
- [ ] Code examples tested
- [ ] Cross-references updated
- [ ] Added to README index
- [ ] Proper directory placement
- [ ] Follows style guidelines
- [ ] No broken links

---

## 📊 Documentation Statistics

- **Total Files**: 121 markdown files
- **User Guides**: 9 quick starts and tutorials
- **Features**: 21 feature documentation files
- **API Docs**: 10 technical references
- **Internal**: 16 development summaries
- **TUI Tests**: 34 test documentation files
- **Archive**: 30 historical documents

---

## 📝 License

Same as Periplon SDK

---

## 🆘 Need Help?

- 🐛 **Found a bug?** Report it in the main repository
- 📖 **Documentation unclear?** Open an issue
- 💡 **Have a suggestion?** We welcome contributions!
- 🚀 **Getting started?** Begin with [DSL Quick Start](guides/DSL_QUICKSTART.md)

---

**Last Updated**: 2025-10-26
**Version**: 1.0.0
