# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "xml_dom_minidom"
# dimension = "type"
# case = "DocumentFragment__removeChild__oldChild_as__DFChildrenVar_wrong"
# subject = "xml.dom.minidom.DocumentFragment.removeChild(oldChild: _DFChildrenVar)"
# kind = "semantic"
# xfail = "force-typed arg enforcement pending; mamba must raise TypeError on wrong-typed oldChild"
# mem_carveout = ""
# source = "vendor/typeshed/stdlib/xml/dom/minidom.pyi"
# status = "filled"
# ///
# mamba-xfail: force-typed arg enforcement pending; mamba must raise TypeError on wrong-typed oldChild
# mamba-strict-type: TypeError
"""Type wall: xml.dom.minidom.DocumentFragment.removeChild(oldChild: _DFChildrenVar); call it with the wrong type.

typeshed contract: oldChild is _DFChildrenVar. mamba is force-typed, so a wrong-typed
argument MUST raise TypeError (CPython may accept or raise — mamba's to enforce)."""

class _W:
    pass


from xml.dom.minidom import DocumentFragment
obj = object.__new__(DocumentFragment)
try:
    obj.removeChild(_W())  # oldChild: _DFChildrenVar <- wrong-typed
    print("no_typeerror:")  # CPython accepted the wrong-typed arg; mamba must raise
except TypeError as e:
    print("typeerror:", type(e).__name__)
except Exception as e:
    print("setup_or_other:", type(e).__name__)
