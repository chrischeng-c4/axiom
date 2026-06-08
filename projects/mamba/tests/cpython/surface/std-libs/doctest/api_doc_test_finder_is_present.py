# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "doctest"
# dimension = "surface"
# case = "api_doc_test_finder_is_present"
# subject = "doctest.DocTestFinder"
# kind = "mechanical"
# xfail = ""
# mem_carveout = ""
# source = "projects/mamba/data/cpython312_surface.json"
# status = "filled"
# ///
"""doctest.DocTestFinder: api_doc_test_finder_is_present (surface)."""
import doctest

assert hasattr(doctest, "DocTestFinder")
print("api_doc_test_finder_is_present OK")
