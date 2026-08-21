# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "wsgiref_validate"
# dimension = "type"
# case = "IteratorWrapper__init__wsgi_iterator_as_Iterator_wrong"
# subject = "wsgiref.validate.IteratorWrapper.__init__(wsgi_iterator: Iterator)"
# kind = "semantic"
# xfail = ""
# mem_carveout = ""
# source = "vendor/typeshed/stdlib/wsgiref/validate.pyi"
# status = "filled"
# ///
# mamba-strict-type: TypeError
"""Type wall: wsgiref.validate.IteratorWrapper.__init__(wsgi_iterator: Iterator); call it with the wrong type.

typeshed contract: wsgi_iterator is Iterator. mamba is force-typed, so a wrong-typed
argument MUST raise TypeError (CPython may accept or raise — mamba's to enforce)."""

class _W:
    pass


from wsgiref.validate import IteratorWrapper
try:
    IteratorWrapper(_W(), None)  # wsgi_iterator: Iterator <- wrong-typed
    print("no_typeerror:")  # CPython accepted the wrong-typed arg; mamba must raise
except TypeError as e:
    print("typeerror:", type(e).__name__)
except Exception as e:
    print("setup_or_other:", type(e).__name__)
