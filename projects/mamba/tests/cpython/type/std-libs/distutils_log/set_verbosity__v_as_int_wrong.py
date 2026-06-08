# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "distutils_log"
# dimension = "type"
# case = "set_verbosity__v_as_int_wrong"
# subject = "distutils.log.set_verbosity(v: int)"
# kind = "semantic"
# xfail = ""
# mem_carveout = ""
# source = "vendor/typeshed/stdlib/distutils/log.pyi"
# status = "filled"
# ///
# mamba-strict-type: TypeError
"""Type wall: distutils.log.set_verbosity(v: int); call it with the wrong type.

typeshed contract: v is int. mamba is force-typed, so a wrong-typed
argument MUST raise TypeError (CPython may accept or raise — mamba's to enforce)."""

from distutils.log import set_verbosity
try:
    set_verbosity("not_an_int")  # v: int <- wrong-typed
    print("no_typeerror:")  # CPython accepted the wrong-typed arg; mamba must raise
except TypeError as e:
    print("typeerror:", type(e).__name__)
except Exception as e:
    print("setup_or_other:", type(e).__name__)
