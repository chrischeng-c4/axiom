# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "copyreg"
# dimension = "type"
# case = "remove_extension__module_as_Hashable_wrong"
# subject = "copyreg.remove_extension(module: Hashable)"
# kind = "semantic"
# mem_carveout = ""
# source = "vendor/typeshed/stdlib/copyreg.pyi"
# status = "filled"
# ///
# mamba-strict-type: TypeError
"""Type wall: copyreg.remove_extension(module: Hashable); call it with the wrong type.

typeshed contract: module is Hashable. mamba is force-typed, so a wrong-typed
argument MUST raise TypeError (CPython may accept or raise — mamba's to enforce)."""

class _W:
    pass


from copyreg import remove_extension
try:
    remove_extension(_W(), None, 0)  # module: Hashable <- wrong-typed
    print("no_typeerror:")  # CPython accepted the wrong-typed arg; mamba must raise
except TypeError as e:
    print("typeerror:", type(e).__name__)
except Exception as e:
    print("setup_or_other:", type(e).__name__)
