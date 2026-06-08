# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "getopt"
# dimension = "type"
# case = "getopt__args_as__SliceableT_wrong"
# subject = "getopt.getopt(args: _SliceableT)"
# kind = "semantic"
# xfail = "force-typed arg enforcement pending; mamba must raise TypeError on wrong-typed args"
# mem_carveout = ""
# source = "vendor/typeshed/stdlib/getopt.pyi"
# status = "filled"
# ///
# mamba-xfail: force-typed arg enforcement pending; mamba must raise TypeError on wrong-typed args
# mamba-strict-type: TypeError
"""Type wall: getopt.getopt(args: _SliceableT); call it with the wrong type.

typeshed contract: args is _SliceableT. mamba is force-typed, so a wrong-typed
argument MUST raise TypeError (CPython may accept or raise — mamba's to enforce)."""

class _W:
    pass


from getopt import getopt
try:
    getopt(_W(), "")  # args: _SliceableT <- wrong-typed
    print("no_typeerror:")  # CPython accepted the wrong-typed arg; mamba must raise
except TypeError as e:
    print("typeerror:", type(e).__name__)
except Exception as e:
    print("setup_or_other:", type(e).__name__)
