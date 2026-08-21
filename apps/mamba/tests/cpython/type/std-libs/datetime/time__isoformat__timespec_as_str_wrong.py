# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "datetime"
# dimension = "type"
# case = "time__isoformat__timespec_as_str_wrong"
# subject = "datetime.time.isoformat(timespec: str)"
# kind = "semantic"
# mem_carveout = ""
# source = "vendor/typeshed/stdlib/datetime.pyi"
# status = "filled"
# ///
# mamba-strict-type: TypeError
"""Type wall: datetime.time.isoformat(timespec: str); call it with the wrong type.

typeshed contract: timespec is str. mamba is force-typed, so a wrong-typed
argument MUST raise TypeError (CPython may accept or raise — mamba's to enforce)."""

from datetime import time
obj = object.__new__(time)
try:
    obj.isoformat(12345)  # timespec: str <- wrong-typed
    print("no_typeerror:")  # CPython accepted the wrong-typed arg; mamba must raise
except TypeError as e:
    print("typeerror:", type(e).__name__)
except Exception as e:
    print("setup_or_other:", type(e).__name__)
