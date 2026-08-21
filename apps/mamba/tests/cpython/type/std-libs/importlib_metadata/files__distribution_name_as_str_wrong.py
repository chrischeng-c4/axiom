# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "importlib_metadata"
# dimension = "type"
# case = "files__distribution_name_as_str_wrong"
# subject = "importlib.metadata.files(distribution_name: str)"
# kind = "semantic"
# xfail = ""
# mem_carveout = ""
# source = "vendor/typeshed/stdlib/importlib/metadata.pyi"
# status = "filled"
# ///
# mamba-strict-type: TypeError
"""Type wall: importlib.metadata.files(distribution_name: str); call it with the wrong type.

typeshed contract: distribution_name is str. mamba is force-typed, so a wrong-typed
argument MUST raise TypeError (CPython may accept or raise — mamba's to enforce)."""

from importlib.metadata import files
try:
    files(12345)  # distribution_name: str <- wrong-typed
    print("no_typeerror:")  # CPython accepted the wrong-typed arg; mamba must raise
except TypeError as e:
    print("typeerror:", type(e).__name__)
except Exception as e:
    print("setup_or_other:", type(e).__name__)
