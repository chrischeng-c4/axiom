# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "cmd"
# dimension = "type"
# case = "Cmd__init__completekey_as_str_wrong"
# subject = "cmd.Cmd.__init__(completekey: str)"
# kind = "semantic"
# xfail = ""
# mem_carveout = ""
# source = "vendor/typeshed/stdlib/cmd.pyi"
# status = "filled"
# ///
# mamba-strict-type: TypeError
"""Type wall: cmd.Cmd.__init__(completekey: str); call it with the wrong type.

typeshed contract: completekey is str. mamba is force-typed, so a wrong-typed
argument MUST raise TypeError (CPython may accept or raise — mamba's to enforce)."""

from cmd import Cmd
try:
    Cmd(12345)  # completekey: str <- wrong-typed
    print("no_typeerror:")  # CPython accepted the wrong-typed arg; mamba must raise
except TypeError as e:
    print("typeerror:", type(e).__name__)
except Exception as e:
    print("setup_or_other:", type(e).__name__)
