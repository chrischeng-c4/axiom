# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "pep"
# lib = "variable_annotations"
# dimension = "behavior"
# case = "final_not_enforced_at_runtime"
# subject = "__annotations__"
# kind = "semantic"
# xfail = "mamba is force-typed; reassigning a Final-annotated name may be rejected, and the underlying annotation machinery diverges. See project_mamba_pep_silent_divergences_2026_05_27."
# mem_carveout = ""
# source = ""
# status = "filled"
# ///
"""__annotations__: a `Final` annotation is a type-checker hint only; at runtime `MAX: Final = 100` can be reassigned to 200 with no error"""
from typing import Final

# Final is enforced by type checkers, never at runtime.
MAX: Final = 100
assert MAX == 100, MAX
MAX = 200  # type: ignore[misc]
assert MAX == 200, "Final not enforced at runtime"
print("final_not_enforced_at_runtime OK")
