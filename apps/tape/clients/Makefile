# HANDWRITE-BEGIN gap="missing-generator:logic:d9fe5762" tracker="pending-tracker" reason="make gen-ts/gen-py/gen-rust targets wrapping `cargo run -p tape --features self-update -- spec gen --lang <lang> --out clients/<lang>`, plus a refresh-openapi target regenerating clients/openapi.json, mirroring lumen's projects/lumen/clients/Makefile layout."
# tape client codegen (#1329)
#
# The committed `clients/openapi.json` is the contract reference for the
# checked-in offline snapshot. Language clients are generated on demand,
# in-binary, via `tape spec gen` (libs/openapi-codegen) — no external
# openapi-generator tool, unlike lumen's clients/ Makefile.
#
#     make refresh-openapi   # regenerate clients/openapi.json from source
#     make gen-ts            # TypeScript client -> clients/ts/
#     make gen-py             # Python client     -> clients/py/
#     make gen-rust           # Rust client        -> clients/rust/
#     make gen-all            # all three languages
#     make clean              # remove generated language clients
#
# Requirements: cargo (rustup toolchain) only — no node/npx.

CLIENTS_DIR := $(abspath $(dir $(lastword $(MAKEFILE_LIST))))
TAPE_DIR := $(abspath $(CLIENTS_DIR)/..)
WORKSPACE_ROOT := $(abspath $(TAPE_DIR)/../..)

.PHONY: gen-all gen-ts gen-py gen-rust refresh-openapi clean

gen-all: gen-ts gen-py gen-rust

gen-ts:
	@echo ">> generating TypeScript client into $(CLIENTS_DIR)/ts"
	cd $(WORKSPACE_ROOT) && cargo run -q -p tape --features self-update -- spec gen --lang ts --out $(CLIENTS_DIR)/ts

gen-py:
	@echo ">> generating Python client into $(CLIENTS_DIR)/py"
	cd $(WORKSPACE_ROOT) && cargo run -q -p tape --features self-update -- spec gen --lang py --out $(CLIENTS_DIR)/py

gen-rust:
	@echo ">> generating Rust client into $(CLIENTS_DIR)/rust"
	cd $(WORKSPACE_ROOT) && cargo run -q -p tape --features self-update -- spec gen --lang rust --out $(CLIENTS_DIR)/rust

# Regenerate the committed OpenAPI spec snapshot from the live tape source.
refresh-openapi:
	@echo ">> regenerating $(CLIENTS_DIR)/openapi.json"
	cd $(WORKSPACE_ROOT) && cargo run -q -p tape --bin tape -- spec --format openapi > $(CLIENTS_DIR)/openapi.json

clean:
	rm -rf $(CLIENTS_DIR)/ts $(CLIENTS_DIR)/py $(CLIENTS_DIR)/rust
# HANDWRITE-END
