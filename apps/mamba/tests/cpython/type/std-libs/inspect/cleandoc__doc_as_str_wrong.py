# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "inspect"
# dimension = "type"
# case = "cleandoc__doc_as_str_wrong"
# subject = "inspect.cleandoc(doc: str)"
# kind = "semantic"
# xfail = ""
# mem_carveout = ""
# source = "vendor/typeshed/stdlib/inspect.pyi"
# status = "filled"
# ///
# mamba-strict-type: TypeError
"""Type wall: inspect.cleandoc(doc: str); call it with the wrong type.

typeshed contract: doc is str. mamba is force-typed, so a wrong-typed
argument MUST raise TypeError (CPython may accept or raise — mamba's to enforce)."""

from inspect import cleandoc
try:
    cleandoc(12345)  # doc: str <- wrong-typed
    print("no_typeerror:")  # CPython accepted the wrong-typed arg; mamba must raise
except TypeError as e:
    print("typeerror:", type(e).__name__)
except Exception as e:
    print("setup_or_other:", type(e).__name__)
