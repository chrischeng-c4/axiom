# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "inspect"
# dimension = "surface"
# case = "api_cleandoc_is_present"
# subject = "inspect.cleandoc"
# kind = "mechanical"
# xfail = ""
# mem_carveout = ""
# source = "projects/mamba/data/cpython312_surface.json"
# status = "filled"
# ///
"""inspect.cleandoc: api_cleandoc_is_present (surface)."""
import inspect

assert hasattr(inspect, "cleandoc")
print("api_cleandoc_is_present OK")
