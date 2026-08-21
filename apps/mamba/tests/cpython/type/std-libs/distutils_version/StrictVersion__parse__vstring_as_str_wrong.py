# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "distutils_version"
# dimension = "type"
# case = "StrictVersion__parse__vstring_as_str_wrong"
# subject = "distutils.version.StrictVersion.parse(vstring: str)"
# kind = "semantic"
# xfail = ""
# mem_carveout = ""
# source = "vendor/typeshed/stdlib/distutils/version.pyi"
# status = "filled"
# ///
# mamba-strict-type: TypeError
"""Type wall: distutils.version.StrictVersion.parse(vstring: str); call it with the wrong type.

typeshed contract: vstring is str. mamba is force-typed, so a wrong-typed
argument MUST raise TypeError (CPython may accept or raise — mamba's to enforce)."""

from distutils.version import StrictVersion
obj = object.__new__(StrictVersion)
try:
    obj.parse(12345)  # vstring: str <- wrong-typed
    print("no_typeerror:")  # CPython accepted the wrong-typed arg; mamba must raise
except TypeError as e:
    print("typeerror:", type(e).__name__)
except Exception as e:
    print("setup_or_other:", type(e).__name__)
