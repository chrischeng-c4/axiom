# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "_imp"
# dimension = "type"
# case = "exec_dynamic__mod_as_ModuleType_wrong"
# subject = "_imp.exec_dynamic(mod: ModuleType)"
# kind = "semantic"
# xfail = ""
# mem_carveout = ""
# source = "vendor/typeshed/stdlib/_imp.pyi"
# status = "filled"
# ///
# mamba-strict-type: TypeError
"""Type wall: _imp.exec_dynamic(mod: ModuleType); call it with the wrong type.

typeshed contract: mod is ModuleType. mamba is force-typed, so a wrong-typed
argument MUST raise TypeError (CPython may accept or raise — mamba's to enforce)."""

class _W:
    pass


from _imp import exec_dynamic
try:
    exec_dynamic(_W())  # mod: ModuleType <- wrong-typed
    print("no_typeerror:")  # CPython accepted the wrong-typed arg; mamba must raise
except TypeError as e:
    print("typeerror:", type(e).__name__)
except Exception as e:
    print("setup_or_other:", type(e).__name__)
