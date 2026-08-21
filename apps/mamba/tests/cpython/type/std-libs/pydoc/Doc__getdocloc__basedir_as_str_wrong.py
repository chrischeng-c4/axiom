# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "pydoc"
# dimension = "type"
# case = "Doc__getdocloc__basedir_as_str_wrong"
# subject = "pydoc.Doc.getdocloc(basedir: str)"
# kind = "semantic"
# xfail = ""
# mem_carveout = ""
# source = "vendor/typeshed/stdlib/pydoc.pyi"
# status = "filled"
# ///
# mamba-strict-type: TypeError
"""Type wall: pydoc.Doc.getdocloc(basedir: str); call it with the wrong type.

typeshed contract: basedir is str. mamba is force-typed, so a wrong-typed
argument MUST raise TypeError (CPython may accept or raise — mamba's to enforce)."""

from pydoc import Doc
obj = object.__new__(Doc)
try:
    obj.getdocloc(None, 12345)  # basedir: str <- wrong-typed
    print("no_typeerror:")  # CPython accepted the wrong-typed arg; mamba must raise
except TypeError as e:
    print("typeerror:", type(e).__name__)
except Exception as e:
    print("setup_or_other:", type(e).__name__)
