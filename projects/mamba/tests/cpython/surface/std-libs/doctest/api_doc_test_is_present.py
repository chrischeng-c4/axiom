# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "doctest"
# dimension = "surface"
# case = "api_doc_test_is_present"
# subject = "doctest.DocTest"
# kind = "mechanical"
# xfail = ""
# mem_carveout = ""
# source = "projects/mamba/data/cpython312_surface.json"
# status = "filled"
# ///
"""doctest.DocTest: api_doc_test_is_present (surface)."""
import doctest

assert hasattr(doctest, "DocTest")
print("api_doc_test_is_present OK")
