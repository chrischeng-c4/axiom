# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "pep"
# lib = "526"
# dimension = "behavior"
# case = "class_annotation_records"
# subject = "__annotations__"
# kind = "semantic"
# xfail = "class __annotations__ is an undefined name on mamba. See project_mamba_pep_silent_divergences_2026_05_27."
# mem_carveout = ""
# source = ""
# status = "filled"
# ///
"""__annotations__: class-body annotations `a: int = 1; b: str = 'hi'` are recorded in the class __annotations__ mapping (keys {'a', 'b'})"""


class C:
    a: int = 1
    b: str = "hi"


assert sorted(C.__annotations__.keys()) == ["a", "b"], C.__annotations__
print("class_annotation_records OK")
