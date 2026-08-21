# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "doctest"
# dimension = "type"
# case = "DocTest____lt____other_as_DocTest_wrong"
# subject = "doctest.DocTest.__lt__(other: DocTest)"
# kind = "semantic"
# mem_carveout = ""
# source = "vendor/typeshed/stdlib/doctest.pyi"
# status = "filled"
# ///
# mamba-strict-type: TypeError
"""Type wall: doctest.DocTest.__lt__(other: DocTest); call it with the wrong type.

typeshed contract: other is DocTest. mamba is force-typed, so a wrong-typed
argument MUST raise TypeError (CPython may accept or raise — mamba's to enforce)."""

class _W:
    pass


from doctest import DocTest
obj = object.__new__(DocTest)
try:
    obj.__lt__(_W())  # other: DocTest <- wrong-typed
    print("no_typeerror:")  # CPython accepted the wrong-typed arg; mamba must raise
except TypeError as e:
    print("typeerror:", type(e).__name__)
except Exception as e:
    print("setup_or_other:", type(e).__name__)
