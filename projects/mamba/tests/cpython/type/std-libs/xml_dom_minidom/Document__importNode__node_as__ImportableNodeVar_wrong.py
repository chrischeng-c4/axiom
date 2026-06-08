# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "xml_dom_minidom"
# dimension = "type"
# case = "Document__importNode__node_as__ImportableNodeVar_wrong"
# subject = "xml.dom.minidom.Document.importNode(node: _ImportableNodeVar)"
# kind = "semantic"
# xfail = "force-typed arg enforcement pending; mamba must raise TypeError on wrong-typed node"
# mem_carveout = ""
# source = "vendor/typeshed/stdlib/xml/dom/minidom.pyi"
# status = "filled"
# ///
# mamba-xfail: force-typed arg enforcement pending; mamba must raise TypeError on wrong-typed node
# mamba-strict-type: TypeError
"""Type wall: xml.dom.minidom.Document.importNode(node: _ImportableNodeVar); call it with the wrong type.

typeshed contract: node is _ImportableNodeVar. mamba is force-typed, so a wrong-typed
argument MUST raise TypeError (CPython may accept or raise — mamba's to enforce)."""

class _W:
    pass


from xml.dom.minidom import Document
obj = object.__new__(Document)
try:
    obj.importNode(_W(), True)  # node: _ImportableNodeVar <- wrong-typed
    print("no_typeerror:")  # CPython accepted the wrong-typed arg; mamba must raise
except TypeError as e:
    print("typeerror:", type(e).__name__)
except Exception as e:
    print("setup_or_other:", type(e).__name__)
