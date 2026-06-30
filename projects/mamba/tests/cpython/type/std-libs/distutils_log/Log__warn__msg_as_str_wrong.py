# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "distutils_log"
# dimension = "type"
# case = "Log__warn__msg_as_str_wrong"
# subject = "distutils.log.Log.warn(msg: str)"
# kind = "semantic"
# xfail = ""
# mem_carveout = ""
# source = "vendor/typeshed/stdlib/distutils/log.pyi"
# status = "filled"
# ///
# mamba-strict-type: TypeError
"""Type wall: distutils.log.Log.warn(msg: str); call it with the wrong type.

typeshed contract: msg is str. mamba is force-typed, so a wrong-typed
argument MUST raise TypeError (CPython may accept or raise — mamba's to enforce)."""

from distutils.log import Log
obj = object.__new__(Log)
try:
    obj.warn(12345)  # msg: str <- wrong-typed
    print("no_typeerror:")  # CPython accepted the wrong-typed arg; mamba must raise
except TypeError as e:
    print("typeerror:", type(e).__name__)
except Exception as e:
    print("setup_or_other:", type(e).__name__)
