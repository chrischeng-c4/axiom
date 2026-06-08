# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "ossaudiodev"
# dimension = "type"
# case = "openmixer__device_as_str_wrong"
# subject = "ossaudiodev.openmixer(device: str)"
# kind = "semantic"
# xfail = ""
# mem_carveout = ""
# source = "vendor/typeshed/stdlib/ossaudiodev.pyi"
# status = "filled"
# ///
# mamba-strict-type: TypeError
"""Type wall: ossaudiodev.openmixer(device: str); call it with the wrong type.

typeshed contract: device is str. mamba is force-typed, so a wrong-typed
argument MUST raise TypeError (CPython may accept or raise — mamba's to enforce)."""

from ossaudiodev import openmixer
try:
    openmixer(12345)  # device: str <- wrong-typed
    print("no_typeerror:")  # CPython accepted the wrong-typed arg; mamba must raise
except TypeError as e:
    print("typeerror:", type(e).__name__)
except Exception as e:
    print("setup_or_other:", type(e).__name__)
