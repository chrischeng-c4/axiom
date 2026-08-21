# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "importlib_metadata__meta"
# dimension = "type"
# case = "PackageMetadata__get_all__name_as_str_wrong"
# subject = "importlib.metadata._meta.PackageMetadata.get_all(name: str)"
# kind = "semantic"
# xfail = ""
# mem_carveout = ""
# source = "vendor/typeshed/stdlib/importlib/metadata/_meta.pyi"
# status = "filled"
# ///
# mamba-strict-type: TypeError
"""Type wall: importlib.metadata._meta.PackageMetadata.get_all(name: str); call it with the wrong type.

typeshed contract: name is str. mamba is force-typed, so a wrong-typed
argument MUST raise TypeError (CPython may accept or raise — mamba's to enforce)."""

from importlib.metadata._meta import PackageMetadata
obj = object.__new__(PackageMetadata)
try:
    obj.get_all(12345)  # name: str <- wrong-typed
    print("no_typeerror:")  # CPython accepted the wrong-typed arg; mamba must raise
except TypeError as e:
    print("typeerror:", type(e).__name__)
except Exception as e:
    print("setup_or_other:", type(e).__name__)
