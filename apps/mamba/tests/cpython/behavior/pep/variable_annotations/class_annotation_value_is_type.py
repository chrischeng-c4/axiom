# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "pep"
# lib = "variable_annotations"
# dimension = "behavior"
# case = "class_annotation_value_is_type"
# subject = "__annotations__"
# kind = "semantic"
# xfail = "class __annotations__ is an undefined name on mamba. See project_mamba_pep_silent_divergences_2026_05_27."
# mem_carveout = ""
# source = ""
# status = "filled"
# ///
"""__annotations__: the recorded class annotation value is the annotation object itself: C.__annotations__['host'] is str"""


class Config:
    host: str = "localhost"
    port: int = 8080


assert Config.__annotations__["host"] is str, Config.__annotations__["host"]
assert Config.__annotations__["port"] is int, Config.__annotations__["port"]
print("class_annotation_value_is_type OK")
