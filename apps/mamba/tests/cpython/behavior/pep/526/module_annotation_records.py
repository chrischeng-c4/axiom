# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "pep"
# lib = "526"
# dimension = "behavior"
# case = "module_annotation_records"
# subject = "__annotations__"
# kind = "semantic"
# xfail = "module __annotations__ is an undefined name on mamba. See project_mamba_pep_silent_divergences_2026_05_27."
# mem_carveout = ""
# source = ""
# status = "filled"
# ///
"""__annotations__: an annotated module-scope assignment `x: int = 1` records 'x' in the module __annotations__ mapping"""

x: int = 1
assert "x" in __annotations__, __annotations__
print("module_annotation_records OK")
