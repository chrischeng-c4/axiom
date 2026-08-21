# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "os"
# dimension = "type"
# case = "fspath__path_as_bytes_wrong"
# subject = "os.fspath(path: bytes)"
# kind = "semantic"
# xfail = ""
# mem_carveout = ""
# source = "vendor/typeshed/stdlib/os.pyi"
# status = "filled"
# ///
# mamba-strict-type: TypeError
"""Type wall: os.fspath(path: bytes); call it with the wrong type.

typeshed contract: path is bytes. mamba is force-typed, so a wrong-typed
argument MUST raise TypeError (CPython may accept or raise — mamba's to enforce)."""

from os import fspath
try:
    fspath(12345)  # path: bytes <- wrong-typed
    print("no_typeerror:")  # CPython accepted the wrong-typed arg; mamba must raise
except TypeError as e:
    print("typeerror:", type(e).__name__)
except Exception as e:
    print("setup_or_other:", type(e).__name__)
