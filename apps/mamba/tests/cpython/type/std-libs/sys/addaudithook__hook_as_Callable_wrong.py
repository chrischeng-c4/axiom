# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "sys"
# dimension = "type"
# case = "addaudithook__hook_as_Callable_wrong"
# subject = "sys.addaudithook(hook: Callable)"
# kind = "semantic"
# xfail = "force-typed arg enforcement pending; mamba must raise TypeError on wrong-typed hook"
# mem_carveout = ""
# source = "vendor/typeshed/stdlib/sys.pyi"
# status = "filled"
# ///
# mamba-xfail: force-typed arg enforcement pending; mamba must raise TypeError on wrong-typed hook
# mamba-strict-type: TypeError
"""Type wall: sys.addaudithook(hook: Callable); call it with the wrong type.

typeshed contract: hook is Callable. mamba is force-typed, so a wrong-typed
argument MUST raise TypeError (CPython may accept or raise — mamba's to enforce)."""

class _W:
    pass


from sys import addaudithook
try:
    addaudithook(_W())  # hook: Callable <- wrong-typed
    print("no_typeerror:")  # CPython accepted the wrong-typed arg; mamba must raise
except TypeError as e:
    print("typeerror:", type(e).__name__)
except Exception as e:
    print("setup_or_other:", type(e).__name__)
