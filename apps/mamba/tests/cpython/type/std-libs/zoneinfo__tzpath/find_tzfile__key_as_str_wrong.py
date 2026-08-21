# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "zoneinfo__tzpath"
# dimension = "type"
# case = "find_tzfile__key_as_str_wrong"
# subject = "zoneinfo._tzpath.find_tzfile(key: str)"
# kind = "semantic"
# xfail = ""
# mem_carveout = ""
# source = "vendor/typeshed/stdlib/zoneinfo/_tzpath.pyi"
# status = "filled"
# ///
# mamba-strict-type: TypeError
"""Type wall: zoneinfo._tzpath.find_tzfile(key: str); call it with the wrong type.

typeshed contract: key is str. mamba is force-typed, so a wrong-typed
argument MUST raise TypeError (CPython may accept or raise — mamba's to enforce)."""

from zoneinfo._tzpath import find_tzfile
try:
    find_tzfile(12345)  # key: str <- wrong-typed
    print("no_typeerror:")  # CPython accepted the wrong-typed arg; mamba must raise
except TypeError as e:
    print("typeerror:", type(e).__name__)
except Exception as e:
    print("setup_or_other:", type(e).__name__)
