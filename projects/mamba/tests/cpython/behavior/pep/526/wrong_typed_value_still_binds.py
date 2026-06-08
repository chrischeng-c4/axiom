# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "pep"
# lib = "526"
# dimension = "behavior"
# case = "wrong_typed_value_still_binds"
# subject = "__annotations__"
# kind = "semantic"
# xfail = "mamba is force-typed; an annotation/value type mismatch is rejected rather than bound. See project_mamba_pep_silent_divergences_2026_05_27."
# mem_carveout = ""
# source = ""
# status = "filled"
# ///
"""__annotations__: CPython does not runtime type-check annotations: `z: int = 'not_an_int'` still binds z to the string (no TypeError)"""

# The annotation is documentation only; CPython never enforces it at runtime,
# so a str value flows straight into an int-annotated name.
z: int = "not_an_int"  # type: ignore[assignment]
assert z == "not_an_int", z
assert isinstance(z, str), type(z)
print("wrong_typed_value_still_binds OK")
