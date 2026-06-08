# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "shlex"
# dimension = "type"
# case = "shlex__sourcehook__newfile_as_str_wrong"
# subject = "shlex.shlex.sourcehook(newfile: str)"
# kind = "semantic"
# xfail = ""
# mem_carveout = ""
# source = "vendor/typeshed/stdlib/shlex.pyi"
# status = "filled"
# ///
# mamba-strict-type: TypeError
"""Type wall: shlex.shlex.sourcehook(newfile: str); call it with the wrong type.

typeshed contract: newfile is str. mamba is force-typed, so a wrong-typed
argument MUST raise TypeError (CPython may accept or raise — mamba's to enforce)."""

from shlex import shlex
obj = object.__new__(shlex)
try:
    obj.sourcehook(12345)  # newfile: str <- wrong-typed
    print("no_typeerror:")  # CPython accepted the wrong-typed arg; mamba must raise
except TypeError as e:
    print("typeerror:", type(e).__name__)
except Exception as e:
    print("setup_or_other:", type(e).__name__)
