# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "configparser"
# dimension = "type"
# case = "SectionProxy__get__option_as_str_wrong"
# subject = "configparser.SectionProxy.get(option: str)"
# kind = "semantic"
# xfail = ""
# mem_carveout = ""
# source = "vendor/typeshed/stdlib/configparser.pyi"
# status = "filled"
# ///
# mamba-strict-type: TypeError
"""Type wall: configparser.SectionProxy.get(option: str); call it with the wrong type.

typeshed contract: option is str. mamba is force-typed, so a wrong-typed
argument MUST raise TypeError (CPython may accept or raise — mamba's to enforce)."""

from configparser import SectionProxy
obj = object.__new__(SectionProxy)
try:
    obj.get(12345)  # option: str <- wrong-typed
    print("no_typeerror:")  # CPython accepted the wrong-typed arg; mamba must raise
except TypeError as e:
    print("typeerror:", type(e).__name__)
except Exception as e:
    print("setup_or_other:", type(e).__name__)
