# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "importlib_resources_abc"
# dimension = "type"
# case = "Traversable__open__mode_as_Literal_wrong"
# subject = "importlib.resources.abc.Traversable.open(mode: Literal)"
# kind = "semantic"
# xfail = ""
# mem_carveout = ""
# source = "vendor/typeshed/stdlib/importlib/resources/abc.pyi"
# status = "filled"
# ///
# mamba-strict-type: TypeError
"""Type wall: importlib.resources.abc.Traversable.open(mode: Literal); call it with the wrong type.

typeshed contract: mode is Literal. mamba is force-typed, so a wrong-typed
argument MUST raise TypeError (CPython may accept or raise — mamba's to enforce)."""

class _W:
    pass


from importlib.resources.abc import Traversable
obj = object.__new__(Traversable)
try:
    obj.open(_W())  # mode: Literal <- wrong-typed
    print("no_typeerror:")  # CPython accepted the wrong-typed arg; mamba must raise
except TypeError as e:
    print("typeerror:", type(e).__name__)
except Exception as e:
    print("setup_or_other:", type(e).__name__)
