---
change: 1132-patrol
group: wire-mamba-modules
date: 2026-04-04
---

# Requirements

Wire the compiler's import resolver to the MAMBA_MODULES distributed slice / RuntimeSymbol registry. When a mamba script does `from cclab.log import get_logger`, the import resolver should: (1) call find_module() to search MAMBA_MODULES for a matching module, (2) look up the RuntimeSymbol entry for the imported name, (3) resolve the Python name to the FFI function pointer via the expose table or alias mapping, (4) emit correct JIT codegen to call the FFI function. Currently all imported names resolve to 0.0 (default float).
