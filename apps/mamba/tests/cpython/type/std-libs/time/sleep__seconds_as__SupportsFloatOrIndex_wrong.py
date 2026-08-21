# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "time"
# dimension = "type"
# case = "sleep__seconds_as__SupportsFloatOrIndex_wrong"
# subject = "time.sleep(seconds: _SupportsFloatOrIndex)"
# kind = "semantic"
# xfail = ""
# mem_carveout = ""
# source = "vendor/typeshed/stdlib/time.pyi"
# status = "filled"
# ///
# mamba-strict-type: TypeError
"""Type wall: time.sleep(seconds: _SupportsFloatOrIndex); call it with the wrong type.

typeshed contract: seconds is _SupportsFloatOrIndex. mamba is force-typed, so a wrong-typed
argument MUST raise TypeError (CPython may accept or raise — mamba's to enforce)."""

class _W:
    pass


from time import sleep
try:
    sleep(_W())  # seconds: _SupportsFloatOrIndex <- wrong-typed
    print("no_typeerror:")  # CPython accepted the wrong-typed arg; mamba must raise
except TypeError as e:
    print("typeerror:", type(e).__name__)
except Exception as e:
    print("setup_or_other:", type(e).__name__)
