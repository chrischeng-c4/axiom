# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "xml_dom_minidom"
# dimension = "type"
# case = "Document__removeChild__oldChild_as__DocumentChildrenVar_wrong"
# subject = "xml.dom.minidom.Document.removeChild(oldChild: _DocumentChildrenVar)"
# kind = "semantic"
# xfail = "force-typed arg enforcement pending; mamba must raise TypeError on wrong-typed oldChild"
# mem_carveout = ""
# source = "vendor/typeshed/stdlib/xml/dom/minidom.pyi"
# status = "filled"
# ///
# mamba-xfail: force-typed arg enforcement pending; mamba must raise TypeError on wrong-typed oldChild
# mamba-strict-type: TypeError
"""Type wall: xml.dom.minidom.Document.removeChild(oldChild: _DocumentChildrenVar); call it with the wrong type.

typeshed contract: oldChild is _DocumentChildrenVar. mamba is force-typed, so a wrong-typed
argument MUST raise TypeError (CPython may accept or raise — mamba's to enforce)."""

class _W:
    pass


from xml.dom.minidom import Document
obj = object.__new__(Document)
try:
    obj.removeChild(_W())  # oldChild: _DocumentChildrenVar <- wrong-typed
    print("no_typeerror:")  # CPython accepted the wrong-typed arg; mamba must raise
except TypeError as e:
    print("typeerror:", type(e).__name__)
except Exception as e:
    print("setup_or_other:", type(e).__name__)
