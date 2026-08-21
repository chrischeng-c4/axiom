# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "wsgiref_util"
# dimension = "type"
# case = "shift_path_info__environ_as_WSGIEnvironment_wrong"
# subject = "wsgiref.util.shift_path_info(environ: WSGIEnvironment)"
# kind = "semantic"
# xfail = ""
# mem_carveout = ""
# source = "vendor/typeshed/stdlib/wsgiref/util.pyi"
# status = "filled"
# ///
# mamba-strict-type: TypeError
"""Type wall: wsgiref.util.shift_path_info(environ: WSGIEnvironment); call it with the wrong type.

typeshed contract: environ is WSGIEnvironment. mamba is force-typed, so a wrong-typed
argument MUST raise TypeError (CPython may accept or raise — mamba's to enforce)."""

class _W:
    pass


from wsgiref.util import shift_path_info
try:
    shift_path_info(_W())  # environ: WSGIEnvironment <- wrong-typed
    print("no_typeerror:")  # CPython accepted the wrong-typed arg; mamba must raise
except TypeError as e:
    print("typeerror:", type(e).__name__)
except Exception as e:
    print("setup_or_other:", type(e).__name__)
