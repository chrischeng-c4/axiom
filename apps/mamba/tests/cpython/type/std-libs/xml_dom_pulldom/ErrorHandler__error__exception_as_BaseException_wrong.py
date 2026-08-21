# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "xml_dom_pulldom"
# dimension = "type"
# case = "ErrorHandler__error__exception_as_BaseException_wrong"
# subject = "xml.dom.pulldom.ErrorHandler.error(exception: BaseException)"
# kind = "semantic"
# xfail = ""
# mem_carveout = ""
# source = "vendor/typeshed/stdlib/xml/dom/pulldom.pyi"
# status = "filled"
# ///
# mamba-strict-type: TypeError
"""Type wall: xml.dom.pulldom.ErrorHandler.error(exception: BaseException); call it with the wrong type.

typeshed contract: exception is BaseException. mamba is force-typed, so a wrong-typed
argument MUST raise TypeError (CPython may accept or raise — mamba's to enforce)."""

class _W:
    pass


from xml.dom.pulldom import ErrorHandler
obj = object.__new__(ErrorHandler)
try:
    obj.error(_W())  # exception: BaseException <- wrong-typed
    print("no_typeerror:")  # CPython accepted the wrong-typed arg; mamba must raise
except TypeError as e:
    print("typeerror:", type(e).__name__)
except Exception as e:
    print("setup_or_other:", type(e).__name__)
