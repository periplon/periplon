# DSL TUI Keyboard Shortcuts Reference

Complete keyboard shortcuts reference for the DSL TUI application.

## Global Shortcuts

These shortcuts work in any view:

| Key | Action |
|-----|--------|
| `F1` | Show context-sensitive help |
| `Tab` | Switch to next view |
| `Shift+Tab` | Switch to previous view |
| `q` | Quit application (with confirmation if workflows running) |
| `Ctrl+C` | Force quit (emergency exit) |
| `Ctrl+L` | Refresh screen |
| `Ctrl+Z` | Suspend application (return to shell) |
| `?` | Show keyboard shortcuts (same as F1) |

## Navigation

### List Navigation

| Key | Action |
|-----|--------|
| `↑` / `k` | Move up |
| `↓` / `j` | Move down |
| `←` / `h` | Move left / collapse |
| `→` / `l` | Move right / expand |
| `Home` / `g` | Jump to first item |
| `End` / `G` | Jump to last item |
| `PageUp` | Scroll up one page |
| `PageDown` | Scroll down one page |
| `Ctrl+U` | Scroll up half page |
| `Ctrl+D` | Scroll down half page |
| `Enter` | Select / activate item |
| `Esc` | Go back / cancel |
| `Backspace` | Go to parent directory (in file browser) |

### Search

| Key | Action |
|-----|--------|
| `/` | Start search |
| `n` | Next search result |
| `N` | Previous search result |
| `Esc` | Clear search |

## Workflows View

### File Browser

| Key | Action |
|-----|--------|
| `↑` / `↓` | Navigate files and folders |
| `Enter` | Open workflow or expand folder |
| `Backspace` | Go to parent directory |
| `Space` | Preview workflow (quick view) |
| `n` | Create new workflow |
| `e` | Edit selected workflow |
| `v` | View workflow details (read-only) |
| `x` | Execute selected workflow |
| `d` | Delete workflow (with confirmation) |
| `r` | Rename workflow |
| `c` | Copy workflow |
| `m` | Move workflow |
| `f` | Filter/search workflows |
| `s` | Sort workflows (name, date, size) |
| `Ctrl+R` | Refresh file list |
| `.` | Toggle hidden files |
| `*` | Select/deselect all |

### Workflow Creation

| Key | Action |
|-----|--------|
| `n` | New workflow menu |
| `t` | Create from template |
| `g` | Generate with AI |
| `b` | Create blank workflow |

## Workflow Editor

### Editing

| Key | Action |
|-----|--------|
| `Ctrl+S` | Save changes |
| `Ctrl+W` | Save and close |
| `Esc` | Cancel and close (with confirmation if modified) |
| `Ctrl+Q` | Force close without saving |
| `Ctrl+Z` | Undo |
| `Ctrl+Y` / `Ctrl+Shift+Z` | Redo |
| `Ctrl+X` | Cut line |
| `Ctrl+C` | Copy line |
| `Ctrl+V` | Paste |
| `Ctrl+D` | Duplicate line |
| `Ctrl+K` | Delete line |
| `Tab` | Indent |
| `Shift+Tab` | Unindent |
| `Ctrl+]` | Increase indentation |
| `Ctrl+[` | Decrease indentation |
| `Ctrl+/` | Toggle line comment |
| `Ctrl+A` | Select all |

### Navigation

| Key | Action |
|-----|--------|
| `Ctrl+Home` | Go to start of file |
| `Ctrl+End` | Go to end of file |
| `Ctrl+G` | Go to line number |
| `Ctrl+↑` | Scroll up |
| `Ctrl+↓` | Scroll down |
| `Home` | Go to start of line |
| `End` | Go to end of line |
| `Ctrl+←` | Move word left |
| `Ctrl+→` | Move word right |

### Search and Replace

| Key | Action |
|-----|--------|
| `Ctrl+F` | Find |
| `F3` / `Ctrl+G` | Find next |
| `Shift+F3` / `Ctrl+Shift+G` | Find previous |
| `Ctrl+H` | Find and replace |
| `Ctrl+Shift+H` | Replace all |
| `Esc` | Close search |

### Auto-completion

| Key | Action |
|-----|--------|
| `Ctrl+Space` | Trigger auto-completion |
| `↑` / `↓` | Navigate suggestions |
| `Enter` | Accept suggestion |
| `Esc` | Close suggestions |
| `Tab` | Accept and move to next field |

### Validation

| Key | Action |
|-----|--------|
| `Ctrl+Shift+V` | Validate workflow |
| `F8` | Go to next error/warning |
| `Shift+F8` | Go to previous error/warning |
| `Ctrl+Shift+E` | Show error details |

## Execution View

### Execution Control

| Key | Action |
|-----|--------|
| `x` | Start execution (if not running) |
| `p` | Pause execution |
| `r` | Resume paused execution |
| `s` | Stop execution (with confirmation) |
| `k` | Kill execution (force stop) |
| `Esc` | Move to background and return to browser |
| `Ctrl+X` | Execute workflow from file browser |

### Monitoring

| Key | Action |
|-----|--------|
| `↑` / `↓` | Scroll logs |
| `PageUp` / `PageDown` | Fast scroll logs |
| `Home` | Jump to first log entry |
| `End` | Jump to last log entry (auto-scroll) |
| `Space` | Pause/resume auto-scroll |
| `l` | Toggle log level filter |
| `f` | Filter logs by keyword |
| `c` | Clear log display (not history) |
| `e` | Export logs to file |
| `w` | Toggle log wrap |

### Views

| Key | Action |
|-----|--------|
| `t` | Toggle task list |
| `g` | Show task graph view |
| `a` | Show agent view |
| `v` | Show variables view |
| `d` | Show detailed view |
| `m` | Toggle minimal view |

### History

| Key | Action |
|-----|--------|
| `h` | Show execution history |
| `Enter` | View execution details |
| `d` | Delete history entry |
| `e` | Export execution report |

## State View

### State Browser

| Key | Action |
|-----|--------|
| `↑` / `↓` | Navigate states |
| `Enter` | View state details |
| `r` | Resume execution from state |
| `d` | Delete state (with confirmation) |
| `e` | Export state to JSON |
| `c` | Compare states |
| `f` | Filter states (by status, date, workflow) |
| `s` | Sort states (date, status, workflow) |
| `Space` | Quick preview state |

### State Details

| Key | Action |
|-----|--------|
| `r` | Resume execution |
| `e` | Export state |
| `v` | View variables |
| `t` | View task progress |
| `l` | View logs |
| `Esc` | Return to state browser |

## Help View

### Navigation

| Key | Action |
|-----|--------|
| `↑` / `↓` | Scroll content |
| `PageUp` / `PageDown` | Fast scroll |
| `Home` | Jump to top |
| `End` | Jump to bottom |
| `Tab` | Next section |
| `Shift+Tab` | Previous section |
| `Esc` | Close help |

### Content

| Key | Action |
|-----|--------|
| `1` | User guide |
| `2` | Keyboard shortcuts |
| `3` | Architecture docs |
| `4` | Troubleshooting |
| `5` | About |
| `/` | Search help content |

## Modal Dialogs

### Confirmation Dialogs

| Key | Action |
|-----|--------|
| `y` / `Enter` | Confirm (Yes) |
| `n` / `Esc` | Cancel (No) |
| `Tab` | Switch between buttons |

### Input Dialogs

| Key | Action |
|-----|--------|
| `Enter` | Submit |
| `Esc` | Cancel |
| `Ctrl+A` | Select all |
| `Ctrl+U` | Clear input |
| `Tab` | Next field |
| `Shift+Tab` | Previous field |

### Selection Dialogs

| Key | Action |
|-----|--------|
| `↑` / `↓` | Navigate options |
| `Enter` | Select option |
| `Esc` | Cancel |
| `Space` | Toggle selection (multi-select) |
| `/` | Filter options |

## Advanced Shortcuts

### Window Management

| Key | Action |
|-----|--------|
| `Ctrl+T` | New tab (if supported) |
| `Ctrl+W` | Close tab |
| `Ctrl+Tab` | Next tab |
| `Ctrl+Shift+Tab` | Previous tab |
| `Alt+1-9` | Jump to tab N |

### Developer Tools

| Key | Action |
|-----|--------|
| `Ctrl+Shift+D` | Toggle debug mode |
| `Ctrl+Shift+I` | Show internal state |
| `Ctrl+Shift+L` | Open log file |
| `Ctrl+Shift+R` | Reload configuration |

## Mouse Support

If your terminal supports mouse input:

| Action | Result |
|--------|--------|
| Click | Select item |
| Double-click | Activate/open item |
| Right-click | Context menu |
| Scroll wheel | Scroll content |
| Drag | Select text (in editor) |

## Customizing Shortcuts

Shortcuts can be customized in `~/.claude-sdk/tui-config.yaml`:

```yaml
keybindings:
  quit: "q"
  help: "F1"
  save: "Ctrl+S"
  # ... add custom bindings
```

## Vi Mode

Enable vi-style keybindings in config:

```yaml
editor:
  vi_mode: true
```

This enables:
- `h/j/k/l` for navigation
- `i` for insert mode
- `Esc` for normal mode
- `dd` to delete line
- `yy` to copy line
- `:w` to save
- `:q` to quit

## Emacs Mode

Enable emacs-style keybindings:

```yaml
editor:
  emacs_mode: true
```

This enables:
- `Ctrl+A` / `Ctrl+E` for line start/end
- `Ctrl+K` to kill line
- `Ctrl+Y` to yank
- `Alt+F` / `Alt+B` for word movement

## Tips

1. **Learn the essentials first**: Focus on navigation (`↑↓←→`), selection (`Enter`), and back (`Esc`)
2. **Use F1 liberally**: Context-sensitive help shows relevant shortcuts
3. **Practice in safe mode**: Use `--readonly` flag while learning
4. **Print this reference**: Keep it handy while learning the TUI
5. **Customize to your workflow**: Modify keybindings to match your preferences

## Quick Reference Card

Print-friendly one-page reference:

```
┌─────────────────────────────────────────────────────────────────┐
│                    DSL TUI Quick Reference                      │
├─────────────────────────────────────────────────────────────────┤
│ Global:   F1=Help  Tab=Views  q=Quit  Esc=Back                 │
│ Navigate: ↑↓=Move  Enter=Select  /=Search  g/G=Top/Bottom      │
│ Files:    n=New  e=Edit  v=View  x=Execute  d=Delete           │
│ Editor:   Ctrl+S=Save  Ctrl+F=Find  Ctrl+Space=Complete        │
│ Execute:  p=Pause  r=Resume  s=Stop  Esc=Background            │
│ State:    r=Resume  e=Export  d=Delete                         │
└─────────────────────────────────────────────────────────────────┘
```

---

**Version**: 1.0.0
**Last Updated**: 2025-10-21
