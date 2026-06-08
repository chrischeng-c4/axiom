# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "doctest"
# dimension = "surface"
# case = "api_doc_test_parser_is_present"
# subject = "doctest.DocTestParser"
# kind = "mechanical"
# xfail = ""
# mem_carveout = ""
# source = "projects/mamba/data/cpython312_surface.json"
# status = "filled"
# ///
"""doctest.DocTestParser: api_doc_test_parser_is_present (surface)."""
import doctest

assert hasattr(doctest, "DocTestParser")
print("api_doc_test_parser_is_present OK")
