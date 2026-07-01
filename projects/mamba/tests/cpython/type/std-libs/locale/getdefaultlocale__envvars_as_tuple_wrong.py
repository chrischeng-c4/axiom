# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "locale"
# dimension = "type"
# case = "getdefaultlocale__envvars_as_tuple_wrong"
# subject = "locale.getdefaultlocale(envvars: tuple)"
# kind = "semantic"
# xfail = ""
# mem_carveout = ""
# source = "vendor/typeshed/stdlib/locale.pyi"
# status = "filled"
# ///
# mamba-strict-type: TypeError
"""Type wall: locale.getdefaultlocale(envvars: tuple); call it with the wrong type.

typeshed contract: envvars is tuple. mamba is force-typed, so a wrong-typed
argument MUST raise TypeError (CPython may accept or raise — mamba's to enforce)."""

from locale import getdefaultlocale
try:
    getdefaultlocale(12345)  # envvars: tuple <- wrong-typed
    print("no_typeerror:")  # CPython accepted the wrong-typed arg; mamba must raise
except TypeError as e:
    print("typeerror:", type(e).__name__)
except Exception as e:
    print("setup_or_other:", type(e).__name__)
