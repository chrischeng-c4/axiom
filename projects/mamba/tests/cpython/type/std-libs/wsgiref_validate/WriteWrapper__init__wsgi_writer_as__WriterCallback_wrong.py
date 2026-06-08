# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "wsgiref_validate"
# dimension = "type"
# case = "WriteWrapper__init__wsgi_writer_as__WriterCallback_wrong"
# subject = "wsgiref.validate.WriteWrapper.__init__(wsgi_writer: _WriterCallback)"
# kind = "semantic"
# xfail = ""
# mem_carveout = ""
# source = "vendor/typeshed/stdlib/wsgiref/validate.pyi"
# status = "filled"
# ///
# mamba-strict-type: TypeError
"""Type wall: wsgiref.validate.WriteWrapper.__init__(wsgi_writer: _WriterCallback); call it with the wrong type.

typeshed contract: wsgi_writer is _WriterCallback. mamba is force-typed, so a wrong-typed
argument MUST raise TypeError (CPython may accept or raise — mamba's to enforce)."""

class _W:
    pass


from wsgiref.validate import WriteWrapper
try:
    WriteWrapper(_W())  # wsgi_writer: _WriterCallback <- wrong-typed
    print("no_typeerror:")  # CPython accepted the wrong-typed arg; mamba must raise
except TypeError as e:
    print("typeerror:", type(e).__name__)
except Exception as e:
    print("setup_or_other:", type(e).__name__)
