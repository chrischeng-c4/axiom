# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "importlib_util"
# dimension = "type"
# case = "set_package__fxn_as_Callable_wrong"
# subject = "importlib.util.set_package(fxn: Callable)"
# kind = "semantic"
# mem_carveout = ""
# source = "vendor/typeshed/stdlib/importlib/util.pyi"
# status = "filled"
# ///
# mamba-strict-type: TypeError
"""Type wall: importlib.util.set_package(fxn: Callable); call it with the wrong type.

typeshed contract: fxn is Callable. mamba is force-typed, so a wrong-typed
argument MUST raise TypeError (CPython may accept or raise — mamba's to enforce)."""

class _W:
    pass


from importlib.util import set_package
try:
    set_package(_W())  # fxn: Callable <- wrong-typed
    print("no_typeerror:")  # CPython accepted the wrong-typed arg; mamba must raise
except TypeError as e:
    print("typeerror:", type(e).__name__)
except Exception as e:
    print("setup_or_other:", type(e).__name__)
