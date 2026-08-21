# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "xml_etree_ElementTree"
# dimension = "type"
# case = "fromstringlist__sequence_as_Sequence_wrong"
# subject = "xml.etree.ElementTree.fromstringlist(sequence: Sequence)"
# kind = "semantic"
# xfail = "force-typed arg enforcement pending; mamba must raise TypeError on wrong-typed sequence"
# mem_carveout = ""
# source = "vendor/typeshed/stdlib/xml/etree/ElementTree.pyi"
# status = "filled"
# ///
# mamba-xfail: force-typed arg enforcement pending; mamba must raise TypeError on wrong-typed sequence
# mamba-strict-type: TypeError
"""Type wall: xml.etree.ElementTree.fromstringlist(sequence: Sequence); call it with the wrong type.

typeshed contract: sequence is Sequence. mamba is force-typed, so a wrong-typed
argument MUST raise TypeError (CPython may accept or raise — mamba's to enforce)."""

class _W:
    pass


from xml.etree.ElementTree import fromstringlist
try:
    fromstringlist(_W())  # sequence: Sequence <- wrong-typed
    print("no_typeerror:")  # CPython accepted the wrong-typed arg; mamba must raise
except TypeError as e:
    print("typeerror:", type(e).__name__)
except Exception as e:
    print("setup_or_other:", type(e).__name__)
