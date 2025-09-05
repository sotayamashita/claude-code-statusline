# Variables
HOOKS_DIR := hooks
GIT_HOOKS_DIR := .git/hooks

.PHONY: install-hooks
install-hooks:
	@echo "📎 Installing git hooks..."
	@mkdir -p $(GIT_HOOKS_DIR)
	@cp $(HOOKS_DIR)/pre-commit.sh $(GIT_HOOKS_DIR)/pre-commit
	@chmod +x $(GIT_HOOKS_DIR)/pre-commit
	@echo "✅ pre-commit installed"

.PHONY: bench
bench:
	cargo bench -p beacon-core

.PHONY: bench-check
bench-check: bench
	@python3 scripts/bench_check.py --name engine_render_default --threshold-ms 50
