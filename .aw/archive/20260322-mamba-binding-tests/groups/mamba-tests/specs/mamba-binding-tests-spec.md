---
id: mamba-binding-tests-spec
main_spec_ref: "cclab-mamba/testing/mamba-binding-tests-spec.md"
merge_strategy: new
filled_sections: [overview, requirements, scenarios, test-plan, changes]
create_complete: true
---

# Mamba Binding Tests Spec

## Overview

## Overview

Add integration test suites for 7 Mamba Python binding crates and the `cclab-mamba-registry` foundation crate. Each binding crate exposes `mb_*` FFI functions callable from the Mamba JIT via the native-call ABI (`extern "C" fn(*const MbValue, usize) -> MbValue`). Currently each crate has 5–9 inline unit tests in `methods.rs` but no `tests/` integration test directory.

This change creates `crates/cclab-{name}-mamba/tests/methods_test.rs` for each of the 7 binding crates, covering all 72 `mb_*` functions (happy path + error/boundary case per function). It also investigates and resolves the 3 reported ignored tests in `cclab-mamba-registry`.

**Scope**: binding crates ONLY. Runtime/lower/resolve/stdlib coverage is tracked separately in issue #1035.

| Crate | Module | mb_* count | Current tests |
|-------|--------|-----------|---------------|
| cclab-pg-mamba | `cclab.pg` | 19 | 6 inline |
| cclab-api-mamba | `cclab.api` | 17 | 6 inline |
| cclab-runtime-mamba | `cclab.runtime` | 4 | 5 inline |
| cclab-agent-mamba | `cclab.agent` | 13 | 9 inline |
| cclab-fetch-mamba | `cclab.fetch` | 8 | 5 inline |
| cclab-log-mamba | `cclab.log` | 5 | 5 inline |
| cclab-mcp-mamba | `cclab.mcp` | 6 | 5 inline |
| cclab-mamba-registry | (foundation) | — | 17 inline |
| **Total** | | **72** | **58 inline** |
## Requirements

## Requirements

| ID | Requirement | Priority |
|----|-------------|----------|
| R1 | Each of the 7 binding crates MUST have `crates/cclab-{name}-mamba/tests/methods_test.rs` | P0 |
| R2 | Every `mb_*` function MUST have ≥2 tests: (a) happy path with valid inputs, (b) error/boundary case | P0 |
| R3 | `cclab-mamba-registry`: verify status of 3 reported `#[ignore]` tests; un-ignore if linkage is resolved, document if blocked | P0 |
| R4 | Integration tests MUST NOT require live PostgreSQL or external network services | P0 |
| R5 | Tests MUST compile and pass with `cargo test -p cclab-{name}-mamba` | P0 |
| R6 | Tests live in `tests/` (integration-style); existing inline `#[cfg(test)]` blocks in `methods.rs` remain untouched | P1 |
| R7 | No CI configuration changes in this change | P1 |
| R8 | No coverage tooling changes in this change | P1 |
| R9 | Total new integration tests: ≥ 120 (≥2 per function × 72 functions) | P1 |
| R10 | `mb_pg_connect` and `mb_pg_execute` tests MUST mock/avoid live DB (offline behavior validated via error return) | P0 |

### Binding Crate Function Counts

| Crate | Functions | Min new tests |
|-------|-----------|---------------|
| cclab-pg-mamba | 19 | 38 |
| cclab-api-mamba | 17 | 34 |
| cclab-runtime-mamba | 4 | 8 |
| cclab-agent-mamba | 13 | 26 |
| cclab-fetch-mamba | 8 | 16 |
| cclab-log-mamba | 5 | 10 |
| cclab-mcp-mamba | 6 | 12 |
| **Total** | **72** | **≥144** |
## Scenarios

## Scenarios

All test helpers use the shared pattern:
```rust
fn make_str_val(s: &str) -> MbValue {
    MbValue::from_ptr(Box::into_raw(Box::new(s.to_string())) as usize)
}
```

### cclab-pg-mamba (19 functions)

| Scenario | Function | Inputs | Expected |
|----------|----------|--------|----------|
| qb_new_happy | mb_pg_query_builder_new | `"users"` str | PTR to MbQueryBuilder |
| qb_new_zero_args | mb_pg_query_builder_new | `nargs=0` | PTR with table=`"unknown"` |
| qb_build_no_clauses | mb_pg_query_builder_build | bare builder | `"SELECT * FROM users"` |
| qb_build_null_ptr | mb_pg_query_builder_build | null ptr | `MbValue::none()` |
| qb_limit_sets_sql | mb_pg_query_builder_limit + build | builder + `Int(10)` | SQL contains `"LIMIT 10"` |
| qb_limit_null_ptr | mb_pg_query_builder_limit | null ptr | `none()` |
| qb_where_happy | mb_pg_query_builder_where + build | `"price",">","100"` | SQL contains WHERE |
| qb_where_null_ptr | mb_pg_query_builder_where | null ptr | `none()` |
| qb_order_by_happy | mb_pg_query_builder_order_by + build | `"price","DESC"` | SQL has `ORDER BY price DESC` |
| qb_order_by_null | mb_pg_query_builder_order_by | null ptr | `none()` |
| qb_select_cols | mb_pg_query_builder_select + build | Vec of col ptrs | SQL has named cols |
| qb_select_null_ptr | mb_pg_query_builder_select | null builder | `none()` |
| decl_base_happy | mb_pg_declarative_base_new | `"User"` | PTR; `class_name=="User"`, `table_name=="user"` |
| decl_base_default | mb_pg_declarative_base_new | `nargs=0` | PTR; `class_name=="UnknownTable"` |
| table_name_set_happy | mb_pg_table_name_set | table + `"users"` | `table_name=="users"` |
| table_name_set_null | mb_pg_table_name_set | null ptr | `none()` |
| mapped_col_happy | mb_pg_mapped_column | type + name + pk=true + nullable=false | PTR; `primary_key==true` |
| mapped_col_defaults | mb_pg_mapped_column | no pk/nullable args | `primary_key==false, nullable==true` |
| relationship_happy | mb_pg_relationship | `"Post","posts","author"` | PTR; fields set |
| relationship_no_back | mb_pg_relationship | target + attr + none | `back_populates==None` |
| foreign_key_happy | mb_pg_foreign_key | `"users.id"` | PTR string starts with `"FK:"` |
| foreign_key_empty | mb_pg_foreign_key | empty str | PTR with `"FK:"` |
| index_happy | mb_pg_index | `"idx_price"` + cols | PTR string starts with `"INDEX:"` |
| type_string_happy | mb_pg_type_string | Int(255) | `type_name=="String", max_len==Some(255)` |
| type_string_no_len | mb_pg_type_string | nargs=0 | `type_name=="String", max_len==None` |
| type_text_happy | mb_pg_type_text | no args | `type_name=="Text"` |
| type_json_happy | mb_pg_type_json | no args | `type_name=="JSON"` |
| type_uuid_happy | mb_pg_type_uuid | no args | `type_name=="UUID"` |
| type_datetime_happy | mb_pg_type_datetime | no args | `type_name=="DateTime"` |
| connect_offline | mb_pg_connect | invalid URL str | PTR; `connected==false` |
| execute_null_pool | mb_pg_execute | null ptr pool | `none()` |
| execute_empty_sql | mb_pg_execute | valid pool + empty str | `none()` |

### cclab-api-mamba (17 functions)

| Scenario | Function | Inputs | Expected |
|----------|----------|--------|----------|
| router_new_happy | mb_api_router_new | `"/api"` + none | PTR; `prefix=="/api"` |
| router_new_with_tags | mb_api_router_new | prefix + tag list | PTR; `tags` populated |
| router_add_get_happy | mb_api_router_add_get | router + path + func | route_count==1 |
| router_add_get_null | mb_api_router_add_get | null router | `none()` |
| router_add_post_happy | mb_api_router_add_post | router + path + func | route_count incremented |
| router_add_put_happy | mb_api_router_add_put | router + path + func | route registered |
| router_add_delete_happy | mb_api_router_add_delete | router + path + func | route registered |
| router_add_patch_happy | mb_api_router_add_patch | router + path + func | route registered |
| routes_count_zero | mb_api_router_routes_count | empty router | Int(0) |
| routes_count_n | mb_api_router_routes_count | router with 3 routes | Int(3) |
| routes_count_null | mb_api_router_routes_count | null ptr | Int(0) |
| depends_new_happy | mb_api_depends_new | func ptr | PTR; `callable_ptr` set |
| http_exc_404 | mb_api_http_exception_new | Int(404) + `"Not Found"` | `status_code==404` |
| http_exc_default_status | mb_api_http_exception_new | nargs=0 | `status_code==500` |
| request_new_happy | mb_api_request_new | `"GET"` + `"/health"` | PTR; method+path set |
| request_method_get | mb_api_request_method | request | `"GET"` |
| request_method_null | mb_api_request_method | null ptr | `none()` |
| request_path_happy | mb_api_request_path | request | `"/health"` |
| request_path_null | mb_api_request_path | null ptr | `none()` |
| request_query_param_miss | mb_api_request_query_param | request + `"q"` (not set) | `none()` |
| response_new_happy | mb_api_response_new | Int(201) + body | `status_code==201` |
| response_new_no_status | mb_api_response_new | none + body | `status_code==200` |
| response_json_happy | mb_api_response_json | json str | `content_type=="application/json"` |
| response_json_empty | mb_api_response_json | nargs=0 | `content_type=="application/json"` (defaults to `{}`) |
| bg_tasks_new | mb_api_background_tasks_new | no args | PTR; `tasks.len==0` |
| bg_tasks_add_happy | mb_api_background_tasks_add | tasks + func | `tasks.len==1` |
| bg_tasks_add_null | mb_api_background_tasks_add | null ptr | `none()` |

### cclab-runtime-mamba (4 functions)

| Scenario | Function | Inputs | Expected |
|----------|----------|--------|----------|
| sleep_float_zero | mb_runtime_sleep | Float(0.0) | `none()` |
| sleep_float_positive | mb_runtime_sleep | Float(0.001) | `none()` |
| sleep_int_zero | mb_runtime_sleep | Int(0) | `none()` |
| sleep_negative_clamps | mb_runtime_sleep | Float(-1.0) | `none()` (no panic) |
| sleep_no_args | mb_runtime_sleep | nargs=0 | `none()` |
| spawn_happy | mb_runtime_spawn | func ptr | PTR; `task_id>0`, `done==true` |
| spawn_no_args | mb_runtime_spawn | nargs=0 | PTR (stub, no crash) |
| spawn_unique_ids | mb_runtime_spawn × 3 | func ptrs | all distinct task_ids |
| gather_stub_no_args | mb_runtime_gather | nargs=0 | `none()` |
| gather_stub_with_list | mb_runtime_gather | Vec ptr | `none()` |

### cclab-agent-mamba (13 functions)

| Scenario | Function | Inputs | Expected |
|----------|----------|--------|----------|
| builder_new_empty | mb_agent_builder_new | no args | PTR; all fields empty |
| claude_provider_happy | mb_agent_claude_provider | api_key | PTR; `name=="claude"` |
| claude_provider_empty_key | mb_agent_claude_provider | empty str | PTR; `api_key==""` |
| gemini_provider_happy | mb_agent_gemini_provider | api_key | PTR; `name=="gemini"` |
| openai_provider_happy | mb_agent_openai_provider | api_key | PTR; `name=="openai"` |
| builder_set_provider | mb_agent_builder_provider | builder + claude provider | `provider_name=="claude"` |
| builder_set_provider_null | mb_agent_builder_provider | null builder | `none()` |
| builder_system_prompt | mb_agent_builder_system_prompt | builder + prompt | `system_prompt` set |
| builder_system_prompt_null | mb_agent_builder_system_prompt | null builder | `none()` |
| builder_build_configured | mb_agent_builder_build | fully configured builder | PTR; all fields match |
| builder_build_null | mb_agent_builder_build | null builder | PTR; empty fields |
| agent_run_stub | mb_agent_run | gemini agent + prompt | PTR; response contains `"stub"` |
| agent_run_null_agent | mb_agent_run | null ptr | PTR; contains `"error"` |
| message_new_happy | mb_agent_message_new | `"user"` + content | PTR; role+content set |
| message_new_default_role | mb_agent_message_new | nargs=0 | PTR; `role=="user"` |
| message_role_happy | mb_agent_message_role | message | `"user"` |
| message_role_null | mb_agent_message_role | null ptr | empty string |
| message_content_happy | mb_agent_message_content | message | content string |
| message_content_null | mb_agent_message_content | null ptr | empty string |
| tool_registry_new | mb_agent_tool_registry_new | no args | PTR; `tools.len==0` |
| tool_registry_register_happy | mb_agent_tool_registry_register | registry + name + func | `tools.len==1` |
| tool_registry_register_null | mb_agent_tool_registry_register | null registry | `none()` |
| tool_registry_multiple | mb_agent_tool_registry_register × 3 | different names | `tools.len==3` |

### cclab-fetch-mamba (8 functions)

| Scenario | Function | Inputs | Expected |
|----------|----------|--------|----------|
| client_new_happy | mb_fetch_client_new | URL + Float(10.0) | PTR; `base_url` + `timeout_secs==10.0` |
| client_new_default_timeout | mb_fetch_client_new | URL only (nargs=1) | PTR; `timeout_secs==30.0` |
| client_new_int_timeout | mb_fetch_client_new | URL + Int(5) | PTR; `timeout_secs==5.0` |
| client_new_no_args | mb_fetch_client_new | nargs=0 | PTR; empty base_url |
| response_status_200 | mb_fetch_response_status | `MbHttpResponse::ok(200,…)` | Int(200) |
| response_status_404 | mb_fetch_response_status | `MbHttpResponse::ok(404,…)` | Int(404) |
| response_status_null | mb_fetch_response_status | null ptr | Int(0) |
| response_status_error | mb_fetch_response_status | `MbHttpResponse::error()` | Int(0) |
| response_text_happy | mb_fetch_response_text | response with body | PTR; body string |
| response_text_empty | mb_fetch_response_text | response with empty body | PTR; empty string |
| response_text_null | mb_fetch_response_text | null ptr | PTR; empty string |
| response_json_delegates | mb_fetch_response_json | response | same body as text |
| client_get_invalid_url | mb_fetch_client_get | client(`""`) + path | PTR; `status==0` (error) |
| client_post_null_client | mb_fetch_client_post | null ptr + path | PTR; status==0 |
| client_put_null_client | mb_fetch_client_put | null ptr + path | PTR; status==0 |
| client_delete_null_client | mb_fetch_client_delete | null ptr + path | PTR; status==0 |

### cclab-log-mamba (5 functions)

| Scenario | Function | Inputs | Expected |
|----------|----------|--------|----------|
| get_logger_named | mb_log_get_logger | `"myapp"` | PTR; `name=="myapp"` |
| get_logger_default | mb_log_get_logger | nargs=0 | PTR; `name=="root"` |
| get_logger_empty | mb_log_get_logger | empty str | PTR; `name==""` |
| log_info_returns_none | mb_log_info | logger + msg | `none()` |
| log_info_null_logger | mb_log_info | null ptr + msg | `none()` (no crash) |
| log_error_returns_none | mb_log_error | logger + msg | `none()` |
| log_error_null_logger | mb_log_error | null ptr + msg | `none()` |
| log_debug_returns_none | mb_log_debug | logger + msg | `none()` |
| log_debug_null_logger | mb_log_debug | null ptr + msg | `none()` |
| log_warning_returns_none | mb_log_warning | logger + msg | `none()` |
| log_warning_null_logger | mb_log_warning | null ptr + msg | `none()` |
| log_all_levels_sequence | all 4 level fns | same logger | all return `none()` |

### cclab-mcp-mamba (6 functions)

| Scenario | Function | Inputs | Expected |
|----------|----------|--------|----------|
| server_new_happy | mb_mcp_server_new | `"Conductor"` | PTR; `name=="Conductor"`, `tools.len==0` |
| server_new_default | mb_mcp_server_new | nargs=0 | PTR; `name=="mcp"` |
| register_tool_happy | mb_mcp_server_register_tool | server + name + doc + func | `tools.len==1` |
| register_tool_fields | mb_mcp_server_register_tool | server + specific values | `tools[0]` = (name, doc, func_ptr) |
| register_tool_null_server | mb_mcp_server_register_tool | null ptr | `none()` |
| tool_count_zero | mb_mcp_server_tool_count | empty server | Int(0) |
| tool_count_multiple | mb_mcp_server_tool_count | server with 3 tools | Int(3) |
| tool_count_null | mb_mcp_server_tool_count | null ptr | Int(0) |
| run_stdio_returns_none | mb_mcp_server_run_stdio | server | `none()` |
| run_stdio_null | mb_mcp_server_run_stdio | null ptr | `none()` |
| streamable_http_app_happy | mb_mcp_server_streamable_http_app | server | PTR; `server_name` matches |
| streamable_http_app_null | mb_mcp_server_streamable_http_app | null ptr | PTR; `server_name=="mcp"` |
| server_name_happy | mb_mcp_server_name | server | PTR; `"Conductor"` |
| server_name_null | mb_mcp_server_name | null ptr | PTR; empty string |

### cclab-mamba-registry (ignored tests)

| Scenario | Action | Expected |
|----------|--------|----------|
| ignored_tests_audit | `cargo test -p mamba-registry -- --ignored` | 0 ignored tests (already active) or document blocker |
| all_tests_pass | `cargo test -p mamba-registry` | all 17 tests pass |
| module_registration_smoke | `all_modules()` in integration test | slice accessible; count ≥ 0 |
## Diagrams

### Interaction
<!-- type: interaction lang: mermaid -->
<!-- TODO -->

### Logic
<!-- type: logic lang: mermaid -->
<!-- TODO -->

### Dependencies
<!-- type: dependency lang: mermaid -->
<!-- TODO -->

### State Machine
<!-- type: state-machine lang: mermaid -->
<!-- TODO -->

### Data Model
<!-- type: db-model lang: mermaid -->
<!-- TODO -->

## API Spec

### REST API
<!-- type: rest-api lang: yaml -->
<!-- TODO -->

### RPC API
<!-- type: rpc-api lang: json -->
<!-- TODO -->

### Async API
<!-- type: async-api lang: yaml -->
<!-- TODO -->

### CLI
<!-- type: cli lang: yaml -->
<!-- TODO -->

### Schema
<!-- type: schema lang: json -->
<!-- TODO -->

### Config
<!-- type: config lang: json -->
<!-- TODO -->

## Test Plan

## Test Plan

### Test File Organization

| File | Crate | Functions under test | Min tests |
|------|-------|---------------------|----------|
| `crates/cclab-pg-mamba/tests/methods_test.rs` | cclab-pg-mamba | 19 mb_pg_* | ≥32 |
| `crates/cclab-api-mamba/tests/methods_test.rs` | cclab-api-mamba | 17 mb_api_* | ≥27 |
| `crates/cclab-runtime-mamba/tests/methods_test.rs` | cclab-runtime-mamba | 4 mb_runtime_* | ≥10 |
| `crates/cclab-agent-mamba/tests/methods_test.rs` | cclab-agent-mamba | 13 mb_agent_* | ≥23 |
| `crates/cclab-fetch-mamba/tests/methods_test.rs` | cclab-fetch-mamba | 8 mb_fetch_* | ≥16 |
| `crates/cclab-log-mamba/tests/methods_test.rs` | cclab-log-mamba | 5 mb_log_* | ≥12 |
| `crates/cclab-mcp-mamba/tests/methods_test.rs` | cclab-mcp-mamba | 6 mb_mcp_* | ≥14 |

### Test Categories Per Function

1. **Happy path** — valid inputs; assert return type (`is_ptr()`, `is_int()`, `is_none()`) and value
2. **Null/missing args** — `null ptr` input or `nargs=0`; assert graceful return (no panic, return `none()` or sensible default)
3. **Boundary** — edge values (empty string, zero, negative); assert correct behavior per contract

### Shared Test Helper Pattern

```rust
// All integration tests use this pattern to create string MbValues:
fn make_str_val(s: &str) -> MbValue {
    MbValue::from_ptr(Box::into_raw(Box::new(s.to_string())) as usize)
}

// For reading string MbValues back:
unsafe fn read_str_val(v: MbValue) -> String {
    let addr = v.as_ptr().unwrap();
    unsafe { &*(addr as *const String) }.clone()
}
```

### Network-Dependent Functions

Functions that make real network calls (`mb_pg_connect`, `mb_pg_execute`, `mb_fetch_client_get/post/put/delete`) MUST be tested in offline/error mode only:
- `mb_pg_connect` with an invalid URL → `connected==false`
- `mb_pg_execute` with a null pool ptr → `none()`
- `mb_fetch_client_*` with empty base URL or null ptr → error response (`status==0`)

No `#[ignore]` tags needed — error paths do not require external services.

### Registry Audit

```bash
# Run to verify no ignored tests remain:
cargo test -p mamba-registry -- --ignored
# Expected: 0 tests filtered (no #[ignore] in current code)
cargo test -p mamba-registry
# Expected: 17 tests pass
```

### Verification Commands

```bash
# Run all binding crate tests
cargo test -p cclab-pg-mamba -p cclab-api-mamba -p cclab-runtime-mamba \
           -p cclab-agent-mamba -p cclab-fetch-mamba \
           -p cclab-log-mamba -p cclab-mcp-mamba -p mamba-registry

# Count total tests (should be ≥ 120 new + 58 existing inline)
cargo test --workspace 2>&1 | grep -E 'test result'
```
## Changes

## Changes

```yaml
files:
  - path: crates/cclab-pg-mamba/tests/methods_test.rs
    action: CREATE
    desc: |
      Integration tests for all 19 mb_pg_* functions.
      Covers: mb_pg_connect (offline), mb_pg_execute (null/empty), mb_pg_query_builder_* (new/build/select/where/limit/order_by),
      mb_pg_declarative_base_new, mb_pg_table_name_set, mb_pg_mapped_column, mb_pg_relationship,
      mb_pg_foreign_key, mb_pg_index, mb_pg_type_{string,text,json,uuid,datetime}.
      Tests: ≥32 (happy + null-ptr/error per function).

  - path: crates/cclab-api-mamba/tests/methods_test.rs
    action: CREATE
    desc: |
      Integration tests for all 17 mb_api_* functions.
      Covers: mb_api_router_{new,add_get,add_post,add_put,add_delete,add_patch,routes_count},
      mb_api_depends_new, mb_api_http_exception_new, mb_api_request_{new,method,path,query_param},
      mb_api_response_{new,json}, mb_api_background_tasks_{new,add}.
      Tests: ≥27 (happy + null-ptr/error per function).

  - path: crates/cclab-runtime-mamba/tests/methods_test.rs
    action: CREATE
    desc: |
      Integration tests for all 4 mb_runtime_* functions.
      Covers: mb_runtime_sleep (float/int/negative/zero), mb_runtime_spawn (happy/unique-ids/no-args),
      mb_runtime_gather (stub no-args/with-list).
      Tests: ≥10.

  - path: crates/cclab-agent-mamba/tests/methods_test.rs
    action: CREATE
    desc: |
      Integration tests for all 13 mb_agent_* functions.
      Covers: mb_agent_builder_{new,provider,system_prompt,build}, mb_agent_run,
      mb_agent_{claude,gemini,openai}_provider, mb_agent_message_{new,role,content},
      mb_agent_tool_registry_{new,register}.
      Tests: ≥23 (happy + null-ptr/error per function).

  - path: crates/cclab-fetch-mamba/tests/methods_test.rs
    action: CREATE
    desc: |
      Integration tests for all 8 mb_fetch_* functions.
      Covers: mb_fetch_client_new (url/timeout/defaults), mb_fetch_response_{status,text,json},
      mb_fetch_client_{get,post,put,delete} in offline/null-ptr mode only.
      Tests: ≥16.

  - path: crates/cclab-log-mamba/tests/methods_test.rs
    action: CREATE
    desc: |
      Integration tests for all 5 mb_log_* functions.
      Covers: mb_log_get_logger (named/default/empty), mb_log_{info,error,debug,warning}
      (returns none, null-logger no-crash, multi-level sequence).
      Tests: ≥12.

  - path: crates/cclab-mcp-mamba/tests/methods_test.rs
    action: CREATE
    desc: |
      Integration tests for all 6 mb_mcp_* functions.
      Covers: mb_mcp_server_{new,register_tool,tool_count,run_stdio,streamable_http_app,name}.
      Tests: ≥14 (happy + null-ptr/zero-arg per function).
```
## Wireframe
<!-- type: wireframe lang: yaml -->

<!-- TODO -->

## Component
<!-- type: component lang: json -->

<!-- TODO -->

## Design Token
<!-- type: design-token lang: json -->

<!-- TODO -->

## Doc
<!-- type: doc lang: markdown -->

<!-- TODO -->

# Reviews
