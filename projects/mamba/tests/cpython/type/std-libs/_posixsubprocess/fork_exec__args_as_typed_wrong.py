# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "_posixsubprocess"
# dimension = "type"
# case = "fork_exec__args_as_typed_wrong"
# subject = "_posixsubprocess.fork_exec(args: typed)"
# kind = "semantic"
# xfail = "force-typed arg enforcement pending; mamba must raise TypeError on wrong-typed args"
# mem_carveout = ""
# source = "vendor/typeshed/stdlib/_posixsubprocess.pyi"
# status = "filled"
# ///
# mamba-xfail: force-typed arg enforcement pending; mamba must raise TypeError on wrong-typed args
# mamba-strict-type: TypeError
"""Type wall: _posixsubprocess.fork_exec(args: typed); call it with the wrong type.

typeshed contract: args is typed. mamba is force-typed, so a wrong-typed
argument MUST raise TypeError (CPython may accept or raise — mamba's to enforce)."""

class _W:
    pass


from _posixsubprocess import fork_exec
try:
    fork_exec(_W(), None, True, None, "", None, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, None, None, None, 0, None)  # args: typed <- wrong-typed
    print("no_typeerror:")  # CPython accepted the wrong-typed arg; mamba must raise
except TypeError as e:
    print("typeerror:", type(e).__name__)
except Exception as e:
    print("setup_or_other:", type(e).__name__)
