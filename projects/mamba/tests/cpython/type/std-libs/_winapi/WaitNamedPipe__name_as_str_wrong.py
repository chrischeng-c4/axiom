# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "_winapi"
# dimension = "type"
# case = "WaitNamedPipe__name_as_str_wrong"
# subject = "_winapi.WaitNamedPipe(name: str)"
# kind = "semantic"
# xfail = ""
# mem_carveout = ""
# source = "vendor/typeshed/stdlib/_winapi.pyi"
# status = "filled"
# ///
# mamba-strict-type: TypeError
"""Type wall: _winapi.WaitNamedPipe(name: str); call it with the wrong type.

typeshed contract: name is str. mamba is force-typed, so a wrong-typed
argument MUST raise TypeError (CPython may accept or raise — mamba's to enforce)."""

from _winapi import WaitNamedPipe
try:
    WaitNamedPipe(12345, 0)  # name: str <- wrong-typed
    print("no_typeerror:")  # CPython accepted the wrong-typed arg; mamba must raise
except TypeError as e:
    print("typeerror:", type(e).__name__)
except Exception as e:
    print("setup_or_other:", type(e).__name__)
