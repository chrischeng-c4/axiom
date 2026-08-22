use super::super::super::super::harness::*;

/// Ported from `tests/cpython/type/std-libs/xml_dom_minicompat/EmptyNodeList____add____other_as_Iterable_wrong.py`.
#[test]
fn test_gen_type_std_libs_xml_dom_minicompat_EmptyNodeList____add____other_as_Iterable_wrong() {
    let out = run_type_wall_fixture(r###"# /// script
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
"###);
    assert!(out == "STRICT_TYPE_REJECTED" || out.starts_with("RUNTIME_REJECTED"),
        "type wall did not hold: {out}");
}

/// Ported from `tests/cpython/type/std-libs/xml_dom_minicompat/EmptyNodeList____radd____other_as_Iterable_wrong.py`.
#[test]
fn test_gen_type_std_libs_xml_dom_minicompat_EmptyNodeList____radd____other_as_Iterable_wrong() {
    let out = run_type_wall_fixture(r###"# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "xml_dom_minicompat"
# dimension = "type"
# case = "EmptyNodeList____radd____other_as_Iterable_wrong"
# subject = "xml.dom.minicompat.EmptyNodeList.__radd__(other: Iterable)"
# kind = "semantic"
# xfail = ""
# mem_carveout = ""
# source = "vendor/typeshed/stdlib/xml/dom/minicompat.pyi"
# status = "filled"
# ///
# mamba-strict-type: TypeError
"""Type wall: xml.dom.minicompat.EmptyNodeList.__radd__(other: Iterable); call it with the wrong type.

typeshed contract: other is Iterable. mamba is force-typed, so a wrong-typed
argument MUST raise TypeError (CPython may accept or raise — mamba's to enforce)."""

class _W:
    pass


from xml.dom.minicompat import EmptyNodeList
obj = object.__new__(EmptyNodeList)
try:
    obj.__radd__(_W())  # other: Iterable <- wrong-typed
    print("no_typeerror:")  # CPython accepted the wrong-typed arg; mamba must raise
except TypeError as e:
    print("typeerror:", type(e).__name__)
except Exception as e:
    print("setup_or_other:", type(e).__name__)
"###);
    assert!(out == "STRICT_TYPE_REJECTED" || out.starts_with("RUNTIME_REJECTED"),
        "type wall did not hold: {out}");
}

/// Ported from `tests/cpython/type/std-libs/xml_dom_minicompat/EmptyNodeList__item__index_as_int_wrong.py`.
#[test]
fn test_gen_type_std_libs_xml_dom_minicompat_EmptyNodeList__item__index_as_int_wrong() {
    let out = run_type_wall_fixture(r###"# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "xml_dom_minicompat"
# dimension = "type"
# case = "EmptyNodeList__item__index_as_int_wrong"
# subject = "xml.dom.minicompat.EmptyNodeList.item(index: int)"
# kind = "semantic"
# xfail = ""
# mem_carveout = ""
# source = "vendor/typeshed/stdlib/xml/dom/minicompat.pyi"
# status = "filled"
# ///
# mamba-strict-type: TypeError
"""Type wall: xml.dom.minicompat.EmptyNodeList.item(index: int); call it with the wrong type.

typeshed contract: index is int. mamba is force-typed, so a wrong-typed
argument MUST raise TypeError (CPython may accept or raise — mamba's to enforce)."""

from xml.dom.minicompat import EmptyNodeList
obj = object.__new__(EmptyNodeList)
try:
    obj.item("not_an_int")  # index: int <- wrong-typed
    print("no_typeerror:")  # CPython accepted the wrong-typed arg; mamba must raise
except TypeError as e:
    print("typeerror:", type(e).__name__)
except Exception as e:
    print("setup_or_other:", type(e).__name__)
"###);
    assert!(out == "STRICT_TYPE_REJECTED" || out.starts_with("RUNTIME_REJECTED"),
        "type wall did not hold: {out}");
}

/// Ported from `tests/cpython/type/std-libs/xml_dom_minicompat/NodeList__item__index_as_int_wrong.py`.
#[test]
fn test_gen_type_std_libs_xml_dom_minicompat_NodeList__item__index_as_int_wrong() {
    let out = run_type_wall_fixture(r###"# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "xml_dom_minicompat"
# dimension = "type"
# case = "NodeList__item__index_as_int_wrong"
# subject = "xml.dom.minicompat.NodeList.item(index: int)"
# kind = "semantic"
# xfail = ""
# mem_carveout = ""
# source = "vendor/typeshed/stdlib/xml/dom/minicompat.pyi"
# status = "filled"
# ///
# mamba-strict-type: TypeError
"""Type wall: xml.dom.minicompat.NodeList.item(index: int); call it with the wrong type.

typeshed contract: index is int. mamba is force-typed, so a wrong-typed
argument MUST raise TypeError (CPython may accept or raise — mamba's to enforce)."""

from xml.dom.minicompat import NodeList
obj = object.__new__(NodeList)
try:
    obj.item("not_an_int")  # index: int <- wrong-typed
    print("no_typeerror:")  # CPython accepted the wrong-typed arg; mamba must raise
except TypeError as e:
    print("typeerror:", type(e).__name__)
except Exception as e:
    print("setup_or_other:", type(e).__name__)
"###);
    assert!(out == "STRICT_TYPE_REJECTED" || out.starts_with("RUNTIME_REJECTED"),
        "type wall did not hold: {out}");
}
