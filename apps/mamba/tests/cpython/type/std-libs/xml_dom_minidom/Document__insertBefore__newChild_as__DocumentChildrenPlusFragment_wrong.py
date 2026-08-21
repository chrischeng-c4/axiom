# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "xml_dom_minidom"
# dimension = "type"
# case = "Document__insertBefore__newChild_as__DocumentChildrenPlusFragment_wrong"
# subject = "xml.dom.minidom.Document.insertBefore(newChild: _DocumentChildrenPlusFragment)"
# kind = "semantic"
# xfail = "force-typed arg enforcement pending; mamba must raise TypeError on wrong-typed newChild"
# mem_carveout = ""
# source = "vendor/typeshed/stdlib/xml/dom/minidom.pyi"
# status = "filled"
# ///
# mamba-xfail: force-typed arg enforcement pending; mamba must raise TypeError on wrong-typed newChild
# mamba-strict-type: TypeError
"""Type wall: xml.dom.minidom.Document.insertBefore(newChild: _DocumentChildrenPlusFragment); call it with the wrong type.

typeshed contract: newChild is _DocumentChildrenPlusFragment. mamba is force-typed, so a wrong-typed
argument MUST raise TypeError (CPython may accept or raise — mamba's to enforce)."""

class _W:
    pass


from xml.dom.minidom import Document
obj = object.__new__(Document)
try:
    obj.insertBefore(_W(), None)  # newChild: _DocumentChildrenPlusFragment <- wrong-typed
    print("no_typeerror:")  # CPython accepted the wrong-typed arg; mamba must raise
except TypeError as e:
    print("typeerror:", type(e).__name__)
except Exception as e:
    print("setup_or_other:", type(e).__name__)
