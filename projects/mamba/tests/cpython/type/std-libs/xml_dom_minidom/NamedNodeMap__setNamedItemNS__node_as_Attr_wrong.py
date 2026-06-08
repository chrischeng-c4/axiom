# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "xml_dom_minidom"
# dimension = "type"
# case = "NamedNodeMap__setNamedItemNS__node_as_Attr_wrong"
# subject = "xml.dom.minidom.NamedNodeMap.setNamedItemNS(node: Attr)"
# kind = "semantic"
# xfail = ""
# mem_carveout = ""
# source = "vendor/typeshed/stdlib/xml/dom/minidom.pyi"
# status = "filled"
# ///
# mamba-strict-type: TypeError
"""Type wall: xml.dom.minidom.NamedNodeMap.setNamedItemNS(node: Attr); call it with the wrong type.

typeshed contract: node is Attr. mamba is force-typed, so a wrong-typed
argument MUST raise TypeError (CPython may accept or raise — mamba's to enforce)."""

class _W:
    pass


from xml.dom.minidom import NamedNodeMap
obj = object.__new__(NamedNodeMap)
try:
    obj.setNamedItemNS(_W())  # node: Attr <- wrong-typed
    print("no_typeerror:")  # CPython accepted the wrong-typed arg; mamba must raise
except TypeError as e:
    print("typeerror:", type(e).__name__)
except Exception as e:
    print("setup_or_other:", type(e).__name__)
