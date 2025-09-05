# Variables
HOOKS_DIR := hooks
CLAUDE_CODE_DIR := .claude
GIT_HOOKS_DIR := .git/hooks

.PHONY: install-hooks
install-hooks:
	@echo "📎 Installing git hooks..."
	@mkdir -p $(GIT_HOOKS_DIR)
	@cp $(HOOKS_DIR)/pre-commit.sh $(GIT_HOOKS_DIR)/pre-commit
	@chmod +x $(GIT_HOOKS_DIR)/pre-commit
	@echo "✅ pre-commit installed"

.PHONY: debug
debug:
    @echo "Building debug version..."
    @cargo build --workspace --release
    @mv target/release/beacon $(CLAUDE_CODE_DIR)/beacon
    @chmod +x $(CLAUDE_CODE_DIR)/beacon
    @echo "✅ Debug version built and installed"

.PHONY: bench
bench:
	cargo bench -p beacon-core

.PHONY: bench-check
bench-check: bench
	@python3 scripts/bench_check.py --name engine_render_default --threshold-ms 50
