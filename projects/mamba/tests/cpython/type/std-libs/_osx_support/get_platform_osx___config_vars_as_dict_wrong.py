# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "_osx_support"
# dimension = "type"
# case = "get_platform_osx___config_vars_as_dict_wrong"
# subject = "_osx_support.get_platform_osx(_config_vars: dict)"
# kind = "semantic"
# xfail = ""
# mem_carveout = ""
# source = "vendor/typeshed/stdlib/_osx_support.pyi"
# status = "filled"
# ///
# mamba-strict-type: TypeError
"""Type wall: _osx_support.get_platform_osx(_config_vars: dict); call it with the wrong type.

typeshed contract: _config_vars is dict. mamba is force-typed, so a wrong-typed
argument MUST raise TypeError (CPython may accept or raise — mamba's to enforce)."""

from _osx_support import get_platform_osx
try:
    get_platform_osx(12345, None, None, None)  # _config_vars: dict <- wrong-typed
    print("no_typeerror:")  # CPython accepted the wrong-typed arg; mamba must raise
except TypeError as e:
    print("typeerror:", type(e).__name__)
except Exception as e:
    print("setup_or_other:", type(e).__name__)
