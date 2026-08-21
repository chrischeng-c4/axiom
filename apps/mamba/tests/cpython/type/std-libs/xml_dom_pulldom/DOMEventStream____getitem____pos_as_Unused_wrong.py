# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "xml_dom_pulldom"
# dimension = "type"
# case = "DOMEventStream____getitem____pos_as_Unused_wrong"
# subject = "xml.dom.pulldom.DOMEventStream.__getitem__(pos: Unused)"
# kind = "semantic"
# mem_carveout = ""
# source = "vendor/typeshed/stdlib/xml/dom/pulldom.pyi"
# status = "filled"
# ///
# mamba-strict-type: TypeError
"""Type wall: xml.dom.pulldom.DOMEventStream.__getitem__(pos: Unused); call it with the wrong type.

typeshed contract: pos is Unused. mamba is force-typed, so a wrong-typed
argument MUST raise TypeError (CPython may accept or raise — mamba's to enforce)."""

class _W:
    pass


from xml.dom.pulldom import DOMEventStream
obj = object.__new__(DOMEventStream)
try:
    obj.__getitem__(_W())  # pos: Unused <- wrong-typed
    print("no_typeerror:")  # CPython accepted the wrong-typed arg; mamba must raise
except TypeError as e:
    print("typeerror:", type(e).__name__)
except Exception as e:
    print("setup_or_other:", type(e).__name__)
