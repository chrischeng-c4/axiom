# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "wsgiref_validate"
# dimension = "type"
# case = "ErrorWrapper__init__wsgi_errors_as_ErrorStream_wrong"
# subject = "wsgiref.validate.ErrorWrapper.__init__(wsgi_errors: ErrorStream)"
# kind = "semantic"
# xfail = ""
# mem_carveout = ""
# source = "vendor/typeshed/stdlib/wsgiref/validate.pyi"
# status = "filled"
# ///
# mamba-strict-type: TypeError
"""Type wall: wsgiref.validate.ErrorWrapper.__init__(wsgi_errors: ErrorStream); call it with the wrong type.

typeshed contract: wsgi_errors is ErrorStream. mamba is force-typed, so a wrong-typed
argument MUST raise TypeError (CPython may accept or raise — mamba's to enforce)."""

class _W:
    pass


from wsgiref.validate import ErrorWrapper
try:
    ErrorWrapper(_W())  # wsgi_errors: ErrorStream <- wrong-typed
    print("no_typeerror:")  # CPython accepted the wrong-typed arg; mamba must raise
except TypeError as e:
    print("typeerror:", type(e).__name__)
except Exception as e:
    print("setup_or_other:", type(e).__name__)
