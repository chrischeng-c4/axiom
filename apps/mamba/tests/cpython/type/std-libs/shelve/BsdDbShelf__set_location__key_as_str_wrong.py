# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "shelve"
# dimension = "type"
# case = "BsdDbShelf__set_location__key_as_str_wrong"
# subject = "shelve.BsdDbShelf.set_location(key: str)"
# kind = "semantic"
# xfail = ""
# mem_carveout = ""
# source = "vendor/typeshed/stdlib/shelve.pyi"
# status = "filled"
# ///
# mamba-strict-type: TypeError
"""Type wall: shelve.BsdDbShelf.set_location(key: str); call it with the wrong type.

typeshed contract: key is str. mamba is force-typed, so a wrong-typed
argument MUST raise TypeError (CPython may accept or raise — mamba's to enforce)."""

from shelve import BsdDbShelf
obj = object.__new__(BsdDbShelf)
try:
    obj.set_location(12345)  # key: str <- wrong-typed
    print("no_typeerror:")  # CPython accepted the wrong-typed arg; mamba must raise
except TypeError as e:
    print("typeerror:", type(e).__name__)
except Exception as e:
    print("setup_or_other:", type(e).__name__)
