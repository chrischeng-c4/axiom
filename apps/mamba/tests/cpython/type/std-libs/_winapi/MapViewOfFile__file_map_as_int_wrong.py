# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "_winapi"
# dimension = "type"
# case = "MapViewOfFile__file_map_as_int_wrong"
# subject = "_winapi.MapViewOfFile(file_map: int)"
# kind = "semantic"
# xfail = ""
# mem_carveout = ""
# source = "vendor/typeshed/stdlib/_winapi.pyi"
# status = "filled"
# ///
# mamba-strict-type: TypeError
"""Type wall: _winapi.MapViewOfFile(file_map: int); call it with the wrong type.

typeshed contract: file_map is int. mamba is force-typed, so a wrong-typed
argument MUST raise TypeError (CPython may accept or raise — mamba's to enforce)."""

from _winapi import MapViewOfFile
try:
    MapViewOfFile("not_an_int", 0, 0, 0, 0)  # file_map: int <- wrong-typed
    print("no_typeerror:")  # CPython accepted the wrong-typed arg; mamba must raise
except TypeError as e:
    print("typeerror:", type(e).__name__)
except Exception as e:
    print("setup_or_other:", type(e).__name__)
