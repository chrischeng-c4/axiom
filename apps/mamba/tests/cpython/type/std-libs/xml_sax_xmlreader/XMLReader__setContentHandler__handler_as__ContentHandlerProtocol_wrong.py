# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "xml_sax_xmlreader"
# dimension = "type"
# case = "XMLReader__setContentHandler__handler_as__ContentHandlerProtocol_wrong"
# subject = "xml.sax.xmlreader.XMLReader.setContentHandler(handler: _ContentHandlerProtocol)"
# kind = "semantic"
# xfail = ""
# mem_carveout = ""
# source = "vendor/typeshed/stdlib/xml/sax/xmlreader.pyi"
# status = "filled"
# ///
# mamba-strict-type: TypeError
"""Type wall: xml.sax.xmlreader.XMLReader.setContentHandler(handler: _ContentHandlerProtocol); call it with the wrong type.

typeshed contract: handler is _ContentHandlerProtocol. mamba is force-typed, so a wrong-typed
argument MUST raise TypeError (CPython may accept or raise — mamba's to enforce)."""

class _W:
    pass


from xml.sax.xmlreader import XMLReader
obj = object.__new__(XMLReader)
try:
    obj.setContentHandler(_W())  # handler: _ContentHandlerProtocol <- wrong-typed
    print("no_typeerror:")  # CPython accepted the wrong-typed arg; mamba must raise
except TypeError as e:
    print("typeerror:", type(e).__name__)
except Exception as e:
    print("setup_or_other:", type(e).__name__)
