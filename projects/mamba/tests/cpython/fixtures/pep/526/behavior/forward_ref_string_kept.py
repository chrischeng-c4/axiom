# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "pep"
# lib = "526"
# dimension = "behavior"
# case = "forward_ref_string_kept"
# subject = "__annotations__"
# kind = "semantic"
# xfail = "function __annotations__ returns None on mamba. See project_mamba_pep_silent_divergences_2026_05_27."
# mem_carveout = ""
# source = ""
# status = "filled"
# ///
"""__annotations__: a string forward-reference annotation `x: 'Undefined'` is stored verbatim as the string 'Undefined' in __annotations__ (never evaluated)"""


# The annotation names a type that does not exist; given as a string it is
# stored as-is and never resolved, so defining the function does not raise.
def lazy(x: "Undefined") -> "Undefined":  # type: ignore[name-defined]  # noqa: F821
    return x


assert lazy.__annotations__.get("x") == "Undefined", lazy.__annotations__
assert lazy.__annotations__.get("return") == "Undefined", lazy.__annotations__
print("forward_ref_string_kept OK")
