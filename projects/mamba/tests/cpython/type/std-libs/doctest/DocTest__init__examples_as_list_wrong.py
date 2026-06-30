# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "doctest"
# dimension = "type"
# case = "DocTest__init__examples_as_list_wrong"
# subject = "doctest.DocTest.__init__(examples: list)"
# kind = "semantic"
# xfail = ""
# mem_carveout = ""
# source = "vendor/typeshed/stdlib/doctest.pyi"
# status = "filled"
# ///
# mamba-strict-type: TypeError
"""Type wall: doctest.DocTest.__init__(examples: list); call it with the wrong type.

typeshed contract: examples is list. mamba is force-typed, so a wrong-typed
argument MUST raise TypeError (CPython may accept or raise — mamba's to enforce)."""

from doctest import DocTest
try:
    DocTest(12345, None, "", None, None, None)  # examples: list <- wrong-typed
    print("no_typeerror:")  # CPython accepted the wrong-typed arg; mamba must raise
except TypeError as e:
    print("typeerror:", type(e).__name__)
except Exception as e:
    print("setup_or_other:", type(e).__name__)
