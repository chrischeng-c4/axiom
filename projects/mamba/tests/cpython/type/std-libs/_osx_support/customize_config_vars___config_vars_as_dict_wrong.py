# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "_osx_support"
# dimension = "type"
# case = "customize_config_vars___config_vars_as_dict_wrong"
# subject = "_osx_support.customize_config_vars(_config_vars: dict)"
# kind = "semantic"
# mem_carveout = ""
# source = "vendor/typeshed/stdlib/_osx_support.pyi"
# status = "filled"
# ///
# mamba-strict-type: TypeError
"""Type wall: _osx_support.customize_config_vars(_config_vars: dict); call it with the wrong type.

typeshed contract: _config_vars is dict. mamba is force-typed, so a wrong-typed
argument MUST raise TypeError (CPython may accept or raise — mamba's to enforce)."""

from _osx_support import customize_config_vars
try:
    customize_config_vars(12345)  # _config_vars: dict <- wrong-typed
    print("no_typeerror:")  # CPython accepted the wrong-typed arg; mamba must raise
except TypeError as e:
    print("typeerror:", type(e).__name__)
except Exception as e:
    print("setup_or_other:", type(e).__name__)
