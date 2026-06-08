# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "pep"
# lib = "variable_annotations"
# dimension = "behavior"
# case = "module_annotations_is_dict"
# subject = "__annotations__"
# kind = "semantic"
# xfail = "module __annotations__ is an undefined name on mamba. See project_mamba_pep_silent_divergences_2026_05_27."
# mem_carveout = ""
# source = ""
# status = "filled"
# ///
"""__annotations__: module-level __annotations__ is a plain dict after a `score: int = 0` annotation"""

score: int = 0
assert isinstance(__annotations__, dict), type(__annotations__)
assert "score" in __annotations__, __annotations__
print("module_annotations_is_dict OK")
