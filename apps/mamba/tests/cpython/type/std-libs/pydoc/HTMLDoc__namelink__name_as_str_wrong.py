# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "pydoc"
# dimension = "type"
# case = "HTMLDoc__namelink__name_as_str_wrong"
# subject = "pydoc.HTMLDoc.namelink(name: str)"
# kind = "semantic"
# xfail = "force-typed arg enforcement pending; mamba must raise TypeError on wrong-typed name"
# mem_carveout = ""
# source = "vendor/typeshed/stdlib/pydoc.pyi"
# status = "filled"
# ///
# mamba-xfail: force-typed arg enforcement pending; mamba must raise TypeError on wrong-typed name
# mamba-strict-type: TypeError
"""Type wall: pydoc.HTMLDoc.namelink(name: str); call it with the wrong type.

typeshed contract: name is str. mamba is force-typed, so a wrong-typed
argument MUST raise TypeError (CPython may accept or raise — mamba's to enforce)."""

from pydoc import HTMLDoc
obj = object.__new__(HTMLDoc)
try:
    obj.namelink(12345)  # name: str <- wrong-typed
    print("no_typeerror:")  # CPython accepted the wrong-typed arg; mamba must raise
except TypeError as e:
    print("typeerror:", type(e).__name__)
except Exception as e:
    print("setup_or_other:", type(e).__name__)
