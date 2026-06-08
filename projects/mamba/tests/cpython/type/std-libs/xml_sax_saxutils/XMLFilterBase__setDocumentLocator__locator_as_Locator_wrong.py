# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "xml_sax_saxutils"
# dimension = "type"
# case = "XMLFilterBase__setDocumentLocator__locator_as_Locator_wrong"
# subject = "xml.sax.saxutils.XMLFilterBase.setDocumentLocator(locator: Locator)"
# kind = "semantic"
# xfail = ""
# mem_carveout = ""
# source = "vendor/typeshed/stdlib/xml/sax/saxutils.pyi"
# status = "filled"
# ///
# mamba-strict-type: TypeError
"""Type wall: xml.sax.saxutils.XMLFilterBase.setDocumentLocator(locator: Locator); call it with the wrong type.

typeshed contract: locator is Locator. mamba is force-typed, so a wrong-typed
argument MUST raise TypeError (CPython may accept or raise — mamba's to enforce)."""

class _W:
    pass


from xml.sax.saxutils import XMLFilterBase
obj = object.__new__(XMLFilterBase)
try:
    obj.setDocumentLocator(_W())  # locator: Locator <- wrong-typed
    print("no_typeerror:")  # CPython accepted the wrong-typed arg; mamba must raise
except TypeError as e:
    print("typeerror:", type(e).__name__)
except Exception as e:
    print("setup_or_other:", type(e).__name__)
