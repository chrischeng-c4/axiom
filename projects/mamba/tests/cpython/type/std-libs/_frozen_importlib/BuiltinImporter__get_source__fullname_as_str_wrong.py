# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "_frozen_importlib"
# dimension = "type"
# case = "BuiltinImporter__get_source__fullname_as_str_wrong"
# subject = "_frozen_importlib.BuiltinImporter.get_source(fullname: str)"
# kind = "semantic"
# xfail = ""
# mem_carveout = ""
# source = "vendor/typeshed/stdlib/_frozen_importlib.pyi"
# status = "filled"
# ///
# mamba-strict-type: TypeError
"""Type wall: _frozen_importlib.BuiltinImporter.get_source(fullname: str); call it with the wrong type.

typeshed contract: fullname is str. mamba is force-typed, so a wrong-typed
argument MUST raise TypeError (CPython may accept or raise — mamba's to enforce)."""

from _frozen_importlib import BuiltinImporter
try:
    BuiltinImporter.get_source(12345)  # fullname: str <- wrong-typed
    print("no_typeerror:")  # CPython accepted the wrong-typed arg; mamba must raise
except TypeError as e:
    print("typeerror:", type(e).__name__)
except Exception as e:
    print("setup_or_other:", type(e).__name__)
