# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "sys"
# dimension = "type"
# case = "set_asyncgen_hooks__firstiter_as__AsyncgenHook_wrong"
# subject = "sys.set_asyncgen_hooks(firstiter: _AsyncgenHook)"
# kind = "semantic"
# xfail = ""
# mem_carveout = ""
# source = "vendor/typeshed/stdlib/sys.pyi"
# status = "filled"
# ///
# mamba-strict-type: TypeError
"""Type wall: sys.set_asyncgen_hooks(firstiter: _AsyncgenHook); call it with the wrong type.

typeshed contract: firstiter is _AsyncgenHook. mamba is force-typed, so a wrong-typed
argument MUST raise TypeError (CPython may accept or raise — mamba's to enforce)."""

class _W:
    pass


from sys import set_asyncgen_hooks
try:
    set_asyncgen_hooks(_W())  # firstiter: _AsyncgenHook <- wrong-typed
    print("no_typeerror:")  # CPython accepted the wrong-typed arg; mamba must raise
except TypeError as e:
    print("typeerror:", type(e).__name__)
except Exception as e:
    print("setup_or_other:", type(e).__name__)
