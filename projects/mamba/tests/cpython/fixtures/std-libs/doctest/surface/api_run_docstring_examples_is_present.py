# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "doctest"
# dimension = "surface"
# case = "api_run_docstring_examples_is_present"
# subject = "doctest.run_docstring_examples"
# kind = "mechanical"
# xfail = ""
# mem_carveout = ""
# source = "projects/mamba/data/cpython312_surface.json"
# status = "filled"
# ///
"""doctest.run_docstring_examples: api_run_docstring_examples_is_present (surface)."""
import doctest

assert hasattr(doctest, "run_docstring_examples")
print("api_run_docstring_examples_is_present OK")
