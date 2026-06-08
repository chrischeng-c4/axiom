# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "typing"
# dimension = "surface"
# case = "api_class_var_is_present"
# subject = "typing.ClassVar"
# kind = "mechanical"
# xfail = ""
# mem_carveout = ""
# source = "projects/mamba/data/cpython312_surface.json"
# status = "filled"
# ///
"""typing.ClassVar: api_class_var_is_present (surface)."""
import typing

assert hasattr(typing, "ClassVar")
print("api_class_var_is_present OK")
