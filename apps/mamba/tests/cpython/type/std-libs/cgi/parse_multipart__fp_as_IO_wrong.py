# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "cgi"
# dimension = "type"
# case = "parse_multipart__fp_as_IO_wrong"
# subject = "cgi.parse_multipart(fp: IO)"
# kind = "semantic"
# xfail = ""
# mem_carveout = ""
# source = "vendor/typeshed/stdlib/cgi.pyi"
# status = "filled"
# ///
# mamba-strict-type: TypeError
"""Type wall: cgi.parse_multipart(fp: IO); call it with the wrong type.

typeshed contract: fp is IO. mamba is force-typed, so a wrong-typed
argument MUST raise TypeError (CPython may accept or raise — mamba's to enforce)."""

class _W:
    pass


from cgi import parse_multipart
try:
    parse_multipart(_W(), None)  # fp: IO <- wrong-typed
    print("no_typeerror:")  # CPython accepted the wrong-typed arg; mamba must raise
except TypeError as e:
    print("typeerror:", type(e).__name__)
except Exception as e:
    print("setup_or_other:", type(e).__name__)
