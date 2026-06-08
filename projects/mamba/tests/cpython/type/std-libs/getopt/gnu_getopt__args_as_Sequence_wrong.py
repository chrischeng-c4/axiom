# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "getopt"
# dimension = "type"
# case = "gnu_getopt__args_as_Sequence_wrong"
# subject = "getopt.gnu_getopt(args: Sequence)"
# kind = "semantic"
# xfail = "force-typed arg enforcement pending; mamba must raise TypeError on wrong-typed args"
# mem_carveout = ""
# source = "vendor/typeshed/stdlib/getopt.pyi"
# status = "filled"
# ///
# mamba-xfail: force-typed arg enforcement pending; mamba must raise TypeError on wrong-typed args
# mamba-strict-type: TypeError
"""Type wall: getopt.gnu_getopt(args: Sequence); call it with the wrong type.

typeshed contract: args is Sequence. mamba is force-typed, so a wrong-typed
argument MUST raise TypeError (CPython may accept or raise — mamba's to enforce)."""

class _W:
    pass


from getopt import gnu_getopt
try:
    gnu_getopt(_W(), "")  # args: Sequence <- wrong-typed
    print("no_typeerror:")  # CPython accepted the wrong-typed arg; mamba must raise
except TypeError as e:
    print("typeerror:", type(e).__name__)
except Exception as e:
    print("setup_or_other:", type(e).__name__)
