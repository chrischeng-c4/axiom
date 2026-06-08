# lumen client codegen
#
# The committed `clients/openapi.json` is the contract reference. CI
# regenerates it and `diff`s against the committed copy — any drift
# fails the build. To refresh after an intentional schema change:
#
#     make spec
#     git add openapi.json
#
# Language clients are NOT checked in; regenerate them on demand:
#
#     make all       # spec + go + ts + python
#     make go        # just the Go client (clients/go/)
#     make ts        # just the TS client (clients/ts/)
#     make python    # just the Python client (clients/python/)
#     make clean     # remove generated language clients
#
# Requirements:
#   - cargo (rustup toolchain)
#   - node + npx          (for @openapitools/openapi-generator-cli)

# Allow running from anywhere — this Makefile lives in projects/lumen/clients/
# and shells out into the workspace root + the lumen crate.
CLIENTS_DIR := $(abspath $(dir $(lastword $(MAKEFILE_LIST))))
LUMEN_DIR := $(abspath $(CLIENTS_DIR)/..)
WORKSPACE_ROOT := $(abspath $(LUMEN_DIR)/../..)

SPEC := $(CLIENTS_DIR)/openapi.json
GO_DIR := $(CLIENTS_DIR)/go
TS_DIR := $(CLIENTS_DIR)/ts
PY_DIR := $(CLIENTS_DIR)/python

OPENAPI_GEN := npx -y @openapitools/openapi-generator-cli@2.13.4 generate

.PHONY: all spec go ts python clean

all: spec go ts python

# Regenerate the committed OpenAPI spec from the live lumen source.
spec:
	@echo ">> regenerating $(SPEC)"
	cd $(WORKSPACE_ROOT) && cargo run -q -p lumen --bin lumen-openapi-dump > $(SPEC)

go: spec
	@echo ">> generating Go client into $(GO_DIR)"
	rm -rf $(GO_DIR)
	$(OPENAPI_GEN) \
	  -i $(SPEC) \
	  -g go \
	  -o $(GO_DIR) \
	  --additional-properties=packageName=lumen,withGoMod=true,isGoSubmodule=false

ts: spec
	@echo ">> generating TypeScript client into $(TS_DIR)"
	rm -rf $(TS_DIR)
	$(OPENAPI_GEN) \
	  -i $(SPEC) \
	  -g typescript-fetch \
	  -o $(TS_DIR) \
	  --additional-properties=npmName=@lumen/client,supportsES6=true,typescriptThreePlus=true

python: spec
	@echo ">> generating Python client into $(PY_DIR)"
	rm -rf $(PY_DIR)
	$(OPENAPI_GEN) \
	  -i $(SPEC) \
	  -g python \
	  -o $(PY_DIR) \
	  --additional-properties=packageName=lumen_client,projectName=lumen-client

clean:
	rm -rf $(GO_DIR) $(TS_DIR) $(PY_DIR)
