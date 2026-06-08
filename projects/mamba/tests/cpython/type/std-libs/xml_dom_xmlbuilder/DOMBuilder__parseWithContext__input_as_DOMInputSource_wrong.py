# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "xml_dom_xmlbuilder"
# dimension = "type"
# case = "DOMBuilder__parseWithContext__input_as_DOMInputSource_wrong"
# subject = "xml.dom.xmlbuilder.DOMBuilder.parseWithContext(input: DOMInputSource)"
# kind = "semantic"
# xfail = ""
# mem_carveout = ""
# source = "vendor/typeshed/stdlib/xml/dom/xmlbuilder.pyi"
# status = "filled"
# ///
# mamba-strict-type: TypeError
"""Type wall: xml.dom.xmlbuilder.DOMBuilder.parseWithContext(input: DOMInputSource); call it with the wrong type.

typeshed contract: input is DOMInputSource. mamba is force-typed, so a wrong-typed
argument MUST raise TypeError (CPython may accept or raise — mamba's to enforce)."""

class _W:
    pass


from xml.dom.xmlbuilder import DOMBuilder
obj = object.__new__(DOMBuilder)
try:
    obj.parseWithContext(_W(), None, None)  # input: DOMInputSource <- wrong-typed
    print("no_typeerror:")  # CPython accepted the wrong-typed arg; mamba must raise
except TypeError as e:
    print("typeerror:", type(e).__name__)
except Exception as e:
    print("setup_or_other:", type(e).__name__)
