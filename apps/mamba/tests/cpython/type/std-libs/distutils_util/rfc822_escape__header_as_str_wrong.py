# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "distutils_util"
# dimension = "type"
# case = "rfc822_escape__header_as_str_wrong"
# subject = "distutils.util.rfc822_escape(header: str)"
# kind = "semantic"
# xfail = ""
# mem_carveout = ""
# source = "vendor/typeshed/stdlib/distutils/util.pyi"
# status = "filled"
# ///
# mamba-strict-type: TypeError
"""Type wall: distutils.util.rfc822_escape(header: str); call it with the wrong type.

typeshed contract: header is str. mamba is force-typed, so a wrong-typed
argument MUST raise TypeError (CPython may accept or raise — mamba's to enforce)."""

from distutils.util import rfc822_escape
try:
    rfc822_escape(12345)  # header: str <- wrong-typed
    print("no_typeerror:")  # CPython accepted the wrong-typed arg; mamba must raise
except TypeError as e:
    print("typeerror:", type(e).__name__)
except Exception as e:
    print("setup_or_other:", type(e).__name__)
