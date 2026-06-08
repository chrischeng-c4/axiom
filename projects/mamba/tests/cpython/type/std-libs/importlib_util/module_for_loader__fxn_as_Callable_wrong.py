# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "importlib_util"
# dimension = "type"
# case = "module_for_loader__fxn_as_Callable_wrong"
# subject = "importlib.util.module_for_loader(fxn: Callable)"
# kind = "semantic"
# xfail = "force-typed arg enforcement pending; mamba must raise TypeError on wrong-typed fxn"
# mem_carveout = ""
# source = "vendor/typeshed/stdlib/importlib/util.pyi"
# status = "filled"
# ///
# mamba-xfail: force-typed arg enforcement pending; mamba must raise TypeError on wrong-typed fxn
# mamba-strict-type: TypeError
"""Type wall: importlib.util.module_for_loader(fxn: Callable); call it with the wrong type.

typeshed contract: fxn is Callable. mamba is force-typed, so a wrong-typed
argument MUST raise TypeError (CPython may accept or raise — mamba's to enforce)."""

class _W:
    pass


from importlib.util import module_for_loader
try:
    module_for_loader(_W())  # fxn: Callable <- wrong-typed
    print("no_typeerror:")  # CPython accepted the wrong-typed arg; mamba must raise
except TypeError as e:
    print("typeerror:", type(e).__name__)
except Exception as e:
    print("setup_or_other:", type(e).__name__)
