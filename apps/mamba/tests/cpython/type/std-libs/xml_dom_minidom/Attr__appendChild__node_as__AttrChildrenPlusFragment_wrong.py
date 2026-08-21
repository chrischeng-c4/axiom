# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "xml_dom_minidom"
# dimension = "type"
# case = "Attr__appendChild__node_as__AttrChildrenPlusFragment_wrong"
# subject = "xml.dom.minidom.Attr.appendChild(node: _AttrChildrenPlusFragment)"
# kind = "semantic"
# xfail = "force-typed arg enforcement pending; mamba must raise TypeError on wrong-typed node"
# mem_carveout = ""
# source = "vendor/typeshed/stdlib/xml/dom/minidom.pyi"
# status = "filled"
# ///
# mamba-xfail: force-typed arg enforcement pending; mamba must raise TypeError on wrong-typed node
# mamba-strict-type: TypeError
"""Type wall: xml.dom.minidom.Attr.appendChild(node: _AttrChildrenPlusFragment); call it with the wrong type.

typeshed contract: node is _AttrChildrenPlusFragment. mamba is force-typed, so a wrong-typed
argument MUST raise TypeError (CPython may accept or raise — mamba's to enforce)."""

class _W:
    pass


from xml.dom.minidom import Attr
obj = object.__new__(Attr)
try:
    obj.appendChild(_W())  # node: _AttrChildrenPlusFragment <- wrong-typed
    print("no_typeerror:")  # CPython accepted the wrong-typed arg; mamba must raise
except TypeError as e:
    print("typeerror:", type(e).__name__)
except Exception as e:
    print("setup_or_other:", type(e).__name__)
