# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "cgi"
# dimension = "type"
# case = "test__environ_as__Environ_wrong"
# subject = "cgi.test(environ: _Environ)"
# kind = "semantic"
# xfail = ""
# mem_carveout = ""
# source = "vendor/typeshed/stdlib/cgi.pyi"
# status = "filled"
# ///
# mamba-strict-type: TypeError
"""Type wall: cgi.test(environ: _Environ); call it with the wrong type.

typeshed contract: environ is _Environ. mamba is force-typed, so a wrong-typed
argument MUST raise TypeError (CPython may accept or raise — mamba's to enforce)."""

class _W:
    pass


from cgi import test
try:
    test(_W())  # environ: _Environ <- wrong-typed
    print("no_typeerror:")  # CPython accepted the wrong-typed arg; mamba must raise
except TypeError as e:
    print("typeerror:", type(e).__name__)
except Exception as e:
    print("setup_or_other:", type(e).__name__)
