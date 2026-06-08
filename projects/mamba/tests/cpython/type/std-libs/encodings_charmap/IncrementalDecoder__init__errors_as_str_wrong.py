# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "encodings_charmap"
# dimension = "type"
# case = "IncrementalDecoder__init__errors_as_str_wrong"
# subject = "encodings.charmap.IncrementalDecoder.__init__(errors: str)"
# kind = "semantic"
# xfail = ""
# mem_carveout = ""
# source = "vendor/typeshed/stdlib/encodings/charmap.pyi"
# status = "filled"
# ///
# mamba-strict-type: TypeError
"""Type wall: encodings.charmap.IncrementalDecoder.__init__(errors: str); call it with the wrong type.

typeshed contract: errors is str. mamba is force-typed, so a wrong-typed
argument MUST raise TypeError (CPython may accept or raise — mamba's to enforce)."""

from encodings.charmap import IncrementalDecoder
try:
    IncrementalDecoder(12345)  # errors: str <- wrong-typed
    print("no_typeerror:")  # CPython accepted the wrong-typed arg; mamba must raise
except TypeError as e:
    print("typeerror:", type(e).__name__)
except Exception as e:
    print("setup_or_other:", type(e).__name__)
