# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "xml_dom_minidom"
# dimension = "type"
# case = "ReadOnlySequentialNamedNodeMap__getNamedItemNS__namespaceURI_as_typed_wrong"
# subject = "xml.dom.minidom.ReadOnlySequentialNamedNodeMap.getNamedItemNS(namespaceURI: typed)"
# kind = "semantic"
# xfail = ""
# mem_carveout = ""
# source = "vendor/typeshed/stdlib/xml/dom/minidom.pyi"
# status = "filled"
# ///
# mamba-strict-type: TypeError
"""Type wall: xml.dom.minidom.ReadOnlySequentialNamedNodeMap.getNamedItemNS(namespaceURI: typed); call it with the wrong type.

typeshed contract: namespaceURI is typed. mamba is force-typed, so a wrong-typed
argument MUST raise TypeError (CPython may accept or raise — mamba's to enforce)."""

class _W:
    pass


from xml.dom.minidom import ReadOnlySequentialNamedNodeMap
obj = object.__new__(ReadOnlySequentialNamedNodeMap)
try:
    obj.getNamedItemNS(_W(), "")  # namespaceURI: typed <- wrong-typed
    print("no_typeerror:")  # CPython accepted the wrong-typed arg; mamba must raise
except TypeError as e:
    print("typeerror:", type(e).__name__)
except Exception as e:
    print("setup_or_other:", type(e).__name__)
