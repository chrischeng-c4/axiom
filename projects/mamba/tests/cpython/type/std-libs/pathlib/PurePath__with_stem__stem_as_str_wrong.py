# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "pathlib"
# dimension = "type"
# case = "PurePath__with_stem__stem_as_str_wrong"
# subject = "pathlib.PurePath.with_stem(stem: str)"
# kind = "semantic"
# xfail = ""
# mem_carveout = ""
# source = "vendor/typeshed/stdlib/pathlib.pyi"
# status = "filled"
# ///
# mamba-strict-type: TypeError
"""Type wall: pathlib.PurePath.with_stem(stem: str); call it with the wrong type.

typeshed contract: stem is str. mamba is force-typed, so a wrong-typed
argument MUST raise TypeError (CPython may accept or raise — mamba's to enforce)."""

from pathlib import PurePath
obj = object.__new__(PurePath)
try:
    obj.with_stem(12345)  # stem: str <- wrong-typed
    print("no_typeerror:")  # CPython accepted the wrong-typed arg; mamba must raise
except TypeError as e:
    print("typeerror:", type(e).__name__)
except Exception as e:
    print("setup_or_other:", type(e).__name__)
