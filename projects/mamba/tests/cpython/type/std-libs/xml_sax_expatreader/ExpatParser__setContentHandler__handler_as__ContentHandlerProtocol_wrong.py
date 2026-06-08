# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "xml_sax_expatreader"
# dimension = "type"
# case = "ExpatParser__setContentHandler__handler_as__ContentHandlerProtocol_wrong"
# subject = "xml.sax.expatreader.ExpatParser.setContentHandler(handler: _ContentHandlerProtocol)"
# kind = "semantic"
# xfail = ""
# mem_carveout = ""
# source = "vendor/typeshed/stdlib/xml/sax/expatreader.pyi"
# status = "filled"
# ///
# mamba-strict-type: TypeError
"""Type wall: xml.sax.expatreader.ExpatParser.setContentHandler(handler: _ContentHandlerProtocol); call it with the wrong type.

typeshed contract: handler is _ContentHandlerProtocol. mamba is force-typed, so a wrong-typed
argument MUST raise TypeError (CPython may accept or raise — mamba's to enforce)."""

class _W:
    pass


from xml.sax.expatreader import ExpatParser
obj = object.__new__(ExpatParser)
try:
    obj.setContentHandler(_W())  # handler: _ContentHandlerProtocol <- wrong-typed
    print("no_typeerror:")  # CPython accepted the wrong-typed arg; mamba must raise
except TypeError as e:
    print("typeerror:", type(e).__name__)
except Exception as e:
    print("setup_or_other:", type(e).__name__)
