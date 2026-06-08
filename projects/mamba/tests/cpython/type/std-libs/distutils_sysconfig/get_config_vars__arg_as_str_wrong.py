# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "distutils_sysconfig"
# dimension = "type"
# case = "get_config_vars__arg_as_str_wrong"
# subject = "distutils.sysconfig.get_config_vars(arg: str)"
# kind = "semantic"
# xfail = "force-typed arg enforcement pending; mamba must raise TypeError on wrong-typed arg"
# mem_carveout = ""
# source = "vendor/typeshed/stdlib/distutils/sysconfig.pyi"
# status = "filled"
# ///
# mamba-xfail: force-typed arg enforcement pending; mamba must raise TypeError on wrong-typed arg
# mamba-strict-type: TypeError
"""Type wall: distutils.sysconfig.get_config_vars(arg: str); call it with the wrong type.

typeshed contract: arg is str. mamba is force-typed, so a wrong-typed
argument MUST raise TypeError (CPython may accept or raise — mamba's to enforce)."""

from distutils.sysconfig import get_config_vars
try:
    get_config_vars(12345)  # arg: str <- wrong-typed
    print("no_typeerror:")  # CPython accepted the wrong-typed arg; mamba must raise
except TypeError as e:
    print("typeerror:", type(e).__name__)
except Exception as e:
    print("setup_or_other:", type(e).__name__)
