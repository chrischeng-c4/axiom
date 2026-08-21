# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "wave"
# dimension = "type"
# case = "Wave_write__setframerate__framerate_as_float_wrong"
# subject = "wave.Wave_write.setframerate(framerate: float)"
# kind = "semantic"
# xfail = ""
# mem_carveout = ""
# source = "vendor/typeshed/stdlib/wave.pyi"
# status = "filled"
# ///
# mamba-strict-type: TypeError
"""Type wall: wave.Wave_write.setframerate(framerate: float); call it with the wrong type.

typeshed contract: framerate is float. mamba is force-typed, so a wrong-typed
argument MUST raise TypeError (CPython may accept or raise — mamba's to enforce)."""

from wave import Wave_write
obj = object.__new__(Wave_write)
try:
    obj.setframerate("not_a_float")  # framerate: float <- wrong-typed
    print("no_typeerror:")  # CPython accepted the wrong-typed arg; mamba must raise
except TypeError as e:
    print("typeerror:", type(e).__name__)
except Exception as e:
    print("setup_or_other:", type(e).__name__)
