# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "importlib_abc"
# dimension = "type"
# case = "Traversable__joinpath__child_as_str_wrong"
# subject = "importlib.abc.Traversable.joinpath(child: str)"
# kind = "semantic"
# xfail = "force-typed arg enforcement pending; mamba must raise TypeError on wrong-typed child"
# mem_carveout = ""
# source = "vendor/typeshed/stdlib/importlib/abc.pyi"
# status = "filled"
# ///
# mamba-xfail: force-typed arg enforcement pending; mamba must raise TypeError on wrong-typed child
# mamba-strict-type: TypeError
"""Type wall: importlib.abc.Traversable.joinpath(child: str); call it with the wrong type.

typeshed contract: child is str. mamba is force-typed, so a wrong-typed
argument MUST raise TypeError (CPython may accept or raise — mamba's to enforce)."""

from importlib.abc import Traversable
obj = object.__new__(Traversable)
try:
    obj.joinpath(12345)  # child: str <- wrong-typed
    print("no_typeerror:")  # CPython accepted the wrong-typed arg; mamba must raise
except TypeError as e:
    print("typeerror:", type(e).__name__)
except Exception as e:
    print("setup_or_other:", type(e).__name__)
