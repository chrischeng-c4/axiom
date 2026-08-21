# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "pep"
# lib = "variable_annotations"
# dimension = "behavior"
# case = "class_annotations_record_keys"
# subject = "__annotations__"
# kind = "semantic"
# xfail = "class __annotations__ is an undefined name on mamba. See project_mamba_pep_silent_divergences_2026_05_27."
# mem_carveout = ""
# source = ""
# status = "filled"
# ///
"""__annotations__: class-body annotations `host: str; port: int; debug: bool` are recorded in the class __annotations__ mapping (keys {'host', 'port', 'debug'})"""


class Config:
    host: str = "localhost"
    port: int = 8080
    debug: bool = False


assert sorted(Config.__annotations__.keys()) == ["debug", "host", "port"], Config.__annotations__
print("class_annotations_record_keys OK")
