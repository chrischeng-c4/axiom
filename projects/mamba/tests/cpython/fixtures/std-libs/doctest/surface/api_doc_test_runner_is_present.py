# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "doctest"
# dimension = "surface"
# case = "api_doc_test_runner_is_present"
# subject = "doctest.DocTestRunner"
# kind = "mechanical"
# xfail = ""
# mem_carveout = ""
# source = "projects/mamba/data/cpython312_surface.json"
# status = "filled"
# ///
"""doctest.DocTestRunner: api_doc_test_runner_is_present (surface)."""
import doctest

assert hasattr(doctest, "DocTestRunner")
print("api_doc_test_runner_is_present OK")
