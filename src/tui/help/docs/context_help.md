# Context-Sensitive Help

Help content organized by view and context.

## Workflows View

### File Browser Context

You are browsing workflow files. Use this view to manage your workflows.

**Available Actions:**
- `↑↓` - Navigate files and folders
- `Enter` - Open workflow or expand folder
- `Space` - Quick preview
- `n` - Create new workflow (template, AI, or blank)
- `e` - Edit selected workflow
- `v` - View workflow details (read-only)
- `x` - Execute workflow
- `d` - Delete workflow (with confirmation)
- `r` - Rename workflow
- `c` - Copy workflow
- `f` - Filter/search by name
- `/` - Quick search
- `.` - Toggle hidden files

**Tips:**
- Double-click workflow name to quickly open
- Use `Ctrl+X` for quick execution
- Navigate directories with arrow keys
- Backspace goes to parent directory

### Workflow Creation Context

Choose how to create your new workflow:

**1. From Template**
- Start with a pre-built workflow structure
- Modify to suit your needs
- Fastest way for common patterns

**2. Generate with AI**
- Describe what you want in natural language
- AI creates complete workflow
- Review and edit before saving

**3. Blank Workflow**
- Start with minimal template
- Full control over structure
- Best for experienced users

**Examples of AI Prompts:**
- "Analyze code quality and generate report"
- "Multi-agent research workflow with summarization"
- "Automated code review with specialized agents"

## Workflow Editor

### Editing Context

You are editing a workflow YAML file. The editor provides syntax highlighting and validation.

**Editing Commands:**
- `Ctrl+S` - Save changes
- `Ctrl+W` - Save and close
- `Esc` - Cancel (confirms if modified)
- `Ctrl+Z` - Undo
- `Ctrl+Y` - Redo
- `Ctrl+F` - Find text
- `Ctrl+H` - Find and replace
- `Tab` - Indent selection
- `Shift+Tab` - Unindent selection

**Auto-completion:**
- `Ctrl+Space` - Show suggestions
- `↑↓` - Navigate suggestions
- `Enter` - Accept suggestion
- `Esc` - Close suggestions

**Available Completions:**
- Agent names (when typing `agent: `)
- Task references (in `depends_on:`)
- Tool names (in `tools: []`)
- Permission modes
- Variable references

**Validation:**

The editor validates your workflow in real-time:
- ✓ Green: Valid workflow
- ⚠ Yellow: Warnings (non-blocking)
- ✗ Red: Errors (must fix)

Common errors:
- Missing required fields (name, agents, tasks)
- Invalid agent references
- Circular task dependencies
- Undefined variable references
- Invalid tool names

**Tips:**
- Use `F8` to jump to next error
- Status bar shows validation results
- Save frequently with `Ctrl+S`
- Use auto-completion for accuracy

## Execution View

### Monitoring Context

You are monitoring a workflow execution. View real-time progress and logs.

**Execution Control:**
- `p` - Pause execution
- `r` - Resume paused execution
- `s` - Stop execution (cannot resume)
- `k` - Kill (force stop)
- `Esc` - Background execution (keeps running)

**Log Navigation:**
- `↑↓` - Scroll logs
- `PageUp/Down` - Fast scroll
- `Home` - Jump to start
- `End` - Jump to end (auto-scroll)
- `Space` - Pause/resume auto-scroll

**View Options:**
- `t` - Toggle task list
- `g` - Show task graph
- `a` - Show agent view
- `v` - Show variables
- `l` - Filter log level
- `f` - Filter by keyword
- `w` - Toggle line wrap

**Status Indicators:**
- **Running** (green) - Currently executing
- **Paused** (yellow) - Execution paused
- **Completed** (green) - All tasks finished
- **Failed** (red) - Execution failed
- **Stopped** (gray) - Manually stopped

**Tips:**
- Background with `Esc` to browse while running
- Status bar shows running workflow count
- Export logs with `e`
- View execution history with `h`

## State View

### State Browser Context

Browse saved workflow states. States are automatically saved during execution.

**Available Actions:**
- `Enter` - View state details
- `r` - Resume execution from state
- `e` - Export state to JSON
- `d` - Delete state (with confirmation)
- `c` - Compare states
- `f` - Filter by status/date/workflow
- `s` - Sort (date, status, workflow)
- `Space` - Quick preview

**State Types:**
- 📝 Running - Currently executing
- ⏸ Paused - Execution paused
- ✓ Completed - Successfully finished
- ✗ Failed - Execution failed

**Resume Capability:**

You can resume from any saved state:
1. Variables are restored
2. Completed tasks are skipped
3. Execution continues from last task
4. Same agents and configuration used

**Tips:**
- States are saved automatically at checkpoints
- Failed workflows can be debugged and resumed
- Export states for backup or analysis
- Clean up old states periodically

### State Details Context

View detailed information about a saved state.

**Information Shown:**
- Workflow name and version
- Execution status and timestamps
- Task progress (completed/total)
- Variable values (inputs/outputs)
- Agent assignments
- Error messages (if failed)

**Available Actions:**
- `r` - Resume execution
- `e` - Export to JSON
- `v` - View variables
- `t` - View task details
- `l` - View logs
- `Esc` - Back to browser

**Tips:**
- Check error messages for debugging
- Review variables to understand state
- Compare multiple states to track progress

## Help View

### Help Content Context

Browse documentation and help content.

**Navigation:**
- `↑↓` - Scroll content
- `PageUp/Down` - Fast scroll
- `Home/End` - Jump to top/bottom
- `Tab` - Next section
- `1-5` - Jump to specific topic

**Topics:**
1. User Guide - Complete usage guide
2. Keyboard Shortcuts - All shortcuts
3. Architecture - Technical documentation
4. Troubleshooting - Common issues
5. About - Version and info

**Search:**
- `/` - Start search
- `n` - Next result
- `N` - Previous result
- `Esc` - Clear search

**Tips:**
- Press `F1` anywhere for context help
- Use search to find specific topics
- Help content is embedded (works offline)

## Common Tasks

### "How do I create a workflow?"

1. Go to Workflows view (Tab)
2. Press `n` for new workflow
3. Choose creation method:
   - Template (fastest)
   - AI generation (easiest)
   - Blank (full control)
4. Edit in the editor
5. Save with `Ctrl+S`

### "How do I execute a workflow?"

1. Select workflow in Workflows view
2. Press `x` to execute
3. Enter inputs if prompted
4. Monitor in Execution view
5. Background with `Esc` if needed

### "How do I resume a failed workflow?"

1. Go to State view (Tab)
2. Find the failed state
3. Press `Enter` to view details
4. Check error message
5. Fix workflow if needed
6. Press `r` to resume

### "How do I search for a workflow?"

1. In Workflows view, press `/`
2. Type search term
3. Press `Enter`
4. Press `n` for next match
5. Press `Esc` to clear

### "How do I view execution logs?"

1. Go to Execution view (Tab)
2. Logs are shown in real-time
3. Scroll with arrow keys
4. Filter with `f` or `l`
5. Export with `e`

### "How do I change the theme?"

1. Launch with `--theme <name>`
2. Available: light, dark, solarized, monokai
3. Or edit config: `~/.claude-sdk/tui-config.yaml`
4. Restart TUI to apply

## Error Messages

### "Workflow validation failed"
Your workflow has errors. Press `F8` in editor to jump to errors. Common issues:
- Missing required fields
- Invalid agent references
- Circular dependencies

### "Execution failed"
Check the error message in logs. Common causes:
- Invalid inputs
- Missing files
- Agent errors
- Permission denied

### "Cannot load workflow"
File may be corrupted or invalid YAML. Try:
- Check file syntax
- Validate with CLI
- Restore from backup

### "State not found"
State may have been deleted or corrupted. States are in `~/.claude-sdk/states/`.

## Tips and Tricks

**Keyboard Shortcuts:**
- Learn the essentials first (↑↓, Enter, Esc, Tab, q)
- Press `F1` to see context-specific shortcuts
- Print the shortcuts reference

**Workflow Organization:**
- Use subdirectories for projects
- Name files descriptively
- Use version numbers in workflows
- Keep templates in separate folder

**Performance:**
- Background long-running workflows
- Clean up old states periodically
- Use filters for large workflow directories
- Limit log verbosity in production

**Development Workflow:**
1. Create from template or AI
2. Edit and test iteratively
3. Monitor execution closely
4. Debug from saved states
5. Refine and save final version

**Getting Unstuck:**
- Press `F1` for context help
- Check error messages carefully
- Review the troubleshooting guide
- Validate workflow with CLI
- Start with simple template

## Accessibility

**Keyboard-Only Navigation:**
All features accessible via keyboard. No mouse required.

**Screen Reader Support:**
Status messages announced for screen readers.

**High Contrast:**
Use `--theme light` or `--theme dark` for better contrast.

**Reduced Motion:**
Animations can be disabled in config.

## Further Reading

- Full User Guide: Press `1` in Help view
- Keyboard Shortcuts: Press `2` in Help view
- Architecture: Press `3` in Help view
- Troubleshooting: Press `4` in Help view

---

*Press `Esc` to close this help and return to your work.*
