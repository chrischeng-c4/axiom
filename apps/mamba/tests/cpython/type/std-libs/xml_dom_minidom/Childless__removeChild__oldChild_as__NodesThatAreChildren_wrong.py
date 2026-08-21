# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "xml_dom_minidom"
# dimension = "type"
# case = "Childless__removeChild__oldChild_as__NodesThatAreChildren_wrong"
# subject = "xml.dom.minidom.Childless.removeChild(oldChild: _NodesThatAreChildren)"
# kind = "semantic"
# xfail = ""
# mem_carveout = ""
# source = "vendor/typeshed/stdlib/xml/dom/minidom.pyi"
# status = "filled"
# ///
# mamba-strict-type: TypeError
"""Type wall: xml.dom.minidom.Childless.removeChild(oldChild: _NodesThatAreChildren); call it with the wrong type.

typeshed contract: oldChild is _NodesThatAreChildren. mamba is force-typed, so a wrong-typed
argument MUST raise TypeError (CPython may accept or raise — mamba's to enforce)."""

class _W:
    pass


from xml.dom.minidom import Childless
obj = object.__new__(Childless)
try:
    obj.removeChild(_W())  # oldChild: _NodesThatAreChildren <- wrong-typed
    print("no_typeerror:")  # CPython accepted the wrong-typed arg; mamba must raise
except TypeError as e:
    print("typeerror:", type(e).__name__)
except Exception as e:
    print("setup_or_other:", type(e).__name__)
