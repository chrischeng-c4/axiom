# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "xml_sax_saxutils"
# dimension = "type"
# case = "XMLFilterBase__warning__exception_as_BaseException_wrong"
# subject = "xml.sax.saxutils.XMLFilterBase.warning(exception: BaseException)"
# kind = "semantic"
# xfail = ""
# mem_carveout = ""
# source = "vendor/typeshed/stdlib/xml/sax/saxutils.pyi"
# status = "filled"
# ///
# mamba-strict-type: TypeError
"""Type wall: xml.sax.saxutils.XMLFilterBase.warning(exception: BaseException); call it with the wrong type.

typeshed contract: exception is BaseException. mamba is force-typed, so a wrong-typed
argument MUST raise TypeError (CPython may accept or raise — mamba's to enforce)."""

class _W:
    pass


from xml.sax.saxutils import XMLFilterBase
obj = object.__new__(XMLFilterBase)
try:
    obj.warning(_W())  # exception: BaseException <- wrong-typed
    print("no_typeerror:")  # CPython accepted the wrong-typed arg; mamba must raise
except TypeError as e:
    print("typeerror:", type(e).__name__)
except Exception as e:
    print("setup_or_other:", type(e).__name__)
