# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "tempfile"
# dimension = "type"
# case = "SpooledTemporaryFile__init__max_size_as_int_wrong"
# subject = "tempfile.SpooledTemporaryFile.__init__(max_size: int)"
# kind = "semantic"
# xfail = ""
# mem_carveout = ""
# source = "vendor/typeshed/stdlib/tempfile.pyi"
# status = "filled"
# ///
# mamba-strict-type: TypeError
"""Type wall: tempfile.SpooledTemporaryFile.__init__(max_size: int); call it with the wrong type.

typeshed contract: max_size is int. mamba is force-typed, so a wrong-typed
argument MUST raise TypeError (CPython may accept or raise — mamba's to enforce)."""

from tempfile import SpooledTemporaryFile
try:
    SpooledTemporaryFile("not_an_int")  # max_size: int <- wrong-typed
    print("no_typeerror:")  # CPython accepted the wrong-typed arg; mamba must raise
except TypeError as e:
    print("typeerror:", type(e).__name__)
except Exception as e:
    print("setup_or_other:", type(e).__name__)
