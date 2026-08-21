# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "xml_dom_minidom"
# dimension = "type"
# case = "Element__insertBefore__newChild_as__ElementChildrenPlusFragment_wrong"
# subject = "xml.dom.minidom.Element.insertBefore(newChild: _ElementChildrenPlusFragment)"
# kind = "semantic"
# xfail = "force-typed arg enforcement pending; mamba must raise TypeError on wrong-typed newChild"
# mem_carveout = ""
# source = "vendor/typeshed/stdlib/xml/dom/minidom.pyi"
# status = "filled"
# ///
# mamba-xfail: force-typed arg enforcement pending; mamba must raise TypeError on wrong-typed newChild
# mamba-strict-type: TypeError
"""Type wall: xml.dom.minidom.Element.insertBefore(newChild: _ElementChildrenPlusFragment); call it with the wrong type.

typeshed contract: newChild is _ElementChildrenPlusFragment. mamba is force-typed, so a wrong-typed
argument MUST raise TypeError (CPython may accept or raise — mamba's to enforce)."""

class _W:
    pass


from xml.dom.minidom import Element
obj = object.__new__(Element)
try:
    obj.insertBefore(_W(), None)  # newChild: _ElementChildrenPlusFragment <- wrong-typed
    print("no_typeerror:")  # CPython accepted the wrong-typed arg; mamba must raise
except TypeError as e:
    print("typeerror:", type(e).__name__)
except Exception as e:
    print("setup_or_other:", type(e).__name__)
