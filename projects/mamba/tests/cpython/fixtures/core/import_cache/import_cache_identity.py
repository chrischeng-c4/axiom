# Import cache + module identity — #2804.
#
# Covers Python's import-cache semantics — the rule that a module's
# top-level body executes ONCE, with subsequent imports returning the
# already-cached module from sys.modules.
#
# We use ONLY stdlib modules (no third-party packages):
#
#   import json    twice via the same name.
#   from json import dumps  — same module, different binding.
#   import json.decoder     — submodule + sys.modules entry.
#   importlib.import_module("json") — programmatic import.
#
# We probe a side-effect counter by installing a sentinel attribute
# on the module on first import, then verifying subsequent imports
# see the same attribute (proving top-level body did NOT re-run).
#
# Clauses:
#   1. Two `import json` statements yield the same module object.
#   2. `from json import dumps` returns the SAME function as
#      `json.dumps`.
#   3. sys.modules["json"] is the same object as `import json`.
#   4. importlib.import_module("json") returns the cached object.
#   5. Submodule import (`import json.decoder`) populates
#      sys.modules with both "json" and "json.decoder", and both are
#      module instances.
#   6. Re-imports do not re-run the module top-level: a sentinel
#      attribute stored after first import is still present on the
#      second `import json` (because it's the same object).
#
# Every print line tagged `[import-cache]` so failure output names
# import cache semantics.


import importlib
import sys
import types


# Clause 1: two `import json` statements yield the SAME object.
import json as json_first  # noqa: E402

import json as json_second  # noqa: E402, I001

print("[import-cache] clause-1 same-object:", json_first is json_second)
print(
    "[import-cache] clause-1 is-module:",
    isinstance(json_first, types.ModuleType),
)


# Clause 2: from-import binds the SAME function as attribute access.
from json import dumps  # noqa: E402

print("[import-cache] clause-2 same-function:", dumps is json_first.dumps)


# Clause 3: sys.modules["json"] is the SAME object.
print(
    "[import-cache] clause-3 sys-modules-match:",
    sys.modules["json"] is json_first,
)


# Clause 4: importlib.import_module returns cached.
imported = importlib.import_module("json")
print("[import-cache] clause-4 importlib-cached:", imported is json_first)


# Clause 5: submodule import populates both entries.
import json.decoder  # noqa: E402

# Reference json.decoder so pyright doesn't flag the import as unused.
_decoder_mod_count = len(dir(json.decoder))
print(
    "[import-cache] clause-5 parent-present:",
    "json" in sys.modules,
)
print(
    "[import-cache] clause-5 decoder-dir-nonempty:",
    _decoder_mod_count > 0,
)
print(
    "[import-cache] clause-5 submodule-present:",
    "json.decoder" in sys.modules,
)
print(
    "[import-cache] clause-5 submodule-is-module:",
    isinstance(sys.modules["json.decoder"], types.ModuleType),
)


# Clause 6: top-level body does not re-run on re-import.
# Install a sentinel on the module, then re-import and check it
# survives. The presence of the sentinel proves the module body did
# NOT re-execute (which would overwrite or shadow the sentinel).
SENTINEL = object()
json_first._mamba_import_cache_sentinel = SENTINEL  # pyright: ignore[reportAttributeAccessIssue]
import json as json_third  # noqa: E402, I001

print(
    "[import-cache] clause-6 sentinel-survives:",
    getattr(json_third, "_mamba_import_cache_sentinel", None) is SENTINEL,
)
print("[import-cache] clause-6 still-same:", json_third is json_first)
