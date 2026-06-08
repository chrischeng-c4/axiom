# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "subprocess"
# dimension = "type"
# case = "Popen__init__args_as__CMD_wrong"
# subject = "subprocess.Popen.__init__(args: _CMD)"
# kind = "semantic"
# xfail = "force-typed arg enforcement pending; mamba must raise TypeError on wrong-typed args"
# mem_carveout = ""
# source = "vendor/typeshed/stdlib/subprocess.pyi"
# status = "filled"
# ///
# mamba-xfail: force-typed arg enforcement pending; mamba must raise TypeError on wrong-typed args
# mamba-strict-type: TypeError
"""Type wall: subprocess.Popen.__init__(args: _CMD); call it with the wrong type.

typeshed contract: args is _CMD. mamba is force-typed, so a wrong-typed
argument MUST raise TypeError (CPython may accept or raise — mamba's to enforce)."""

class _W:
    pass


from subprocess import Popen
try:
    Popen(_W())  # args: _CMD <- wrong-typed
    print("no_typeerror:")  # CPython accepted the wrong-typed arg; mamba must raise
except TypeError as e:
    print("typeerror:", type(e).__name__)
except Exception as e:
    print("setup_or_other:", type(e).__name__)
