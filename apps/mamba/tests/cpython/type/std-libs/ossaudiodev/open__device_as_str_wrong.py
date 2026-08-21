# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "ossaudiodev"
# dimension = "type"
# case = "open__device_as_str_wrong"
# subject = "ossaudiodev.open(device: str)"
# kind = "semantic"
# xfail = "force-typed arg enforcement pending; mamba must raise TypeError on wrong-typed device"
# mem_carveout = ""
# source = "vendor/typeshed/stdlib/ossaudiodev.pyi"
# status = "filled"
# ///
# mamba-xfail: force-typed arg enforcement pending; mamba must raise TypeError on wrong-typed device
# mamba-strict-type: TypeError
"""Type wall: ossaudiodev.open(device: str); call it with the wrong type.

typeshed contract: device is str. mamba is force-typed, so a wrong-typed
argument MUST raise TypeError (CPython may accept or raise — mamba's to enforce)."""

from ossaudiodev import open
try:
    open(12345, None)  # device: str <- wrong-typed
    print("no_typeerror:")  # CPython accepted the wrong-typed arg; mamba must raise
except TypeError as e:
    print("typeerror:", type(e).__name__)
except Exception as e:
    print("setup_or_other:", type(e).__name__)
