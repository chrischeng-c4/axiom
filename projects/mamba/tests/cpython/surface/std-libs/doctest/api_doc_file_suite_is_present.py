# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "doctest"
# dimension = "surface"
# case = "api_doc_file_suite_is_present"
# subject = "doctest.DocFileSuite"
# kind = "mechanical"
# xfail = ""
# mem_carveout = ""
# source = "projects/mamba/data/cpython312_surface.json"
# status = "filled"
# ///
"""doctest.DocFileSuite: api_doc_file_suite_is_present (surface)."""
import doctest

assert hasattr(doctest, "DocFileSuite")
print("api_doc_file_suite_is_present OK")
