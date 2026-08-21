# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "xml_dom_pulldom"
# dimension = "type"
# case = "PullDOM__processingInstruction__target_as_str_wrong"
# subject = "xml.dom.pulldom.PullDOM.processingInstruction(target: str)"
# kind = "semantic"
# xfail = ""
# mem_carveout = ""
# source = "vendor/typeshed/stdlib/xml/dom/pulldom.pyi"
# status = "filled"
# ///
# mamba-strict-type: TypeError
"""Type wall: xml.dom.pulldom.PullDOM.processingInstruction(target: str); call it with the wrong type.

typeshed contract: target is str. mamba is force-typed, so a wrong-typed
argument MUST raise TypeError (CPython may accept or raise — mamba's to enforce)."""

from xml.dom.pulldom import PullDOM
obj = object.__new__(PullDOM)
try:
    obj.processingInstruction(12345, "")  # target: str <- wrong-typed
    print("no_typeerror:")  # CPython accepted the wrong-typed arg; mamba must raise
except TypeError as e:
    print("typeerror:", type(e).__name__)
except Exception as e:
    print("setup_or_other:", type(e).__name__)
