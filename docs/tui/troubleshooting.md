# DSL TUI Troubleshooting Guide

Common issues and solutions for the DSL TUI application.

## Table of Contents

1. [Installation Issues](#installation-issues)
2. [Startup Problems](#startup-problems)
3. [Display Issues](#display-issues)
4. [Workflow Issues](#workflow-issues)
5. [Execution Issues](#execution-issues)
6. [Performance Issues](#performance-issues)
7. [Configuration Issues](#configuration-issues)
8. [Known Issues](#known-issues)

## Installation Issues

### Build Fails with "feature not found"

**Problem:** `cargo build --features tui` fails with feature not found error.

**Solution:**
```bash
# Ensure you're using the correct feature flag
cargo build --bin periplon-tui --features tui

# Check Cargo.toml has tui feature enabled
grep -A5 "\[features\]" Cargo.toml
```

### Missing Dependencies

**Problem:** Build fails with missing crate errors.

**Solution:**
```bash
# Update dependencies
cargo update

# Clean and rebuild
cargo clean
cargo build --features tui

# Check Rust version (need 1.70+)
rustc --version
```

### Linking Errors on macOS

**Problem:** Linking errors during build on macOS.

**Solution:**
```bash
# Install Xcode command line tools
xcode-select --install

# Update Rust toolchain
rustup update
```

### Linking Errors on Linux

**Problem:** Missing library errors on Linux.

**Solution:**
```bash
# Ubuntu/Debian
sudo apt-get install build-essential pkg-config

# Fedora/RHEL
sudo dnf install gcc pkg-config

# Arch
sudo pacman -S base-devel
```

## Startup Problems

### TUI Won't Launch

**Problem:** Running `periplon-tui` shows no output or immediate exit.

**Diagnosis:**
```bash
# Run with debug output
RUST_LOG=debug ./target/release/periplon-tui

# Check if binary exists and is executable
ls -la ./target/release/periplon-tui

# Try with explicit config
./target/release/periplon-tui --debug --workflow-dir ./test-workflows
```

**Common Causes:**
1. **Permission issues**: Check workflow directory permissions
2. **Invalid config**: Remove `~/.claude-sdk/tui-config.yaml` and retry
3. **Terminal compatibility**: Try different terminal emulator

### "CLI not found" Error

**Problem:** Error message about missing CLI executable.

**Solution:**
```bash
# Install the CLI separately (if required)
# Check PATH contains CLI location
which claude-cli

# Or set explicit path
export CLAUDE_CLI_PATH=/path/to/claude-cli

# Skip CLI check if testing
export PERIPLON_SKIP_VERSION_CHECK=1
```

### Terminal Size Error

**Problem:** Error about terminal size too small.

**Solution:**
```bash
# Check current terminal size
tput cols
tput lines

# Minimum size is 80x24
# Resize terminal window or use:
resize -s 40 120  # 40 rows, 120 columns
```

## Display Issues

### Corrupted Display / Garbled Output

**Problem:** Text appears garbled or UI is corrupted.

**Solution:**
```bash
# Reset terminal
reset

# Force quit TUI (Ctrl+C)
# Then relaunch

# If persistent, try different terminal:
# - iTerm2 (macOS)
# - Windows Terminal (Windows)
# - GNOME Terminal (Linux)
# - Alacritty (cross-platform)
```

### Colors Not Showing

**Problem:** No colors or wrong colors displayed.

**Solution:**
```bash
# Check terminal color support
echo $TERM

# Should be xterm-256color or similar
# Set if needed:
export TERM=xterm-256color

# Try different theme
./target/release/periplon-tui --theme light

# Disable colors if needed
./target/release/periplon-tui --no-color
```

### Unicode Characters Not Displaying

**Problem:** Box drawing characters appear as question marks.

**Solution:**
```bash
# Check locale
locale

# Should include UTF-8
# Set if needed:
export LANG=en_US.UTF-8
export LC_ALL=en_US.UTF-8

# Use ASCII mode if UTF-8 not available
./target/release/periplon-tui --ascii-only
```

### Screen Doesn't Refresh

**Problem:** Display doesn't update or appears frozen.

**Solution:**
1. Press `Ctrl+L` to force refresh
2. Check if background process is stuck
3. Kill and restart TUI
4. Check debug log: `tail -f ~/.claude-sdk/tui-debug.log`

### Flickering Display

**Problem:** Screen flickers during rendering.

**Solution:**
```bash
# Enable double buffering (usually default)
# Try different terminal emulator
# Reduce log output with filtering

# In TUI, press:
# 'l' to filter logs
# 'm' for minimal view
```

## Workflow Issues

### Cannot Load Workflow

**Problem:** Error loading workflow file.

**Diagnosis:**
```bash
# Validate YAML syntax
yamllint workflow.yaml

# Or use Python
python3 -c "import yaml; yaml.safe_load(open('workflow.yaml'))"

# Check file permissions
ls -la workflow.yaml

# Validate with CLI
./target/release/periplon-executor validate workflow.yaml
```

**Common Causes:**
1. **Invalid YAML syntax**: Check indentation, quotes, special characters
2. **File not found**: Check path is correct
3. **Permission denied**: Check file permissions
4. **Corrupted file**: Restore from backup

### Validation Errors

**Problem:** Workflow fails validation.

**Common Errors:**

#### "Agent not found"
```yaml
# ✗ Bad: agent doesn't exist
tasks:
  my_task:
    agent: "non_existent_agent"

# ✓ Good: agent defined
agents:
  researcher:
    description: "Research agent"

tasks:
  my_task:
    agent: "researcher"
```

#### "Circular dependency"
```yaml
# ✗ Bad: circular dependency
tasks:
  task_a:
    depends_on: [task_b]
  task_b:
    depends_on: [task_a]

# ✓ Good: linear dependency
tasks:
  task_a:
    depends_on: []
  task_b:
    depends_on: [task_a]
```

#### "Invalid tool name"
```yaml
# ✗ Bad: typo in tool name
agents:
  my_agent:
    tools: [WebSarch]  # Typo!

# ✓ Good: correct tool name
agents:
  my_agent:
    tools: [WebSearch, WebFetch, Read, Write]
```

#### "Undefined variable"
```yaml
# ✗ Bad: variable not defined
tasks:
  analyze:
    description: "Analyze ${undefined_var}"

# ✓ Good: variable defined
inputs:
  topic:
    type: string
    required: true

tasks:
  analyze:
    description: "Analyze ${workflow.topic}"
```

### Cannot Save Workflow

**Problem:** Saving workflow fails.

**Solution:**
```bash
# Check directory permissions
ls -ld ~/.claude-sdk/workflows

# Create if missing
mkdir -p ~/.claude-sdk/workflows

# Check disk space
df -h

# Try saving to different location
./target/release/periplon-tui --workflow-dir ./test-workflows
```

### Workflow Not Appearing in Browser

**Problem:** Workflow file exists but doesn't show in browser.

**Solution:**
1. Press `Ctrl+R` to refresh file list
2. Check file extension is `.yaml` or `.yml`
3. Check file is in workflow directory
4. Toggle hidden files with `.` key
5. Check filter is not active (press `Esc` to clear)

## Execution Issues

### Execution Won't Start

**Problem:** Pressing `x` doesn't start execution.

**Diagnosis:**
```bash
# Check validation passes
# Press 'v' to view workflow details
# Look for error messages

# Try executing from CLI
./target/release/periplon-executor run workflow.yaml

# Check debug log
tail -f ~/.claude-sdk/tui-debug.log
```

**Common Causes:**
1. **Validation errors**: Fix errors shown in editor
2. **Missing inputs**: Provide required input values
3. **Agent errors**: Check agent configuration
4. **Resource limits**: Check system resources

### Execution Fails Immediately

**Problem:** Execution starts but fails right away.

**Solution:**
```bash
# Check error message in logs
# Common issues:

# 1. Missing files
# Check all file paths in workflow

# 2. Invalid permissions
# Review permission mode settings

# 3. Agent communication error
# Check CLI is accessible

# 4. Invalid inputs
# Verify input values match types
```

### Execution Hangs

**Problem:** Execution starts but makes no progress.

**Solution:**
1. Check task is actually running (not waiting for dependency)
2. View task graph with `g` to see dependencies
3. Check agent is processing (view logs)
4. Look for deadlocks in task dependencies
5. Kill and restart if needed (`k` to kill)

### Cannot Pause Execution

**Problem:** Pressing `p` doesn't pause execution.

**Possible Reasons:**
1. **Not pausable**: Some tasks cannot be paused mid-execution
2. **Already paused**: Check status indicator
3. **Completing**: Task finishing, wait for completion
4. **Error state**: Execution may have failed

**Solution:**
- Wait for current task to finish
- Use `s` to stop instead of pause
- Check execution status in logs

### State Not Saved

**Problem:** No state saved after execution.

**Solution:**
```bash
# Check state directory exists
mkdir -p ~/.claude-sdk/states

# Check permissions
ls -ld ~/.claude-sdk/states

# Check state persistence enabled
# In config: state_persistence: true

# Verify execution reached checkpoint
# States saved at task completion
```

## Performance Issues

### Slow Rendering

**Problem:** UI feels sluggish or slow to respond.

**Solution:**
```bash
# Use lighter theme
./target/release/periplon-tui --theme light

# Reduce log verbosity
# In TUI: press 'l' to filter logs

# Close other resource-intensive apps

# Use minimal view
# In TUI: press 'm'

# Limit log retention in config:
# max_log_lines: 500
```

### High CPU Usage

**Problem:** TUI consumes excessive CPU.

**Solution:**
```bash
# Check what's causing load
# Run with profiling:
cargo flamegraph --bin periplon-tui --features tui

# Reduce refresh rate in config:
# refresh_rate_ms: 250

# Disable auto-scroll if not needed
# In Execution view: press 'Space'

# Background resource-heavy executions
# Press 'Esc' in Execution view
```

### High Memory Usage

**Problem:** Memory usage grows over time.

**Solution:**
```bash
# Limit log retention
# In config:
# max_log_lines: 1000

# Clear old states periodically
# In State view: 'd' to delete old states

# Restart TUI for long sessions

# Close unneeded workflows
# Only load what you're working on
```

### Slow File Loading

**Problem:** Workflow browser slow to load files.

**Solution:**
```bash
# Reduce workflow directory size
# Move old workflows to archive

# Use subdirectories for organization

# Disable file watching if enabled
# In config: watch_files: false

# Increase file cache in config:
# file_cache_size: 100
```

## Configuration Issues

### Config Not Loading

**Problem:** Configuration file ignored.

**Solution:**
```bash
# Check config file location
ls -la ~/.claude-sdk/tui-config.yaml

# Validate YAML syntax
yamllint ~/.claude-sdk/tui-config.yaml

# Check for syntax errors
# Remove and recreate if corrupted

# Use default config
mv ~/.claude-sdk/tui-config.yaml ~/.claude-sdk/tui-config.yaml.bak
./target/release/periplon-tui
```

### Invalid Configuration

**Problem:** Error about invalid configuration.

**Solution:**
```yaml
# Check required fields present
# Example valid config:

theme: "dark"
workflow_dir: "~/.claude-sdk/workflows"
state_dir: "~/.claude-sdk/states"
auto_save: true
refresh_rate_ms: 100
max_log_lines: 1000

keybindings:
  quit: "q"
  help: "F1"
  save: "Ctrl+S"

editor:
  tab_size: 2
  auto_indent: true
  syntax_highlighting: true
```

### Keybindings Not Working

**Problem:** Custom keybindings don't work.

**Solution:**
```yaml
# Check keybinding format in config
keybindings:
  # Use exact key names:
  quit: "q"           # Single key
  help: "F1"          # Function key
  save: "Ctrl+S"      # Modifier + key
  find: "Ctrl+F"
  # Not: save: "^S" or save: "CTRL-S"

# Restart TUI after config change
```

## Known Issues

### Windows Terminal Issues

**Known Issues:**
- Occasional rendering artifacts
- Some Unicode characters may not display
- Mouse support limited in older versions

**Workarounds:**
- Use Windows Terminal (modern version)
- Try `--ascii-only` flag
- Update to latest Windows Terminal version

### macOS Terminal Issues

**Known Issues:**
- Alt key combinations may not work in Terminal.app
- Some themes may look different

**Workarounds:**
- Use iTerm2 instead of Terminal.app
- Remap Alt key in terminal preferences
- Try different theme

### SSH/Remote Sessions

**Known Issues:**
- Color support may be limited
- Terminal size detection issues
- Clipboard operations may not work

**Workarounds:**
```bash
# Set TERM explicitly
export TERM=xterm-256color

# Use simpler theme
./target/release/periplon-tui --theme light

# Forward terminal capabilities
# In SSH: use -t flag for TTY allocation
ssh -t user@host
```

### tmux/screen Issues

**Known Issues:**
- Colors may not display correctly
- Some key combinations captured by multiplexer

**Workarounds:**
```bash
# In tmux, set 256 colors:
# .tmux.conf:
set -g default-terminal "screen-256color"

# Detach before quitting TUI
# Or remap quit key to avoid conflict

# For screen, add to .screenrc:
term screen-256color
```

## Getting Help

### Collecting Debug Information

When reporting issues, collect:

```bash
# 1. Version info
./target/release/periplon-tui --version

# 2. System info
uname -a
echo $TERM

# 3. Debug log
./target/release/periplon-tui --debug
# Then reproduce issue
# Attach: ~/.claude-sdk/tui-debug.log

# 4. Configuration
cat ~/.claude-sdk/tui-config.yaml

# 5. Error messages
# Screenshot or copy exact error text
```

### Debug Mode

Enable comprehensive debugging:

```bash
# Run with all debug options
RUST_LOG=debug ./target/release/periplon-tui --debug

# Check log file
tail -f ~/.claude-sdk/tui-debug.log

# In another terminal, monitor:
watch -n 1 'ps aux | grep periplon-tui'
```

### Common Error Messages

#### "Terminal too small"
- Minimum size: 80 columns × 24 rows
- Resize terminal window

#### "Permission denied"
- Check file/directory permissions
- Use `chmod` to fix

#### "Invalid UTF-8"
- Check file encoding
- Convert to UTF-8: `iconv -f ISO-8859-1 -t UTF-8 file.yaml > file_utf8.yaml`

#### "Broken pipe"
- CLI communication error
- Check CLI is running
- Restart TUI

#### "Resource temporarily unavailable"
- System resource limits hit
- Close other applications
- Check with `ulimit -a`

### Reporting Bugs

When reporting bugs, include:

1. **Steps to reproduce**: Exact steps that cause the issue
2. **Expected behavior**: What should happen
3. **Actual behavior**: What actually happens
4. **Environment**: OS, terminal, Rust version
5. **Debug log**: Attach debug log file
6. **Screenshots**: If display issue
7. **Workflow**: Minimal workflow that reproduces issue (if applicable)

Create issue at: `<project-repository>/issues`

### Community Support

- **Documentation**: Check docs/tui/ for guides
- **Examples**: See examples/ directory
- **Discussions**: Project discussion board
- **Stack Overflow**: Tag with `periplon-tui`

## Recovery Procedures

### Corrupted State

```bash
# Remove corrupted state files
rm -rf ~/.claude-sdk/states/*

# Restart TUI
./target/release/periplon-tui
```

### Reset Configuration

```bash
# Backup current config
mv ~/.claude-sdk/tui-config.yaml ~/.claude-sdk/tui-config.yaml.bak

# TUI will create default config on next run
./target/release/periplon-tui
```

### Clean Reinstall

```bash
# Remove all TUI data
rm -rf ~/.claude-sdk/

# Rebuild
cargo clean
cargo build --release --bin periplon-tui --features tui

# Restart
./target/release/periplon-tui
```

## Preventive Maintenance

### Regular Cleanup

```bash
# Clean old states (keep last 10)
cd ~/.claude-sdk/states
ls -t | tail -n +11 | xargs rm -f

# Clean debug logs older than 7 days
find ~/.claude-sdk -name "*.log" -mtime +7 -delete

# Optimize workflow directory
# Move old workflows to archive
```

### Best Practices

1. **Save frequently**: Use `Ctrl+S` often when editing
2. **Validate before executing**: Check for errors first
3. **Monitor resources**: Watch memory/CPU during long runs
4. **Update regularly**: Keep dependencies updated
5. **Backup workflows**: Keep version control or backups
6. **Clean up states**: Don't accumulate too many old states
7. **Use stable versions**: Avoid development builds for production

---

**Still having issues?**

Check the [User Guide](user-guide.md) for detailed usage information, or visit the [Developer Guide](developer-guide.md) for technical details.

**Version**: 1.0.0
**Last Updated**: 2025-10-21
