# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "doctest"
# dimension = "type"
# case = "DocTestRunner__merge__other_as_DocTestRunner_wrong"
# subject = "doctest.DocTestRunner.merge(other: DocTestRunner)"
# kind = "semantic"
# mem_carveout = ""
# source = "vendor/typeshed/stdlib/doctest.pyi"
# status = "filled"
# ///
# mamba-strict-type: TypeError
"""Type wall: doctest.DocTestRunner.merge(other: DocTestRunner); call it with the wrong type.

typeshed contract: other is DocTestRunner. mamba is force-typed, so a wrong-typed
argument MUST raise TypeError (CPython may accept or raise — mamba's to enforce)."""

class _W:
    pass


from doctest import DocTestRunner
obj = object.__new__(DocTestRunner)
try:
    obj.merge(_W())  # other: DocTestRunner <- wrong-typed
    print("no_typeerror:")  # CPython accepted the wrong-typed arg; mamba must raise
except TypeError as e:
    print("typeerror:", type(e).__name__)
except Exception as e:
    print("setup_or_other:", type(e).__name__)
