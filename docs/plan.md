# Beacon - Development Plan

> See @docs/todo.md for detailed implementation tasks.

## Development Phases

### Phase 1: MVP (Week 1) - Minimum Viable Product

> 📝 **Detailed Tasks**: @docs/todo.md - 11 sections, ~4.5 hours of work
- [ ] Basic CLI structure with `clap`
- [ ] Simple TOML config loading
- [ ] Directory module only
- [ ] Character module only
- [ ] Claude model module (showcase unique feature)
- [ ] Read complete JSON structure from stdin (all Claude Code fields)
- [ ] Single-line ANSI output to stdout
- [ ] Handle executable permission errors gracefully

**Rust Learning Focus**: Basic syntax, ownership, Result type
**Skip for now**: Git modules, parallel processing, custom errors, advanced testing

### Phase 2: Core Features (Week 2) - Add More Modules

> 📝 **Detailed Tasks**: To be added to todo.md after Phase 1 completion

- [ ] Git branch module (using `git2`)
- [ ] Claude session module
- [ ] Basic error handling improvements
- [ ] Config validation

**Rust Learning Focus**: External crates, trait basics, error handling with ?

### Phase 3: Polish (Week 3) - Improve Quality

> 📝 **Detailed Tasks**: To be added to todo.md after Phase 2 completion

- [ ] Git status module
- [ ] Basic caching
- [ ] Module timeout system
- [ ] Basic tests
- [ ] Documentation

**Rust Learning Focus**: Testing, documentation

### Phase 4: Advanced (Optional) - For Continued Learning

> 📝 **Detailed Tasks**: To be added to todo.md after Phase 3 completion
- [ ] Parallel execution with `rayon`
- [ ] Custom error types with `thiserror`
- [ ] Performance benchmarks with `criterion`
- [ ] More language modules
- [ ] Installation script

**Note**: Phase 4 is optional and for continued learning after MVP works

## Related Documents

- 📖 @docs/spec.md - Complete technical specification
- 📋 @docs/todo.md - Detailed implementation task list
- 🚀 @README.md - Project overview

## Progress Management

Detailed tasks for each phase will be added to @docs/todo.md at the beginning of each phase.
