RPC ?= $(ETH_RPC_HTTP)

# ── Build ──────────────────────────────────────────────────────────────────────
build:
	forge build

# ── Tests ─────────────────────────────────────────────────────────────────────

# Pure encoding tests — no fork, no huffc required
test-encoding:
	forge test --match-contract EncodingTest -vv

# All SandwichBot fork tests — requires huffc (huff-rs) + ETH_RPC_HTTP
test-sandwich:
	ETH_RPC_URL="$(RPC)" forge test --match-contract SandwichBotTest --no-match-test testFuzz -vv

# Fuzz tests (slow)
test-fuzz:
	ETH_RPC_URL="$(RPC)" forge test --match-contract SandwichBotTest --match-test testFuzz -vv

# All tests
test:
	ETH_RPC_URL="$(RPC)" forge test -vv

# ── Deploy ────────────────────────────────────────────────────────────────────
# Usage: make deploy RPC=<rpc_url> PRIVATE_KEY=<key>
deploy:
	@echo "Deploying SandwichBot.huff to mainnet..."
	@echo "IMPORTANT: Set OWNER constant in SandwichBot.huff to your address first!"
	huffc src/SandwichBot.huff --bytecode --paste

.PHONY: build test test-encoding test-sandwich test-fuzz deploy
