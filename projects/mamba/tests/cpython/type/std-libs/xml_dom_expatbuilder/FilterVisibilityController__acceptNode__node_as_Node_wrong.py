# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "xml_dom_expatbuilder"
# dimension = "type"
# case = "FilterVisibilityController__acceptNode__node_as_Node_wrong"
# subject = "xml.dom.expatbuilder.FilterVisibilityController.acceptNode(node: Node)"
# kind = "semantic"
# xfail = ""
# mem_carveout = ""
# source = "vendor/typeshed/stdlib/xml/dom/expatbuilder.pyi"
# status = "filled"
# ///
# mamba-strict-type: TypeError
"""Type wall: xml.dom.expatbuilder.FilterVisibilityController.acceptNode(node: Node); call it with the wrong type.

typeshed contract: node is Node. mamba is force-typed, so a wrong-typed
argument MUST raise TypeError (CPython may accept or raise — mamba's to enforce)."""

class _W:
    pass


from xml.dom.expatbuilder import FilterVisibilityController
obj = object.__new__(FilterVisibilityController)
try:
    obj.acceptNode(_W())  # node: Node <- wrong-typed
    print("no_typeerror:")  # CPython accepted the wrong-typed arg; mamba must raise
except TypeError as e:
    print("typeerror:", type(e).__name__)
except Exception as e:
    print("setup_or_other:", type(e).__name__)
