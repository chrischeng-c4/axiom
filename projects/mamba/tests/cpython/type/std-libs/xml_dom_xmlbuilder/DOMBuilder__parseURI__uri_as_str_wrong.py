# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "xml_dom_xmlbuilder"
# dimension = "type"
# case = "DOMBuilder__parseURI__uri_as_str_wrong"
# subject = "xml.dom.xmlbuilder.DOMBuilder.parseURI(uri: str)"
# kind = "semantic"
# xfail = ""
# mem_carveout = ""
# source = "vendor/typeshed/stdlib/xml/dom/xmlbuilder.pyi"
# status = "filled"
# ///
# mamba-strict-type: TypeError
"""Type wall: xml.dom.xmlbuilder.DOMBuilder.parseURI(uri: str); call it with the wrong type.

typeshed contract: uri is str. mamba is force-typed, so a wrong-typed
argument MUST raise TypeError (CPython may accept or raise — mamba's to enforce)."""

from xml.dom.xmlbuilder import DOMBuilder
obj = object.__new__(DOMBuilder)
try:
    obj.parseURI(12345)  # uri: str <- wrong-typed
    print("no_typeerror:")  # CPython accepted the wrong-typed arg; mamba must raise
except TypeError as e:
    print("typeerror:", type(e).__name__)
except Exception as e:
    print("setup_or_other:", type(e).__name__)
