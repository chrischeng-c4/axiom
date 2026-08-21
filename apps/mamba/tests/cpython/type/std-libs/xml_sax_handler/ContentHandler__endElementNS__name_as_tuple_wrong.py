# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "xml_sax_handler"
# dimension = "type"
# case = "ContentHandler__endElementNS__name_as_tuple_wrong"
# subject = "xml.sax.handler.ContentHandler.endElementNS(name: tuple)"
# kind = "semantic"
# xfail = "force-typed arg enforcement pending; mamba must raise TypeError on wrong-typed name"
# mem_carveout = ""
# source = "vendor/typeshed/stdlib/xml/sax/handler.pyi"
# status = "filled"
# ///
# mamba-xfail: force-typed arg enforcement pending; mamba must raise TypeError on wrong-typed name
# mamba-strict-type: TypeError
"""Type wall: xml.sax.handler.ContentHandler.endElementNS(name: tuple); call it with the wrong type.

typeshed contract: name is tuple. mamba is force-typed, so a wrong-typed
argument MUST raise TypeError (CPython may accept or raise — mamba's to enforce)."""

from xml.sax.handler import ContentHandler
obj = object.__new__(ContentHandler)
try:
    obj.endElementNS(12345, None)  # name: tuple <- wrong-typed
    print("no_typeerror:")  # CPython accepted the wrong-typed arg; mamba must raise
except TypeError as e:
    print("typeerror:", type(e).__name__)
except Exception as e:
    print("setup_or_other:", type(e).__name__)
