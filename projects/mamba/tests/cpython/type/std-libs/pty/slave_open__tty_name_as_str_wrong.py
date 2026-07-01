# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "pty"
# dimension = "type"
# case = "slave_open__tty_name_as_str_wrong"
# subject = "pty.slave_open(tty_name: str)"
# kind = "semantic"
# mem_carveout = ""
# source = "vendor/typeshed/stdlib/pty.pyi"
# status = "filled"
# ///
# mamba-strict-type: TypeError
"""Type wall: pty.slave_open(tty_name: str); call it with the wrong type.

typeshed contract: tty_name is str. mamba is force-typed, so a wrong-typed
argument MUST raise TypeError (CPython may accept or raise — mamba's to enforce)."""

from pty import slave_open
try:
    slave_open(12345)  # tty_name: str <- wrong-typed
    print("no_typeerror:")  # CPython accepted the wrong-typed arg; mamba must raise
except TypeError as e:
    print("typeerror:", type(e).__name__)
except Exception as e:
    print("setup_or_other:", type(e).__name__)
