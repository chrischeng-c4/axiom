# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "doctest"
# dimension = "surface"
# case = "api_script_from_examples_is_present"
# subject = "doctest.script_from_examples"
# kind = "mechanical"
# xfail = ""
# mem_carveout = ""
# source = "projects/mamba/data/cpython312_surface.json"
# status = "filled"
# ///
"""doctest.script_from_examples: api_script_from_examples_is_present (surface)."""
import doctest

assert hasattr(doctest, "script_from_examples")
print("api_script_from_examples_is_present OK")
