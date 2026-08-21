# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "xml_dom_minicompat"
# dimension = "type"
# case = "EmptyNodeList____add____other_as_Iterable_wrong"
# subject = "xml.dom.minicompat.EmptyNodeList.__add__(other: Iterable)"
# kind = "semantic"
# xfail = ""
# mem_carveout = ""
# source = "vendor/typeshed/stdlib/xml/dom/minicompat.pyi"
# status = "filled"
# ///
# mamba-strict-type: TypeError
"""Type wall: xml.dom.minicompat.EmptyNodeList.__add__(other: Iterable); call it with the wrong type.

typeshed contract: other is Iterable. mamba is force-typed, so a wrong-typed
argument MUST raise TypeError (CPython may accept or raise — mamba's to enforce)."""

class _W:
    pass


from xml.dom.minicompat import EmptyNodeList
obj = object.__new__(EmptyNodeList)
try:
    obj.__add__(_W())  # other: Iterable <- wrong-typed
    print("no_typeerror:")  # CPython accepted the wrong-typed arg; mamba must raise
except TypeError as e:
    print("typeerror:", type(e).__name__)
except Exception as e:
    print("setup_or_other:", type(e).__name__)
