# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "sys"
# dimension = "type"
# case = "set_lazy_imports__mode_as__LazyImportMode_wrong"
# subject = "sys.set_lazy_imports(mode: _LazyImportMode)"
# kind = "semantic"
# xfail = ""
# mem_carveout = ""
# source = "vendor/typeshed/stdlib/sys.pyi"
# status = "filled"
# ///
# mamba-strict-type: TypeError
"""Type wall: sys.set_lazy_imports(mode: _LazyImportMode); call it with the wrong type.

typeshed contract: mode is _LazyImportMode. mamba is force-typed, so a wrong-typed
argument MUST raise TypeError (CPython may accept or raise — mamba's to enforce)."""

class _W:
    pass


from sys import set_lazy_imports
try:
    set_lazy_imports(_W())  # mode: _LazyImportMode <- wrong-typed
    print("no_typeerror:")  # CPython accepted the wrong-typed arg; mamba must raise
except TypeError as e:
    print("typeerror:", type(e).__name__)
except Exception as e:
    print("setup_or_other:", type(e).__name__)
