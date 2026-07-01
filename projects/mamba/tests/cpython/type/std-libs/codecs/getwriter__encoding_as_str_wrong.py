# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "codecs"
# dimension = "type"
# case = "getwriter__encoding_as_str_wrong"
# subject = "codecs.getwriter(encoding: str)"
# kind = "semantic"
# mem_carveout = ""
# source = "vendor/typeshed/stdlib/codecs.pyi"
# status = "filled"
# ///
# mamba-strict-type: TypeError
"""Type wall: codecs.getwriter(encoding: str); call it with the wrong type.

typeshed contract: encoding is str. mamba is force-typed, so a wrong-typed
argument MUST raise TypeError (CPython may accept or raise — mamba's to enforce)."""

from codecs import getwriter
try:
    getwriter(12345)  # encoding: str <- wrong-typed
    print("no_typeerror:")  # CPython accepted the wrong-typed arg; mamba must raise
except TypeError as e:
    print("typeerror:", type(e).__name__)
except Exception as e:
    print("setup_or_other:", type(e).__name__)
