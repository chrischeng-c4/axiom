---
change: sdd-index-scoped-toolchain
group: scoped-toolchain
date: 2026-03-26
---

# Requirements

1. Index config model:
   - New IndexConfig struct with auto_discover: bool and scopes: Vec<ScopeConfig>
   - ScopeConfig: id, lang (rust/python/typescript), root, search_paths, interpreter
   - Deserialize from [index] + [[index.scope]] in cclab/config.toml
2. Auto-discovery:
   - Scan project root for Cargo.toml (workspace), pyproject.toml, tsconfig.json+package.json
   - For Rust: cargo metadata for dep paths
   - For Python: detect .venv nearby, derive site-packages path
   - For TypeScript: parse tsconfig.json compilerOptions.paths, add node_modules
3. Multi-handler daemon:
   - Replace single handler: Arc<RequestHandler> with handlers: HashMap<String, Arc<RequestHandler>>
   - File path to scope routing by longest prefix match
   - Per-scope cache directories: cclab/.index/scopes/{id}/cache/
4. Per-scope handler init:
   - RequestHandler::new_with_scope(scope) sets up StubLoader with scope search_paths
   - ImportResolver gets scope-specific search_paths
5. CLI:
   - cclab sdd scope list — show discovered scopes
   - Pass index config through daemon startup
