# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "xml_dom_minidom"
# dimension = "type"
# case = "DocumentFragment__appendChild__node_as__DFChildrenPlusFragment_wrong"
# subject = "xml.dom.minidom.DocumentFragment.appendChild(node: _DFChildrenPlusFragment)"
# kind = "semantic"
# xfail = "force-typed arg enforcement pending; mamba must raise TypeError on wrong-typed node"
# mem_carveout = ""
# source = "vendor/typeshed/stdlib/xml/dom/minidom.pyi"
# status = "filled"
# ///
# mamba-xfail: force-typed arg enforcement pending; mamba must raise TypeError on wrong-typed node
# mamba-strict-type: TypeError
"""Type wall: xml.dom.minidom.DocumentFragment.appendChild(node: _DFChildrenPlusFragment); call it with the wrong type.

typeshed contract: node is _DFChildrenPlusFragment. mamba is force-typed, so a wrong-typed
argument MUST raise TypeError (CPython may accept or raise — mamba's to enforce)."""

class _W:
    pass


from xml.dom.minidom import DocumentFragment
obj = object.__new__(DocumentFragment)
try:
    obj.appendChild(_W())  # node: _DFChildrenPlusFragment <- wrong-typed
    print("no_typeerror:")  # CPython accepted the wrong-typed arg; mamba must raise
except TypeError as e:
    print("typeerror:", type(e).__name__)
except Exception as e:
    print("setup_or_other:", type(e).__name__)
