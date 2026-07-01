# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "zoneinfo"
# dimension = "type"
# case = "ZoneInfo____new____key_as_str_wrong"
# subject = "zoneinfo.ZoneInfo.__new__(key: str)"
# kind = "semantic"
# mem_carveout = ""
# source = "vendor/typeshed/stdlib/zoneinfo.pyi"
# status = "filled"
# ///
# mamba-strict-type: TypeError
"""Type wall: zoneinfo.ZoneInfo.__new__(key: str); call it with the wrong type.

typeshed contract: key is str. mamba is force-typed, so a wrong-typed
argument MUST raise TypeError (CPython may accept or raise — mamba's to enforce)."""

from zoneinfo import ZoneInfo
obj = object.__new__(ZoneInfo)
try:
    obj.__new__(12345)  # key: str <- wrong-typed
    print("no_typeerror:")  # CPython accepted the wrong-typed arg; mamba must raise
except TypeError as e:
    print("typeerror:", type(e).__name__)
except Exception as e:
    print("setup_or_other:", type(e).__name__)
