# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "pep"
# lib = "526"
# dimension = "behavior"
# case = "annotated_value_binds"
# subject = "__annotations__"
# kind = "semantic"
# xfail = "module __annotations__ is an undefined name on mamba. See project_mamba_pep_silent_divergences_2026_05_27."
# mem_carveout = ""
# source = ""
# status = "filled"
# ///
"""__annotations__: an annotated assignment with a value `x: int = 42` binds the name to that value (42) like a normal assignment"""

x: int = 42
assert x == 42, x
assert "x" in __annotations__, __annotations__
print("annotated_value_binds OK")
