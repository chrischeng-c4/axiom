# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "sys"
# dimension = "type"
# case = "set_lazy_imports_filter__filter_as_typed_wrong"
# subject = "sys.set_lazy_imports_filter(filter: typed)"
# kind = "semantic"
# xfail = ""
# mem_carveout = ""
# source = "vendor/typeshed/stdlib/sys.pyi"
# status = "filled"
# ///
# mamba-strict-type: TypeError
"""Type wall: sys.set_lazy_imports_filter(filter: typed); call it with the wrong type.

typeshed contract: filter is typed. mamba is force-typed, so a wrong-typed
argument MUST raise TypeError (CPython may accept or raise — mamba's to enforce)."""

class _W:
    pass


from sys import set_lazy_imports_filter
try:
    set_lazy_imports_filter(_W())  # filter: typed <- wrong-typed
    print("no_typeerror:")  # CPython accepted the wrong-typed arg; mamba must raise
except TypeError as e:
    print("typeerror:", type(e).__name__)
except Exception as e:
    print("setup_or_other:", type(e).__name__)
