# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "operator"
# dimension = "surface"
# case = "api_methodcaller_is_present"
# subject = "operator.methodcaller"
# kind = "mechanical"
# xfail = ""
# mem_carveout = ""
# source = "projects/mamba/data/cpython312_surface.json"
# status = "filled"
# ///
"""operator.methodcaller: api_methodcaller_is_present (surface)."""
import operator

assert hasattr(operator, "methodcaller")
print("api_methodcaller_is_present OK")
