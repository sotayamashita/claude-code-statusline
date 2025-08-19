# Removed Features

This document tracks features that were initially implemented but later removed from the codebase.

## Character Module (2025-08-19)

### Original Purpose
The character module was intended to display a prompt character (like `❯` or `$`) in the status line, similar to Starship's prompt.

### Why It Was Removed
Claude Code's status line is **output-only** and doesn't accept user input. Unlike traditional shells where the prompt character indicates where to type, Claude Code's status line is purely informational. Therefore:
- The prompt character serves no functional purpose
- It adds visual clutter without value
- Claude Code already provides its own prompt in the input area

### Configuration Impact
While the module implementation was removed, the `CharacterConfig` structure remains in `types/config.rs` for:
1. **Backward compatibility**: Users with existing config files won't encounter errors
2. **Future flexibility**: If Claude Code adds interactive features, we can easily re-implement
3. **Starship compatibility**: Maintaining similar config structure to Starship

### Original Implementation
The module implemented the `Module` trait with:
- `should_display()` returning `false` (never displayed)
- Success/error symbols configuration
- Format string support

### Files Affected
- Removed: `src/modules/character.rs`
- Kept: `CharacterConfig` in `src/types/config.rs` (for config compatibility)
- Updated: `src/modules/mod.rs` (removed export)
- Updated: `src/main.rs` (removed usage)