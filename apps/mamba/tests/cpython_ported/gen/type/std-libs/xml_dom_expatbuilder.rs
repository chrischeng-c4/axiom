use super::super::super::super::harness::*;

/// Ported from `tests/cpython/type/std-libs/xml_dom_expatbuilder/ElementInfo__getAttributeTypeNS__namespaceURI_as_typed_wrong.py`.
#[test]
fn test_gen_type_std_libs_xml_dom_expatbuilder_ElementInfo__getAttributeTypeNS__namespaceURI_as_typed_wrong() {
    let out = run_type_wall_fixture(r###"# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "xml_dom_expatbuilder"
# dimension = "type"
# case = "ElementInfo__getAttributeTypeNS__namespaceURI_as_typed_wrong"
# subject = "xml.dom.expatbuilder.ElementInfo.getAttributeTypeNS(namespaceURI: typed)"
# kind = "semantic"
# mem_carveout = ""
# source = "vendor/typeshed/stdlib/xml/dom/expatbuilder.pyi"
# status = "filled"
# ///
# mamba-strict-type: TypeError
"""Type wall: xml.dom.expatbuilder.ElementInfo.getAttributeTypeNS(namespaceURI: typed); call it with the wrong type.

typeshed contract: namespaceURI is typed. mamba is force-typed, so a wrong-typed
argument MUST raise TypeError (CPython may accept or raise — mamba's to enforce)."""

class _W:
    pass


from xml.dom.expatbuilder import ElementInfo
obj = object.__new__(ElementInfo)
try:
    obj.getAttributeTypeNS(_W(), "")  # namespaceURI: typed <- wrong-typed
    print("no_typeerror:")  # CPython accepted the wrong-typed arg; mamba must raise
except TypeError as e:
    print("typeerror:", type(e).__name__)
except Exception as e:
    print("setup_or_other:", type(e).__name__)
"###);
    assert!(out == "STRICT_TYPE_REJECTED" || out.starts_with("RUNTIME_REJECTED"),
        "type wall did not hold: {out}");
}

/// Ported from `tests/cpython/type/std-libs/xml_dom_expatbuilder/ElementInfo__getAttributeType__aname_as_str_wrong.py`.
#[test]
fn test_gen_type_std_libs_xml_dom_expatbuilder_ElementInfo__getAttributeType__aname_as_str_wrong() {
    let out = run_type_wall_fixture(r###"# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "xml_dom_expatbuilder"
# dimension = "type"
# case = "ElementInfo__getAttributeType__aname_as_str_wrong"
# subject = "xml.dom.expatbuilder.ElementInfo.getAttributeType(aname: str)"
# kind = "semantic"
# mem_carveout = ""
# source = "vendor/typeshed/stdlib/xml/dom/expatbuilder.pyi"
# status = "filled"
# ///
# mamba-strict-type: TypeError
"""Type wall: xml.dom.expatbuilder.ElementInfo.getAttributeType(aname: str); call it with the wrong type.

typeshed contract: aname is str. mamba is force-typed, so a wrong-typed
argument MUST raise TypeError (CPython may accept or raise — mamba's to enforce)."""

from xml.dom.expatbuilder import ElementInfo
obj = object.__new__(ElementInfo)
try:
    obj.getAttributeType(12345)  # aname: str <- wrong-typed
    print("no_typeerror:")  # CPython accepted the wrong-typed arg; mamba must raise
except TypeError as e:
    print("typeerror:", type(e).__name__)
except Exception as e:
    print("setup_or_other:", type(e).__name__)
"###);
    assert!(out == "STRICT_TYPE_REJECTED" || out.starts_with("RUNTIME_REJECTED"),
        "type wall did not hold: {out}");
}

/// Ported from `tests/cpython/type/std-libs/xml_dom_expatbuilder/ElementInfo__init__tagName_as_str_wrong.py`.
#[test]
fn test_gen_type_std_libs_xml_dom_expatbuilder_ElementInfo__init__tagName_as_str_wrong() {
    let out = run_type_wall_fixture(r###"# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "xml_dom_expatbuilder"
# dimension = "type"
# case = "ElementInfo__init__tagName_as_str_wrong"
# subject = "xml.dom.expatbuilder.ElementInfo.__init__(tagName: str)"
# kind = "semantic"
# mem_carveout = ""
# source = "vendor/typeshed/stdlib/xml/dom/expatbuilder.pyi"
# status = "filled"
# ///
# mamba-strict-type: TypeError
"""Type wall: xml.dom.expatbuilder.ElementInfo.__init__(tagName: str); call it with the wrong type.

typeshed contract: tagName is str. mamba is force-typed, so a wrong-typed
argument MUST raise TypeError (CPython may accept or raise — mamba's to enforce)."""

from xml.dom.expatbuilder import ElementInfo
try:
    ElementInfo(12345)  # tagName: str <- wrong-typed
    print("no_typeerror:")  # CPython accepted the wrong-typed arg; mamba must raise
except TypeError as e:
    print("typeerror:", type(e).__name__)
except Exception as e:
    print("setup_or_other:", type(e).__name__)
"###);
    assert!(out == "STRICT_TYPE_REJECTED" || out.starts_with("RUNTIME_REJECTED"),
        "type wall did not hold: {out}");
}

/// Ported from `tests/cpython/type/std-libs/xml_dom_expatbuilder/ElementInfo__isIdNS__euri_as_str_wrong.py`.
#[test]
fn test_gen_type_std_libs_xml_dom_expatbuilder_ElementInfo__isIdNS__euri_as_str_wrong() {
    let out = run_type_wall_fixture(r###"# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "xml_dom_expatbuilder"
# dimension = "type"
# case = "ElementInfo__isIdNS__euri_as_str_wrong"
# subject = "xml.dom.expatbuilder.ElementInfo.isIdNS(euri: str)"
# kind = "semantic"
# mem_carveout = ""
# source = "vendor/typeshed/stdlib/xml/dom/expatbuilder.pyi"
# status = "filled"
# ///
# mamba-strict-type: TypeError
"""Type wall: xml.dom.expatbuilder.ElementInfo.isIdNS(euri: str); call it with the wrong type.

typeshed contract: euri is str. mamba is force-typed, so a wrong-typed
argument MUST raise TypeError (CPython may accept or raise — mamba's to enforce)."""

from xml.dom.expatbuilder import ElementInfo
obj = object.__new__(ElementInfo)
try:
    obj.isIdNS(12345, "", "", "")  # euri: str <- wrong-typed
    print("no_typeerror:")  # CPython accepted the wrong-typed arg; mamba must raise
except TypeError as e:
    print("typeerror:", type(e).__name__)
except Exception as e:
    print("setup_or_other:", type(e).__name__)
"###);
    assert!(out == "STRICT_TYPE_REJECTED" || out.starts_with("RUNTIME_REJECTED"),
        "type wall did not hold: {out}");
}

/// Ported from `tests/cpython/type/std-libs/xml_dom_expatbuilder/ElementInfo__isId__aname_as_str_wrong.py`.
#[test]
fn test_gen_type_std_libs_xml_dom_expatbuilder_ElementInfo__isId__aname_as_str_wrong() {
    let out = run_type_wall_fixture(r###"# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "xml_dom_expatbuilder"
# dimension = "type"
# case = "ElementInfo__isId__aname_as_str_wrong"
# subject = "xml.dom.expatbuilder.ElementInfo.isId(aname: str)"
# kind = "semantic"
# mem_carveout = ""
# source = "vendor/typeshed/stdlib/xml/dom/expatbuilder.pyi"
# status = "filled"
# ///
# mamba-strict-type: TypeError
"""Type wall: xml.dom.expatbuilder.ElementInfo.isId(aname: str); call it with the wrong type.

typeshed contract: aname is str. mamba is force-typed, so a wrong-typed
argument MUST raise TypeError (CPython may accept or raise — mamba's to enforce)."""

from xml.dom.expatbuilder import ElementInfo
obj = object.__new__(ElementInfo)
try:
    obj.isId(12345)  # aname: str <- wrong-typed
    print("no_typeerror:")  # CPython accepted the wrong-typed arg; mamba must raise
except TypeError as e:
    print("typeerror:", type(e).__name__)
except Exception as e:
    print("setup_or_other:", type(e).__name__)
"###);
    assert!(out == "STRICT_TYPE_REJECTED" || out.starts_with("RUNTIME_REJECTED"),
        "type wall did not hold: {out}");
}

/// Ported from `tests/cpython/type/std-libs/xml_dom_expatbuilder/ExpatBuilder__attlist_decl_handler__elem_as_str_wrong.py`.
#[test]
fn test_gen_type_std_libs_xml_dom_expatbuilder_ExpatBuilder__attlist_decl_handler__elem_as_str_wrong() {
    let out = run_type_wall_fixture(r###"# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "xml_dom_expatbuilder"
# dimension = "type"
# case = "ExpatBuilder__attlist_decl_handler__elem_as_str_wrong"
# subject = "xml.dom.expatbuilder.ExpatBuilder.attlist_decl_handler(elem: str)"
# kind = "semantic"
# mem_carveout = ""
# source = "vendor/typeshed/stdlib/xml/dom/expatbuilder.pyi"
# status = "filled"
# ///
# mamba-strict-type: TypeError
"""Type wall: xml.dom.expatbuilder.ExpatBuilder.attlist_decl_handler(elem: str); call it with the wrong type.

typeshed contract: elem is str. mamba is force-typed, so a wrong-typed
argument MUST raise TypeError (CPython may accept or raise — mamba's to enforce)."""

from xml.dom.expatbuilder import ExpatBuilder
obj = object.__new__(ExpatBuilder)
try:
    obj.attlist_decl_handler(12345, "", "", None, True)  # elem: str <- wrong-typed
    print("no_typeerror:")  # CPython accepted the wrong-typed arg; mamba must raise
except TypeError as e:
    print("typeerror:", type(e).__name__)
except Exception as e:
    print("setup_or_other:", type(e).__name__)
"###);
    assert!(out == "STRICT_TYPE_REJECTED" || out.starts_with("RUNTIME_REJECTED"),
        "type wall did not hold: {out}");
}

/// Ported from `tests/cpython/type/std-libs/xml_dom_expatbuilder/ExpatBuilder__character_data_handler__data_as_str_wrong.py`.
#[test]
fn test_gen_type_std_libs_xml_dom_expatbuilder_ExpatBuilder__character_data_handler__data_as_str_wrong() {
    let out = run_type_wall_fixture(r###"# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "xml_dom_expatbuilder"
# dimension = "type"
# case = "ExpatBuilder__character_data_handler__data_as_str_wrong"
# subject = "xml.dom.expatbuilder.ExpatBuilder.character_data_handler(data: str)"
# kind = "semantic"
# mem_carveout = ""
# source = "vendor/typeshed/stdlib/xml/dom/expatbuilder.pyi"
# status = "filled"
# ///
# mamba-strict-type: TypeError
"""Type wall: xml.dom.expatbuilder.ExpatBuilder.character_data_handler(data: str); call it with the wrong type.

typeshed contract: data is str. mamba is force-typed, so a wrong-typed
argument MUST raise TypeError (CPython may accept or raise — mamba's to enforce)."""

from xml.dom.expatbuilder import ExpatBuilder
obj = object.__new__(ExpatBuilder)
try:
    obj.character_data_handler(12345)  # data: str <- wrong-typed
    print("no_typeerror:")  # CPython accepted the wrong-typed arg; mamba must raise
except TypeError as e:
    print("typeerror:", type(e).__name__)
except Exception as e:
    print("setup_or_other:", type(e).__name__)
"###);
    assert!(out == "STRICT_TYPE_REJECTED" || out.starts_with("RUNTIME_REJECTED"),
        "type wall did not hold: {out}");
}

/// Ported from `tests/cpython/type/std-libs/xml_dom_expatbuilder/ExpatBuilder__character_data_handler_cdata__data_as_str_wrong.py`.
#[test]
fn test_gen_type_std_libs_xml_dom_expatbuilder_ExpatBuilder__character_data_handler_cdata__data_as_str_wrong() {
    let out = run_type_wall_fixture(r###"# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "xml_dom_expatbuilder"
# dimension = "type"
# case = "ExpatBuilder__character_data_handler_cdata__data_as_str_wrong"
# subject = "xml.dom.expatbuilder.ExpatBuilder.character_data_handler_cdata(data: str)"
# kind = "semantic"
# mem_carveout = ""
# source = "vendor/typeshed/stdlib/xml/dom/expatbuilder.pyi"
# status = "filled"
# ///
# mamba-strict-type: TypeError
"""Type wall: xml.dom.expatbuilder.ExpatBuilder.character_data_handler_cdata(data: str); call it with the wrong type.

typeshed contract: data is str. mamba is force-typed, so a wrong-typed
argument MUST raise TypeError (CPython may accept or raise — mamba's to enforce)."""

from xml.dom.expatbuilder import ExpatBuilder
obj = object.__new__(ExpatBuilder)
try:
    obj.character_data_handler_cdata(12345)  # data: str <- wrong-typed
    print("no_typeerror:")  # CPython accepted the wrong-typed arg; mamba must raise
except TypeError as e:
    print("typeerror:", type(e).__name__)
except Exception as e:
    print("setup_or_other:", type(e).__name__)
"###);
    assert!(out == "STRICT_TYPE_REJECTED" || out.starts_with("RUNTIME_REJECTED"),
        "type wall did not hold: {out}");
}

/// Ported from `tests/cpython/type/std-libs/xml_dom_expatbuilder/ExpatBuilder__comment_handler__data_as_str_wrong.py`.
#[test]
fn test_gen_type_std_libs_xml_dom_expatbuilder_ExpatBuilder__comment_handler__data_as_str_wrong() {
    let out = run_type_wall_fixture(r###"# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "xml_dom_expatbuilder"
# dimension = "type"
# case = "ExpatBuilder__comment_handler__data_as_str_wrong"
# subject = "xml.dom.expatbuilder.ExpatBuilder.comment_handler(data: str)"
# kind = "semantic"
# mem_carveout = ""
# source = "vendor/typeshed/stdlib/xml/dom/expatbuilder.pyi"
# status = "filled"
# ///
# mamba-strict-type: TypeError
"""Type wall: xml.dom.expatbuilder.ExpatBuilder.comment_handler(data: str); call it with the wrong type.

typeshed contract: data is str. mamba is force-typed, so a wrong-typed
argument MUST raise TypeError (CPython may accept or raise — mamba's to enforce)."""

from xml.dom.expatbuilder import ExpatBuilder
obj = object.__new__(ExpatBuilder)
try:
    obj.comment_handler(12345)  # data: str <- wrong-typed
    print("no_typeerror:")  # CPython accepted the wrong-typed arg; mamba must raise
except TypeError as e:
    print("typeerror:", type(e).__name__)
except Exception as e:
    print("setup_or_other:", type(e).__name__)
"###);
    assert!(out == "STRICT_TYPE_REJECTED" || out.starts_with("RUNTIME_REJECTED"),
        "type wall did not hold: {out}");
}

/// Ported from `tests/cpython/type/std-libs/xml_dom_expatbuilder/ExpatBuilder__element_decl_handler__name_as_str_wrong.py`.
#[test]
fn test_gen_type_std_libs_xml_dom_expatbuilder_ExpatBuilder__element_decl_handler__name_as_str_wrong() {
    let out = run_type_wall_fixture(r###"# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "xml_dom_expatbuilder"
# dimension = "type"
# case = "ExpatBuilder__element_decl_handler__name_as_str_wrong"
# subject = "xml.dom.expatbuilder.ExpatBuilder.element_decl_handler(name: str)"
# kind = "semantic"
# mem_carveout = ""
# source = "vendor/typeshed/stdlib/xml/dom/expatbuilder.pyi"
# status = "filled"
# ///
# mamba-strict-type: TypeError
"""Type wall: xml.dom.expatbuilder.ExpatBuilder.element_decl_handler(name: str); call it with the wrong type.

typeshed contract: name is str. mamba is force-typed, so a wrong-typed
argument MUST raise TypeError (CPython may accept or raise — mamba's to enforce)."""

from xml.dom.expatbuilder import ExpatBuilder
obj = object.__new__(ExpatBuilder)
try:
    obj.element_decl_handler(12345, None)  # name: str <- wrong-typed
    print("no_typeerror:")  # CPython accepted the wrong-typed arg; mamba must raise
except TypeError as e:
    print("typeerror:", type(e).__name__)
except Exception as e:
    print("setup_or_other:", type(e).__name__)
"###);
    assert!(out == "STRICT_TYPE_REJECTED" || out.starts_with("RUNTIME_REJECTED"),
        "type wall did not hold: {out}");
}

/// Ported from `tests/cpython/type/std-libs/xml_dom_expatbuilder/ExpatBuilder__end_element_handler__name_as_str_wrong.py`.
#[test]
fn test_gen_type_std_libs_xml_dom_expatbuilder_ExpatBuilder__end_element_handler__name_as_str_wrong() {
    let out = run_type_wall_fixture(r###"# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "xml_dom_expatbuilder"
# dimension = "type"
# case = "ExpatBuilder__end_element_handler__name_as_str_wrong"
# subject = "xml.dom.expatbuilder.ExpatBuilder.end_element_handler(name: str)"
# kind = "semantic"
# mem_carveout = ""
# source = "vendor/typeshed/stdlib/xml/dom/expatbuilder.pyi"
# status = "filled"
# ///
# mamba-strict-type: TypeError
"""Type wall: xml.dom.expatbuilder.ExpatBuilder.end_element_handler(name: str); call it with the wrong type.

typeshed contract: name is str. mamba is force-typed, so a wrong-typed
argument MUST raise TypeError (CPython may accept or raise — mamba's to enforce)."""

from xml.dom.expatbuilder import ExpatBuilder
obj = object.__new__(ExpatBuilder)
try:
    obj.end_element_handler(12345)  # name: str <- wrong-typed
    print("no_typeerror:")  # CPython accepted the wrong-typed arg; mamba must raise
except TypeError as e:
    print("typeerror:", type(e).__name__)
except Exception as e:
    print("setup_or_other:", type(e).__name__)
"###);
    assert!(out == "STRICT_TYPE_REJECTED" || out.starts_with("RUNTIME_REJECTED"),
        "type wall did not hold: {out}");
}

/// Ported from `tests/cpython/type/std-libs/xml_dom_expatbuilder/ExpatBuilder__entity_decl_handler__entityName_as_str_wrong.py`.
#[test]
fn test_gen_type_std_libs_xml_dom_expatbuilder_ExpatBuilder__entity_decl_handler__entityName_as_str_wrong() {
    let out = run_type_wall_fixture(r###"# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "xml_dom_expatbuilder"
# dimension = "type"
# case = "ExpatBuilder__entity_decl_handler__entityName_as_str_wrong"
# subject = "xml.dom.expatbuilder.ExpatBuilder.entity_decl_handler(entityName: str)"
# kind = "semantic"
# mem_carveout = ""
# source = "vendor/typeshed/stdlib/xml/dom/expatbuilder.pyi"
# status = "filled"
# ///
# mamba-strict-type: TypeError
"""Type wall: xml.dom.expatbuilder.ExpatBuilder.entity_decl_handler(entityName: str); call it with the wrong type.

typeshed contract: entityName is str. mamba is force-typed, so a wrong-typed
argument MUST raise TypeError (CPython may accept or raise — mamba's to enforce)."""

from xml.dom.expatbuilder import ExpatBuilder
obj = object.__new__(ExpatBuilder)
try:
    obj.entity_decl_handler(12345, True, None, None, "", None, None)  # entityName: str <- wrong-typed
    print("no_typeerror:")  # CPython accepted the wrong-typed arg; mamba must raise
except TypeError as e:
    print("typeerror:", type(e).__name__)
except Exception as e:
    print("setup_or_other:", type(e).__name__)
"###);
    assert!(out == "STRICT_TYPE_REJECTED" || out.starts_with("RUNTIME_REJECTED"),
        "type wall did not hold: {out}");
}

/// Ported from `tests/cpython/type/std-libs/xml_dom_expatbuilder/ExpatBuilder__external_entity_ref_handler__context_as_str_wrong.py`.
#[test]
fn test_gen_type_std_libs_xml_dom_expatbuilder_ExpatBuilder__external_entity_ref_handler__context_as_str_wrong() {
    let out = run_type_wall_fixture(r###"# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "xml_dom_expatbuilder"
# dimension = "type"
# case = "ExpatBuilder__external_entity_ref_handler__context_as_str_wrong"
# subject = "xml.dom.expatbuilder.ExpatBuilder.external_entity_ref_handler(context: str)"
# kind = "semantic"
# mem_carveout = ""
# source = "vendor/typeshed/stdlib/xml/dom/expatbuilder.pyi"
# status = "filled"
# ///
# mamba-strict-type: TypeError
"""Type wall: xml.dom.expatbuilder.ExpatBuilder.external_entity_ref_handler(context: str); call it with the wrong type.

typeshed contract: context is str. mamba is force-typed, so a wrong-typed
argument MUST raise TypeError (CPython may accept or raise — mamba's to enforce)."""

from xml.dom.expatbuilder import ExpatBuilder
obj = object.__new__(ExpatBuilder)
try:
    obj.external_entity_ref_handler(12345, None, None, None)  # context: str <- wrong-typed
    print("no_typeerror:")  # CPython accepted the wrong-typed arg; mamba must raise
except TypeError as e:
    print("typeerror:", type(e).__name__)
except Exception as e:
    print("setup_or_other:", type(e).__name__)
"###);
    assert!(out == "STRICT_TYPE_REJECTED" || out.starts_with("RUNTIME_REJECTED"),
        "type wall did not hold: {out}");
}

/// Ported from `tests/cpython/type/std-libs/xml_dom_expatbuilder/ExpatBuilder__first_element_handler__name_as_str_wrong.py`.
#[test]
fn test_gen_type_std_libs_xml_dom_expatbuilder_ExpatBuilder__first_element_handler__name_as_str_wrong() {
    let out = run_type_wall_fixture(r###"# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "xml_dom_expatbuilder"
# dimension = "type"
# case = "ExpatBuilder__first_element_handler__name_as_str_wrong"
# subject = "xml.dom.expatbuilder.ExpatBuilder.first_element_handler(name: str)"
# kind = "semantic"
# mem_carveout = ""
# source = "vendor/typeshed/stdlib/xml/dom/expatbuilder.pyi"
# status = "filled"
# ///
# mamba-strict-type: TypeError
"""Type wall: xml.dom.expatbuilder.ExpatBuilder.first_element_handler(name: str); call it with the wrong type.

typeshed contract: name is str. mamba is force-typed, so a wrong-typed
argument MUST raise TypeError (CPython may accept or raise — mamba's to enforce)."""

from xml.dom.expatbuilder import ExpatBuilder
obj = object.__new__(ExpatBuilder)
try:
    obj.first_element_handler(12345, None)  # name: str <- wrong-typed
    print("no_typeerror:")  # CPython accepted the wrong-typed arg; mamba must raise
except TypeError as e:
    print("typeerror:", type(e).__name__)
except Exception as e:
    print("setup_or_other:", type(e).__name__)
"###);
    assert!(out == "STRICT_TYPE_REJECTED" || out.starts_with("RUNTIME_REJECTED"),
        "type wall did not hold: {out}");
}

/// Ported from `tests/cpython/type/std-libs/xml_dom_expatbuilder/ExpatBuilder__init__options_as_typed_wrong.py`.
#[test]
fn test_gen_type_std_libs_xml_dom_expatbuilder_ExpatBuilder__init__options_as_typed_wrong() {
    let out = run_type_wall_fixture(r###"# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "xml_dom_expatbuilder"
# dimension = "type"
# case = "ExpatBuilder__init__options_as_typed_wrong"
# subject = "xml.dom.expatbuilder.ExpatBuilder.__init__(options: typed)"
# kind = "semantic"
# mem_carveout = ""
# source = "vendor/typeshed/stdlib/xml/dom/expatbuilder.pyi"
# status = "filled"
# ///
# mamba-strict-type: TypeError
"""Type wall: xml.dom.expatbuilder.ExpatBuilder.__init__(options: typed); call it with the wrong type.

typeshed contract: options is typed. mamba is force-typed, so a wrong-typed
argument MUST raise TypeError (CPython may accept or raise — mamba's to enforce)."""

class _W:
    pass


from xml.dom.expatbuilder import ExpatBuilder
try:
    ExpatBuilder(_W())  # options: typed <- wrong-typed
    print("no_typeerror:")  # CPython accepted the wrong-typed arg; mamba must raise
except TypeError as e:
    print("typeerror:", type(e).__name__)
except Exception as e:
    print("setup_or_other:", type(e).__name__)
"###);
    assert!(out == "STRICT_TYPE_REJECTED" || out.starts_with("RUNTIME_REJECTED"),
        "type wall did not hold: {out}");
}

/// Ported from `tests/cpython/type/std-libs/xml_dom_expatbuilder/ExpatBuilder__install__parser_as_XMLParserType_wrong.py`.
#[test]
fn test_gen_type_std_libs_xml_dom_expatbuilder_ExpatBuilder__install__parser_as_XMLParserType_wrong() {
    let out = run_type_wall_fixture(r###"# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "xml_dom_expatbuilder"
# dimension = "type"
# case = "ExpatBuilder__install__parser_as_XMLParserType_wrong"
# subject = "xml.dom.expatbuilder.ExpatBuilder.install(parser: XMLParserType)"
# kind = "semantic"
# mem_carveout = ""
# source = "vendor/typeshed/stdlib/xml/dom/expatbuilder.pyi"
# status = "filled"
# ///
# mamba-strict-type: TypeError
"""Type wall: xml.dom.expatbuilder.ExpatBuilder.install(parser: XMLParserType); call it with the wrong type.

typeshed contract: parser is XMLParserType. mamba is force-typed, so a wrong-typed
argument MUST raise TypeError (CPython may accept or raise — mamba's to enforce)."""

class _W:
    pass


from xml.dom.expatbuilder import ExpatBuilder
obj = object.__new__(ExpatBuilder)
try:
    obj.install(_W())  # parser: XMLParserType <- wrong-typed
    print("no_typeerror:")  # CPython accepted the wrong-typed arg; mamba must raise
except TypeError as e:
    print("typeerror:", type(e).__name__)
except Exception as e:
    print("setup_or_other:", type(e).__name__)
"###);
    assert!(out == "STRICT_TYPE_REJECTED" || out.starts_with("RUNTIME_REJECTED"),
        "type wall did not hold: {out}");
}

/// Ported from `tests/cpython/type/std-libs/xml_dom_expatbuilder/ExpatBuilder__notation_decl_handler__notationName_as_str_wrong.py`.
#[test]
fn test_gen_type_std_libs_xml_dom_expatbuilder_ExpatBuilder__notation_decl_handler__notationName_as_str_wrong() {
    let out = run_type_wall_fixture(r###"# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "xml_dom_expatbuilder"
# dimension = "type"
# case = "ExpatBuilder__notation_decl_handler__notationName_as_str_wrong"
# subject = "xml.dom.expatbuilder.ExpatBuilder.notation_decl_handler(notationName: str)"
# kind = "semantic"
# mem_carveout = ""
# source = "vendor/typeshed/stdlib/xml/dom/expatbuilder.pyi"
# status = "filled"
# ///
# mamba-strict-type: TypeError
"""Type wall: xml.dom.expatbuilder.ExpatBuilder.notation_decl_handler(notationName: str); call it with the wrong type.

typeshed contract: notationName is str. mamba is force-typed, so a wrong-typed
argument MUST raise TypeError (CPython may accept or raise — mamba's to enforce)."""

from xml.dom.expatbuilder import ExpatBuilder
obj = object.__new__(ExpatBuilder)
try:
    obj.notation_decl_handler(12345, None, "", None)  # notationName: str <- wrong-typed
    print("no_typeerror:")  # CPython accepted the wrong-typed arg; mamba must raise
except TypeError as e:
    print("typeerror:", type(e).__name__)
except Exception as e:
    print("setup_or_other:", type(e).__name__)
"###);
    assert!(out == "STRICT_TYPE_REJECTED" || out.starts_with("RUNTIME_REJECTED"),
        "type wall did not hold: {out}");
}

/// Ported from `tests/cpython/type/std-libs/xml_dom_expatbuilder/ExpatBuilder__parseFile__file_as_SupportsRead_wrong.py`.
#[test]
fn test_gen_type_std_libs_xml_dom_expatbuilder_ExpatBuilder__parseFile__file_as_SupportsRead_wrong() {
    let out = run_type_wall_fixture(r###"# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "xml_dom_expatbuilder"
# dimension = "type"
# case = "ExpatBuilder__parseFile__file_as_SupportsRead_wrong"
# subject = "xml.dom.expatbuilder.ExpatBuilder.parseFile(file: SupportsRead)"
# kind = "semantic"
# mem_carveout = ""
# source = "vendor/typeshed/stdlib/xml/dom/expatbuilder.pyi"
# status = "filled"
# ///
# mamba-strict-type: TypeError
"""Type wall: xml.dom.expatbuilder.ExpatBuilder.parseFile(file: SupportsRead); call it with the wrong type.

typeshed contract: file is SupportsRead. mamba is force-typed, so a wrong-typed
argument MUST raise TypeError (CPython may accept or raise — mamba's to enforce)."""

class _W:
    pass


from xml.dom.expatbuilder import ExpatBuilder
obj = object.__new__(ExpatBuilder)
try:
    obj.parseFile(_W())  # file: SupportsRead <- wrong-typed
    print("no_typeerror:")  # CPython accepted the wrong-typed arg; mamba must raise
except TypeError as e:
    print("typeerror:", type(e).__name__)
except Exception as e:
    print("setup_or_other:", type(e).__name__)
"###);
    assert!(out == "STRICT_TYPE_REJECTED" || out.starts_with("RUNTIME_REJECTED"),
        "type wall did not hold: {out}");
}

/// Ported from `tests/cpython/type/std-libs/xml_dom_expatbuilder/ExpatBuilder__parseString__string_as_typed_wrong.py`.
#[test]
fn test_gen_type_std_libs_xml_dom_expatbuilder_ExpatBuilder__parseString__string_as_typed_wrong() {
    let out = run_type_wall_fixture(r###"# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "xml_dom_expatbuilder"
# dimension = "type"
# case = "ExpatBuilder__parseString__string_as_typed_wrong"
# subject = "xml.dom.expatbuilder.ExpatBuilder.parseString(string: typed)"
# kind = "semantic"
# mem_carveout = ""
# source = "vendor/typeshed/stdlib/xml/dom/expatbuilder.pyi"
# status = "filled"
# ///
# mamba-strict-type: TypeError
"""Type wall: xml.dom.expatbuilder.ExpatBuilder.parseString(string: typed); call it with the wrong type.

typeshed contract: string is typed. mamba is force-typed, so a wrong-typed
argument MUST raise TypeError (CPython may accept or raise — mamba's to enforce)."""

class _W:
    pass


from xml.dom.expatbuilder import ExpatBuilder
obj = object.__new__(ExpatBuilder)
try:
    obj.parseString(_W())  # string: typed <- wrong-typed
    print("no_typeerror:")  # CPython accepted the wrong-typed arg; mamba must raise
except TypeError as e:
    print("typeerror:", type(e).__name__)
except Exception as e:
    print("setup_or_other:", type(e).__name__)
"###);
    assert!(out == "STRICT_TYPE_REJECTED" || out.starts_with("RUNTIME_REJECTED"),
        "type wall did not hold: {out}");
}

/// Ported from `tests/cpython/type/std-libs/xml_dom_expatbuilder/ExpatBuilder__pi_handler__target_as_str_wrong.py`.
#[test]
fn test_gen_type_std_libs_xml_dom_expatbuilder_ExpatBuilder__pi_handler__target_as_str_wrong() {
    let out = run_type_wall_fixture(r###"# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "xml_dom_expatbuilder"
# dimension = "type"
# case = "ExpatBuilder__pi_handler__target_as_str_wrong"
# subject = "xml.dom.expatbuilder.ExpatBuilder.pi_handler(target: str)"
# kind = "semantic"
# mem_carveout = ""
# source = "vendor/typeshed/stdlib/xml/dom/expatbuilder.pyi"
# status = "filled"
# ///
# mamba-strict-type: TypeError
"""Type wall: xml.dom.expatbuilder.ExpatBuilder.pi_handler(target: str); call it with the wrong type.

typeshed contract: target is str. mamba is force-typed, so a wrong-typed
argument MUST raise TypeError (CPython may accept or raise — mamba's to enforce)."""

from xml.dom.expatbuilder import ExpatBuilder
obj = object.__new__(ExpatBuilder)
try:
    obj.pi_handler(12345, "")  # target: str <- wrong-typed
    print("no_typeerror:")  # CPython accepted the wrong-typed arg; mamba must raise
except TypeError as e:
    print("typeerror:", type(e).__name__)
except Exception as e:
    print("setup_or_other:", type(e).__name__)
"###);
    assert!(out == "STRICT_TYPE_REJECTED" || out.starts_with("RUNTIME_REJECTED"),
        "type wall did not hold: {out}");
}

/// Ported from `tests/cpython/type/std-libs/xml_dom_expatbuilder/ExpatBuilder__start_doctype_decl_handler__doctypeName_as_str_wrong.py`.
#[test]
fn test_gen_type_std_libs_xml_dom_expatbuilder_ExpatBuilder__start_doctype_decl_handler__doctypeName_as_str_wrong() {
    let out = run_type_wall_fixture(r###"# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "xml_dom_expatbuilder"
# dimension = "type"
# case = "ExpatBuilder__start_doctype_decl_handler__doctypeName_as_str_wrong"
# subject = "xml.dom.expatbuilder.ExpatBuilder.start_doctype_decl_handler(doctypeName: str)"
# kind = "semantic"
# mem_carveout = ""
# source = "vendor/typeshed/stdlib/xml/dom/expatbuilder.pyi"
# status = "filled"
# ///
# mamba-strict-type: TypeError
"""Type wall: xml.dom.expatbuilder.ExpatBuilder.start_doctype_decl_handler(doctypeName: str); call it with the wrong type.

typeshed contract: doctypeName is str. mamba is force-typed, so a wrong-typed
argument MUST raise TypeError (CPython may accept or raise — mamba's to enforce)."""

from xml.dom.expatbuilder import ExpatBuilder
obj = object.__new__(ExpatBuilder)
try:
    obj.start_doctype_decl_handler(12345, None, None, True)  # doctypeName: str <- wrong-typed
    print("no_typeerror:")  # CPython accepted the wrong-typed arg; mamba must raise
except TypeError as e:
    print("typeerror:", type(e).__name__)
except Exception as e:
    print("setup_or_other:", type(e).__name__)
"###);
    assert!(out == "STRICT_TYPE_REJECTED" || out.starts_with("RUNTIME_REJECTED"),
        "type wall did not hold: {out}");
}

/// Ported from `tests/cpython/type/std-libs/xml_dom_expatbuilder/ExpatBuilder__start_element_handler__name_as_str_wrong.py`.
#[test]
fn test_gen_type_std_libs_xml_dom_expatbuilder_ExpatBuilder__start_element_handler__name_as_str_wrong() {
    let out = run_type_wall_fixture(r###"# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "xml_dom_expatbuilder"
# dimension = "type"
# case = "ExpatBuilder__start_element_handler__name_as_str_wrong"
# subject = "xml.dom.expatbuilder.ExpatBuilder.start_element_handler(name: str)"
# kind = "semantic"
# mem_carveout = ""
# source = "vendor/typeshed/stdlib/xml/dom/expatbuilder.pyi"
# status = "filled"
# ///
# mamba-strict-type: TypeError
"""Type wall: xml.dom.expatbuilder.ExpatBuilder.start_element_handler(name: str); call it with the wrong type.

typeshed contract: name is str. mamba is force-typed, so a wrong-typed
argument MUST raise TypeError (CPython may accept or raise — mamba's to enforce)."""

from xml.dom.expatbuilder import ExpatBuilder
obj = object.__new__(ExpatBuilder)
try:
    obj.start_element_handler(12345, None)  # name: str <- wrong-typed
    print("no_typeerror:")  # CPython accepted the wrong-typed arg; mamba must raise
except TypeError as e:
    print("typeerror:", type(e).__name__)
except Exception as e:
    print("setup_or_other:", type(e).__name__)
"###);
    assert!(out == "STRICT_TYPE_REJECTED" || out.starts_with("RUNTIME_REJECTED"),
        "type wall did not hold: {out}");
}

/// Ported from `tests/cpython/type/std-libs/xml_dom_expatbuilder/ExpatBuilder__xml_decl_handler__version_as_str_wrong.py`.
#[test]
fn test_gen_type_std_libs_xml_dom_expatbuilder_ExpatBuilder__xml_decl_handler__version_as_str_wrong() {
    let out = run_type_wall_fixture(r###"# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "xml_dom_expatbuilder"
# dimension = "type"
# case = "ExpatBuilder__xml_decl_handler__version_as_str_wrong"
# subject = "xml.dom.expatbuilder.ExpatBuilder.xml_decl_handler(version: str)"
# kind = "semantic"
# mem_carveout = ""
# source = "vendor/typeshed/stdlib/xml/dom/expatbuilder.pyi"
# status = "filled"
# ///
# mamba-strict-type: TypeError
"""Type wall: xml.dom.expatbuilder.ExpatBuilder.xml_decl_handler(version: str); call it with the wrong type.

typeshed contract: version is str. mamba is force-typed, so a wrong-typed
argument MUST raise TypeError (CPython may accept or raise — mamba's to enforce)."""

from xml.dom.expatbuilder import ExpatBuilder
obj = object.__new__(ExpatBuilder)
try:
    obj.xml_decl_handler(12345, None, 0)  # version: str <- wrong-typed
    print("no_typeerror:")  # CPython accepted the wrong-typed arg; mamba must raise
except TypeError as e:
    print("typeerror:", type(e).__name__)
except Exception as e:
    print("setup_or_other:", type(e).__name__)
"###);
    assert!(out == "STRICT_TYPE_REJECTED" || out.starts_with("RUNTIME_REJECTED"),
        "type wall did not hold: {out}");
}

/// Ported from `tests/cpython/type/std-libs/xml_dom_expatbuilder/FilterCrutch__init__builder_as_ExpatBuilder_wrong.py`.
#[test]
fn test_gen_type_std_libs_xml_dom_expatbuilder_FilterCrutch__init__builder_as_ExpatBuilder_wrong() {
    let out = run_type_wall_fixture(r###"# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "xml_dom_expatbuilder"
# dimension = "type"
# case = "FilterCrutch__init__builder_as_ExpatBuilder_wrong"
# subject = "xml.dom.expatbuilder.FilterCrutch.__init__(builder: ExpatBuilder)"
# kind = "semantic"
# mem_carveout = ""
# source = "vendor/typeshed/stdlib/xml/dom/expatbuilder.pyi"
# status = "filled"
# ///
# mamba-strict-type: TypeError
"""Type wall: xml.dom.expatbuilder.FilterCrutch.__init__(builder: ExpatBuilder); call it with the wrong type.

typeshed contract: builder is ExpatBuilder. mamba is force-typed, so a wrong-typed
argument MUST raise TypeError (CPython may accept or raise — mamba's to enforce)."""

class _W:
    pass


from xml.dom.expatbuilder import FilterCrutch
try:
    FilterCrutch(_W())  # builder: ExpatBuilder <- wrong-typed
    print("no_typeerror:")  # CPython accepted the wrong-typed arg; mamba must raise
except TypeError as e:
    print("typeerror:", type(e).__name__)
except Exception as e:
    print("setup_or_other:", type(e).__name__)
"###);
    assert!(out == "STRICT_TYPE_REJECTED" || out.starts_with("RUNTIME_REJECTED"),
        "type wall did not hold: {out}");
}

/// Ported from `tests/cpython/type/std-libs/xml_dom_expatbuilder/FilterVisibilityController__acceptNode__node_as_Node_wrong.py`.
#[test]
fn test_gen_type_std_libs_xml_dom_expatbuilder_FilterVisibilityController__acceptNode__node_as_Node_wrong() {
    let out = run_type_wall_fixture(r###"# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "xml_dom_expatbuilder"
# dimension = "type"
# case = "FilterVisibilityController__acceptNode__node_as_Node_wrong"
# subject = "xml.dom.expatbuilder.FilterVisibilityController.acceptNode(node: Node)"
# kind = "semantic"
# mem_carveout = ""
# source = "vendor/typeshed/stdlib/xml/dom/expatbuilder.pyi"
# status = "filled"
# ///
# mamba-strict-type: TypeError
"""Type wall: xml.dom.expatbuilder.FilterVisibilityController.acceptNode(node: Node); call it with the wrong type.

typeshed contract: node is Node. mamba is force-typed, so a wrong-typed
argument MUST raise TypeError (CPython may accept or raise — mamba's to enforce)."""

class _W:
    pass


from xml.dom.expatbuilder import FilterVisibilityController
obj = object.__new__(FilterVisibilityController)
try:
    obj.acceptNode(_W())  # node: Node <- wrong-typed
    print("no_typeerror:")  # CPython accepted the wrong-typed arg; mamba must raise
except TypeError as e:
    print("typeerror:", type(e).__name__)
except Exception as e:
    print("setup_or_other:", type(e).__name__)
"###);
    assert!(out == "STRICT_TYPE_REJECTED" || out.starts_with("RUNTIME_REJECTED"),
        "type wall did not hold: {out}");
}

/// Ported from `tests/cpython/type/std-libs/xml_dom_expatbuilder/FilterVisibilityController__init__filter_as_DOMBuilderFilter_wrong.py`.
#[test]
fn test_gen_type_std_libs_xml_dom_expatbuilder_FilterVisibilityController__init__filter_as_DOMBuilderFilter_wrong() {
    let out = run_type_wall_fixture(r###"# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "xml_dom_expatbuilder"
# dimension = "type"
# case = "FilterVisibilityController__init__filter_as_DOMBuilderFilter_wrong"
# subject = "xml.dom.expatbuilder.FilterVisibilityController.__init__(filter: DOMBuilderFilter)"
# kind = "semantic"
# mem_carveout = ""
# source = "vendor/typeshed/stdlib/xml/dom/expatbuilder.pyi"
# status = "filled"
# ///
# mamba-strict-type: TypeError
"""Type wall: xml.dom.expatbuilder.FilterVisibilityController.__init__(filter: DOMBuilderFilter); call it with the wrong type.

typeshed contract: filter is DOMBuilderFilter. mamba is force-typed, so a wrong-typed
argument MUST raise TypeError (CPython may accept or raise — mamba's to enforce)."""

class _W:
    pass


from xml.dom.expatbuilder import FilterVisibilityController
try:
    FilterVisibilityController(_W())  # filter: DOMBuilderFilter <- wrong-typed
    print("no_typeerror:")  # CPython accepted the wrong-typed arg; mamba must raise
except TypeError as e:
    print("typeerror:", type(e).__name__)
except Exception as e:
    print("setup_or_other:", type(e).__name__)
"###);
    assert!(out == "STRICT_TYPE_REJECTED" || out.starts_with("RUNTIME_REJECTED"),
        "type wall did not hold: {out}");
}

/// Ported from `tests/cpython/type/std-libs/xml_dom_expatbuilder/FilterVisibilityController__startContainer__node_as_Node_wrong.py`.
#[test]
fn test_gen_type_std_libs_xml_dom_expatbuilder_FilterVisibilityController__startContainer__node_as_Node_wrong() {
    let out = run_type_wall_fixture(r###"# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "xml_dom_expatbuilder"
# dimension = "type"
# case = "FilterVisibilityController__startContainer__node_as_Node_wrong"
# subject = "xml.dom.expatbuilder.FilterVisibilityController.startContainer(node: Node)"
# kind = "semantic"
# mem_carveout = ""
# source = "vendor/typeshed/stdlib/xml/dom/expatbuilder.pyi"
# status = "filled"
# ///
# mamba-strict-type: TypeError
"""Type wall: xml.dom.expatbuilder.FilterVisibilityController.startContainer(node: Node); call it with the wrong type.

typeshed contract: node is Node. mamba is force-typed, so a wrong-typed
argument MUST raise TypeError (CPython may accept or raise — mamba's to enforce)."""

class _W:
    pass


from xml.dom.expatbuilder import FilterVisibilityController
obj = object.__new__(FilterVisibilityController)
try:
    obj.startContainer(_W())  # node: Node <- wrong-typed
    print("no_typeerror:")  # CPython accepted the wrong-typed arg; mamba must raise
except TypeError as e:
    print("typeerror:", type(e).__name__)
except Exception as e:
    print("setup_or_other:", type(e).__name__)
"###);
    assert!(out == "STRICT_TYPE_REJECTED" || out.starts_with("RUNTIME_REJECTED"),
        "type wall did not hold: {out}");
}

/// Ported from `tests/cpython/type/std-libs/xml_dom_expatbuilder/FragmentBuilder__external_entity_ref_handler__context_as_str_wrong.py`.
#[test]
fn test_gen_type_std_libs_xml_dom_expatbuilder_FragmentBuilder__external_entity_ref_handler__context_as_str_wrong() {
    let out = run_type_wall_fixture(r###"# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "xml_dom_expatbuilder"
# dimension = "type"
# case = "FragmentBuilder__external_entity_ref_handler__context_as_str_wrong"
# subject = "xml.dom.expatbuilder.FragmentBuilder.external_entity_ref_handler(context: str)"
# kind = "semantic"
# mem_carveout = ""
# source = "vendor/typeshed/stdlib/xml/dom/expatbuilder.pyi"
# status = "filled"
# ///
# mamba-strict-type: TypeError
"""Type wall: xml.dom.expatbuilder.FragmentBuilder.external_entity_ref_handler(context: str); call it with the wrong type.

typeshed contract: context is str. mamba is force-typed, so a wrong-typed
argument MUST raise TypeError (CPython may accept or raise — mamba's to enforce)."""

from xml.dom.expatbuilder import FragmentBuilder
obj = object.__new__(FragmentBuilder)
try:
    obj.external_entity_ref_handler(12345, None, None, None)  # context: str <- wrong-typed
    print("no_typeerror:")  # CPython accepted the wrong-typed arg; mamba must raise
except TypeError as e:
    print("typeerror:", type(e).__name__)
except Exception as e:
    print("setup_or_other:", type(e).__name__)
"###);
    assert!(out == "STRICT_TYPE_REJECTED" || out.starts_with("RUNTIME_REJECTED"),
        "type wall did not hold: {out}");
}

/// Ported from `tests/cpython/type/std-libs/xml_dom_expatbuilder/FragmentBuilder__init__context_as_Node_wrong.py`.
#[test]
fn test_gen_type_std_libs_xml_dom_expatbuilder_FragmentBuilder__init__context_as_Node_wrong() {
    let out = run_type_wall_fixture(r###"# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "xml_dom_expatbuilder"
# dimension = "type"
# case = "FragmentBuilder__init__context_as_Node_wrong"
# subject = "xml.dom.expatbuilder.FragmentBuilder.__init__(context: Node)"
# kind = "semantic"
# mem_carveout = ""
# source = "vendor/typeshed/stdlib/xml/dom/expatbuilder.pyi"
# status = "filled"
# ///
# mamba-strict-type: TypeError
"""Type wall: xml.dom.expatbuilder.FragmentBuilder.__init__(context: Node); call it with the wrong type.

typeshed contract: context is Node. mamba is force-typed, so a wrong-typed
argument MUST raise TypeError (CPython may accept or raise — mamba's to enforce)."""

class _W:
    pass


from xml.dom.expatbuilder import FragmentBuilder
try:
    FragmentBuilder(_W())  # context: Node <- wrong-typed
    print("no_typeerror:")  # CPython accepted the wrong-typed arg; mamba must raise
except TypeError as e:
    print("typeerror:", type(e).__name__)
except Exception as e:
    print("setup_or_other:", type(e).__name__)
"###);
    assert!(out == "STRICT_TYPE_REJECTED" || out.starts_with("RUNTIME_REJECTED"),
        "type wall did not hold: {out}");
}

/// Ported from `tests/cpython/type/std-libs/xml_dom_expatbuilder/FragmentBuilder__parseFile__file_as_SupportsRead_wrong.py`.
#[test]
fn test_gen_type_std_libs_xml_dom_expatbuilder_FragmentBuilder__parseFile__file_as_SupportsRead_wrong() {
    let out = run_type_wall_fixture(r###"# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "xml_dom_expatbuilder"
# dimension = "type"
# case = "FragmentBuilder__parseFile__file_as_SupportsRead_wrong"
# subject = "xml.dom.expatbuilder.FragmentBuilder.parseFile(file: SupportsRead)"
# kind = "semantic"
# mem_carveout = ""
# source = "vendor/typeshed/stdlib/xml/dom/expatbuilder.pyi"
# status = "filled"
# ///
# mamba-strict-type: TypeError
"""Type wall: xml.dom.expatbuilder.FragmentBuilder.parseFile(file: SupportsRead); call it with the wrong type.

typeshed contract: file is SupportsRead. mamba is force-typed, so a wrong-typed
argument MUST raise TypeError (CPython may accept or raise — mamba's to enforce)."""

class _W:
    pass


from xml.dom.expatbuilder import FragmentBuilder
obj = object.__new__(FragmentBuilder)
try:
    obj.parseFile(_W())  # file: SupportsRead <- wrong-typed
    print("no_typeerror:")  # CPython accepted the wrong-typed arg; mamba must raise
except TypeError as e:
    print("typeerror:", type(e).__name__)
except Exception as e:
    print("setup_or_other:", type(e).__name__)
"###);
    assert!(out == "STRICT_TYPE_REJECTED" || out.starts_with("RUNTIME_REJECTED"),
        "type wall did not hold: {out}");
}

/// Ported from `tests/cpython/type/std-libs/xml_dom_expatbuilder/FragmentBuilder__parseString__string_as_typed_wrong.py`.
#[test]
fn test_gen_type_std_libs_xml_dom_expatbuilder_FragmentBuilder__parseString__string_as_typed_wrong() {
    let out = run_type_wall_fixture(r###"# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "xml_dom_expatbuilder"
# dimension = "type"
# case = "FragmentBuilder__parseString__string_as_typed_wrong"
# subject = "xml.dom.expatbuilder.FragmentBuilder.parseString(string: typed)"
# kind = "semantic"
# mem_carveout = ""
# source = "vendor/typeshed/stdlib/xml/dom/expatbuilder.pyi"
# status = "filled"
# ///
# mamba-strict-type: TypeError
"""Type wall: xml.dom.expatbuilder.FragmentBuilder.parseString(string: typed); call it with the wrong type.

typeshed contract: string is typed. mamba is force-typed, so a wrong-typed
argument MUST raise TypeError (CPython may accept or raise — mamba's to enforce)."""

class _W:
    pass


from xml.dom.expatbuilder import FragmentBuilder
obj = object.__new__(FragmentBuilder)
try:
    obj.parseString(_W())  # string: typed <- wrong-typed
    print("no_typeerror:")  # CPython accepted the wrong-typed arg; mamba must raise
except TypeError as e:
    print("typeerror:", type(e).__name__)
except Exception as e:
    print("setup_or_other:", type(e).__name__)
"###);
    assert!(out == "STRICT_TYPE_REJECTED" || out.starts_with("RUNTIME_REJECTED"),
        "type wall did not hold: {out}");
}

/// Ported from `tests/cpython/type/std-libs/xml_dom_expatbuilder/InternalSubsetExtractor__parseFile__file_as_SupportsRead_wrong.py`.
#[test]
fn test_gen_type_std_libs_xml_dom_expatbuilder_InternalSubsetExtractor__parseFile__file_as_SupportsRead_wrong() {
    let out = run_type_wall_fixture(r###"# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "xml_dom_expatbuilder"
# dimension = "type"
# case = "InternalSubsetExtractor__parseFile__file_as_SupportsRead_wrong"
# subject = "xml.dom.expatbuilder.InternalSubsetExtractor.parseFile(file: SupportsRead)"
# kind = "semantic"
# mem_carveout = ""
# source = "vendor/typeshed/stdlib/xml/dom/expatbuilder.pyi"
# status = "filled"
# ///
# mamba-strict-type: TypeError
"""Type wall: xml.dom.expatbuilder.InternalSubsetExtractor.parseFile(file: SupportsRead); call it with the wrong type.

typeshed contract: file is SupportsRead. mamba is force-typed, so a wrong-typed
argument MUST raise TypeError (CPython may accept or raise — mamba's to enforce)."""

class _W:
    pass


from xml.dom.expatbuilder import InternalSubsetExtractor
obj = object.__new__(InternalSubsetExtractor)
try:
    obj.parseFile(_W())  # file: SupportsRead <- wrong-typed
    print("no_typeerror:")  # CPython accepted the wrong-typed arg; mamba must raise
except TypeError as e:
    print("typeerror:", type(e).__name__)
except Exception as e:
    print("setup_or_other:", type(e).__name__)
"###);
    assert!(out == "STRICT_TYPE_REJECTED" || out.starts_with("RUNTIME_REJECTED"),
        "type wall did not hold: {out}");
}

/// Ported from `tests/cpython/type/std-libs/xml_dom_expatbuilder/InternalSubsetExtractor__parseString__string_as_typed_wrong.py`.
#[test]
fn test_gen_type_std_libs_xml_dom_expatbuilder_InternalSubsetExtractor__parseString__string_as_typed_wrong() {
    let out = run_type_wall_fixture(r###"# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "xml_dom_expatbuilder"
# dimension = "type"
# case = "InternalSubsetExtractor__parseString__string_as_typed_wrong"
# subject = "xml.dom.expatbuilder.InternalSubsetExtractor.parseString(string: typed)"
# kind = "semantic"
# mem_carveout = ""
# source = "vendor/typeshed/stdlib/xml/dom/expatbuilder.pyi"
# status = "filled"
# ///
# mamba-strict-type: TypeError
"""Type wall: xml.dom.expatbuilder.InternalSubsetExtractor.parseString(string: typed); call it with the wrong type.

typeshed contract: string is typed. mamba is force-typed, so a wrong-typed
argument MUST raise TypeError (CPython may accept or raise — mamba's to enforce)."""

class _W:
    pass


from xml.dom.expatbuilder import InternalSubsetExtractor
obj = object.__new__(InternalSubsetExtractor)
try:
    obj.parseString(_W())  # string: typed <- wrong-typed
    print("no_typeerror:")  # CPython accepted the wrong-typed arg; mamba must raise
except TypeError as e:
    print("typeerror:", type(e).__name__)
except Exception as e:
    print("setup_or_other:", type(e).__name__)
"###);
    assert!(out == "STRICT_TYPE_REJECTED" || out.starts_with("RUNTIME_REJECTED"),
        "type wall did not hold: {out}");
}

/// Ported from `tests/cpython/type/std-libs/xml_dom_expatbuilder/InternalSubsetExtractor__start_doctype_decl_handler__name_as_str_wrong.py`.
#[test]
fn test_gen_type_std_libs_xml_dom_expatbuilder_InternalSubsetExtractor__start_doctype_decl_handler__name_as_str_wrong() {
    let out = run_type_wall_fixture(r###"# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "xml_dom_expatbuilder"
# dimension = "type"
# case = "InternalSubsetExtractor__start_doctype_decl_handler__name_as_str_wrong"
# subject = "xml.dom.expatbuilder.InternalSubsetExtractor.start_doctype_decl_handler(name: str)"
# kind = "semantic"
# mem_carveout = ""
# source = "vendor/typeshed/stdlib/xml/dom/expatbuilder.pyi"
# status = "filled"
# ///
# mamba-strict-type: TypeError
"""Type wall: xml.dom.expatbuilder.InternalSubsetExtractor.start_doctype_decl_handler(name: str); call it with the wrong type.

typeshed contract: name is str. mamba is force-typed, so a wrong-typed
argument MUST raise TypeError (CPython may accept or raise — mamba's to enforce)."""

from xml.dom.expatbuilder import InternalSubsetExtractor
obj = object.__new__(InternalSubsetExtractor)
try:
    obj.start_doctype_decl_handler(12345, None, None, True)  # name: str <- wrong-typed
    print("no_typeerror:")  # CPython accepted the wrong-typed arg; mamba must raise
except TypeError as e:
    print("typeerror:", type(e).__name__)
except Exception as e:
    print("setup_or_other:", type(e).__name__)
"###);
    assert!(out == "STRICT_TYPE_REJECTED" || out.starts_with("RUNTIME_REJECTED"),
        "type wall did not hold: {out}");
}

/// Ported from `tests/cpython/type/std-libs/xml_dom_expatbuilder/InternalSubsetExtractor__start_element_handler__name_as_str_wrong.py`.
#[test]
fn test_gen_type_std_libs_xml_dom_expatbuilder_InternalSubsetExtractor__start_element_handler__name_as_str_wrong() {
    let out = run_type_wall_fixture(r###"# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "xml_dom_expatbuilder"
# dimension = "type"
# case = "InternalSubsetExtractor__start_element_handler__name_as_str_wrong"
# subject = "xml.dom.expatbuilder.InternalSubsetExtractor.start_element_handler(name: str)"
# kind = "semantic"
# mem_carveout = ""
# source = "vendor/typeshed/stdlib/xml/dom/expatbuilder.pyi"
# status = "filled"
# ///
# mamba-strict-type: TypeError
"""Type wall: xml.dom.expatbuilder.InternalSubsetExtractor.start_element_handler(name: str); call it with the wrong type.

typeshed contract: name is str. mamba is force-typed, so a wrong-typed
argument MUST raise TypeError (CPython may accept or raise — mamba's to enforce)."""

from xml.dom.expatbuilder import InternalSubsetExtractor
obj = object.__new__(InternalSubsetExtractor)
try:
    obj.start_element_handler(12345, None)  # name: str <- wrong-typed
    print("no_typeerror:")  # CPython accepted the wrong-typed arg; mamba must raise
except TypeError as e:
    print("typeerror:", type(e).__name__)
except Exception as e:
    print("setup_or_other:", type(e).__name__)
"###);
    assert!(out == "STRICT_TYPE_REJECTED" || out.starts_with("RUNTIME_REJECTED"),
        "type wall did not hold: {out}");
}

/// Ported from `tests/cpython/type/std-libs/xml_dom_expatbuilder/Namespaces__end_element_handler__name_as_str_wrong.py`.
#[test]
fn test_gen_type_std_libs_xml_dom_expatbuilder_Namespaces__end_element_handler__name_as_str_wrong() {
    let out = run_type_wall_fixture(r###"# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "xml_dom_expatbuilder"
# dimension = "type"
# case = "Namespaces__end_element_handler__name_as_str_wrong"
# subject = "xml.dom.expatbuilder.Namespaces.end_element_handler(name: str)"
# kind = "semantic"
# mem_carveout = ""
# source = "vendor/typeshed/stdlib/xml/dom/expatbuilder.pyi"
# status = "filled"
# ///
# mamba-strict-type: TypeError
"""Type wall: xml.dom.expatbuilder.Namespaces.end_element_handler(name: str); call it with the wrong type.

typeshed contract: name is str. mamba is force-typed, so a wrong-typed
argument MUST raise TypeError (CPython may accept or raise — mamba's to enforce)."""

from xml.dom.expatbuilder import Namespaces
obj = object.__new__(Namespaces)
try:
    obj.end_element_handler(12345)  # name: str <- wrong-typed
    print("no_typeerror:")  # CPython accepted the wrong-typed arg; mamba must raise
except TypeError as e:
    print("typeerror:", type(e).__name__)
except Exception as e:
    print("setup_or_other:", type(e).__name__)
"###);
    assert!(out == "STRICT_TYPE_REJECTED" || out.starts_with("RUNTIME_REJECTED"),
        "type wall did not hold: {out}");
}

/// Ported from `tests/cpython/type/std-libs/xml_dom_expatbuilder/Namespaces__install__parser_as_XMLParserType_wrong.py`.
#[test]
fn test_gen_type_std_libs_xml_dom_expatbuilder_Namespaces__install__parser_as_XMLParserType_wrong() {
    let out = run_type_wall_fixture(r###"# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "xml_dom_expatbuilder"
# dimension = "type"
# case = "Namespaces__install__parser_as_XMLParserType_wrong"
# subject = "xml.dom.expatbuilder.Namespaces.install(parser: XMLParserType)"
# kind = "semantic"
# mem_carveout = ""
# source = "vendor/typeshed/stdlib/xml/dom/expatbuilder.pyi"
# status = "filled"
# ///
# mamba-strict-type: TypeError
"""Type wall: xml.dom.expatbuilder.Namespaces.install(parser: XMLParserType); call it with the wrong type.

typeshed contract: parser is XMLParserType. mamba is force-typed, so a wrong-typed
argument MUST raise TypeError (CPython may accept or raise — mamba's to enforce)."""

class _W:
    pass


from xml.dom.expatbuilder import Namespaces
obj = object.__new__(Namespaces)
try:
    obj.install(_W())  # parser: XMLParserType <- wrong-typed
    print("no_typeerror:")  # CPython accepted the wrong-typed arg; mamba must raise
except TypeError as e:
    print("typeerror:", type(e).__name__)
except Exception as e:
    print("setup_or_other:", type(e).__name__)
"###);
    assert!(out == "STRICT_TYPE_REJECTED" || out.starts_with("RUNTIME_REJECTED"),
        "type wall did not hold: {out}");
}

/// Ported from `tests/cpython/type/std-libs/xml_dom_expatbuilder/Namespaces__start_element_handler__name_as_str_wrong.py`.
#[test]
fn test_gen_type_std_libs_xml_dom_expatbuilder_Namespaces__start_element_handler__name_as_str_wrong() {
    let out = run_type_wall_fixture(r###"# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "xml_dom_expatbuilder"
# dimension = "type"
# case = "Namespaces__start_element_handler__name_as_str_wrong"
# subject = "xml.dom.expatbuilder.Namespaces.start_element_handler(name: str)"
# kind = "semantic"
# mem_carveout = ""
# source = "vendor/typeshed/stdlib/xml/dom/expatbuilder.pyi"
# status = "filled"
# ///
# mamba-strict-type: TypeError
"""Type wall: xml.dom.expatbuilder.Namespaces.start_element_handler(name: str); call it with the wrong type.

typeshed contract: name is str. mamba is force-typed, so a wrong-typed
argument MUST raise TypeError (CPython may accept or raise — mamba's to enforce)."""

from xml.dom.expatbuilder import Namespaces
obj = object.__new__(Namespaces)
try:
    obj.start_element_handler(12345, None)  # name: str <- wrong-typed
    print("no_typeerror:")  # CPython accepted the wrong-typed arg; mamba must raise
except TypeError as e:
    print("typeerror:", type(e).__name__)
except Exception as e:
    print("setup_or_other:", type(e).__name__)
"###);
    assert!(out == "STRICT_TYPE_REJECTED" || out.starts_with("RUNTIME_REJECTED"),
        "type wall did not hold: {out}");
}

/// Ported from `tests/cpython/type/std-libs/xml_dom_expatbuilder/Namespaces__start_namespace_decl_handler__prefix_as_typed_wrong.py`.
#[test]
fn test_gen_type_std_libs_xml_dom_expatbuilder_Namespaces__start_namespace_decl_handler__prefix_as_typed_wrong() {
    let out = run_type_wall_fixture(r###"# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "xml_dom_expatbuilder"
# dimension = "type"
# case = "Namespaces__start_namespace_decl_handler__prefix_as_typed_wrong"
# subject = "xml.dom.expatbuilder.Namespaces.start_namespace_decl_handler(prefix: typed)"
# kind = "semantic"
# mem_carveout = ""
# source = "vendor/typeshed/stdlib/xml/dom/expatbuilder.pyi"
# status = "filled"
# ///
# mamba-strict-type: TypeError
"""Type wall: xml.dom.expatbuilder.Namespaces.start_namespace_decl_handler(prefix: typed); call it with the wrong type.

typeshed contract: prefix is typed. mamba is force-typed, so a wrong-typed
argument MUST raise TypeError (CPython may accept or raise — mamba's to enforce)."""

class _W:
    pass


from xml.dom.expatbuilder import Namespaces
obj = object.__new__(Namespaces)
try:
    obj.start_namespace_decl_handler(_W(), "")  # prefix: typed <- wrong-typed
    print("no_typeerror:")  # CPython accepted the wrong-typed arg; mamba must raise
except TypeError as e:
    print("typeerror:", type(e).__name__)
except Exception as e:
    print("setup_or_other:", type(e).__name__)
"###);
    assert!(out == "STRICT_TYPE_REJECTED" || out.starts_with("RUNTIME_REJECTED"),
        "type wall did not hold: {out}");
}

/// Ported from `tests/cpython/type/std-libs/xml_dom_expatbuilder/makeBuilder__options_as_Options_wrong.py`.
#[test]
fn test_gen_type_std_libs_xml_dom_expatbuilder_makeBuilder__options_as_Options_wrong() {
    let out = run_type_wall_fixture(r###"# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "xml_dom_expatbuilder"
# dimension = "type"
# case = "makeBuilder__options_as_Options_wrong"
# subject = "xml.dom.expatbuilder.makeBuilder(options: Options)"
# kind = "semantic"
# mem_carveout = ""
# source = "vendor/typeshed/stdlib/xml/dom/expatbuilder.pyi"
# status = "filled"
# ///
# mamba-strict-type: TypeError
"""Type wall: xml.dom.expatbuilder.makeBuilder(options: Options); call it with the wrong type.

typeshed contract: options is Options. mamba is force-typed, so a wrong-typed
argument MUST raise TypeError (CPython may accept or raise — mamba's to enforce)."""

class _W:
    pass


from xml.dom.expatbuilder import makeBuilder
try:
    makeBuilder(_W())  # options: Options <- wrong-typed
    print("no_typeerror:")  # CPython accepted the wrong-typed arg; mamba must raise
except TypeError as e:
    print("typeerror:", type(e).__name__)
except Exception as e:
    print("setup_or_other:", type(e).__name__)
"###);
    assert!(out == "STRICT_TYPE_REJECTED" || out.starts_with("RUNTIME_REJECTED"),
        "type wall did not hold: {out}");
}

/// Ported from `tests/cpython/type/std-libs/xml_dom_expatbuilder/parseFragmentString__string_as_typed_wrong.py`.
#[test]
fn test_gen_type_std_libs_xml_dom_expatbuilder_parseFragmentString__string_as_typed_wrong() {
    let out = run_type_wall_fixture(r###"# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "xml_dom_expatbuilder"
# dimension = "type"
# case = "parseFragmentString__string_as_typed_wrong"
# subject = "xml.dom.expatbuilder.parseFragmentString(string: typed)"
# kind = "semantic"
# mem_carveout = ""
# source = "vendor/typeshed/stdlib/xml/dom/expatbuilder.pyi"
# status = "filled"
# ///
# mamba-strict-type: TypeError
"""Type wall: xml.dom.expatbuilder.parseFragmentString(string: typed); call it with the wrong type.

typeshed contract: string is typed. mamba is force-typed, so a wrong-typed
argument MUST raise TypeError (CPython may accept or raise — mamba's to enforce)."""

class _W:
    pass


from xml.dom.expatbuilder import parseFragmentString
try:
    parseFragmentString(_W(), None)  # string: typed <- wrong-typed
    print("no_typeerror:")  # CPython accepted the wrong-typed arg; mamba must raise
except TypeError as e:
    print("typeerror:", type(e).__name__)
except Exception as e:
    print("setup_or_other:", type(e).__name__)
"###);
    assert!(out == "STRICT_TYPE_REJECTED" || out.starts_with("RUNTIME_REJECTED"),
        "type wall did not hold: {out}");
}

/// Ported from `tests/cpython/type/std-libs/xml_dom_expatbuilder/parseFragment__file_as_typed_wrong.py`.
#[test]
fn test_gen_type_std_libs_xml_dom_expatbuilder_parseFragment__file_as_typed_wrong() {
    let out = run_type_wall_fixture(r###"# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "xml_dom_expatbuilder"
# dimension = "type"
# case = "parseFragment__file_as_typed_wrong"
# subject = "xml.dom.expatbuilder.parseFragment(file: typed)"
# kind = "semantic"
# mem_carveout = ""
# source = "vendor/typeshed/stdlib/xml/dom/expatbuilder.pyi"
# status = "filled"
# ///
# mamba-strict-type: TypeError
"""Type wall: xml.dom.expatbuilder.parseFragment(file: typed); call it with the wrong type.

typeshed contract: file is typed. mamba is force-typed, so a wrong-typed
argument MUST raise TypeError (CPython may accept or raise — mamba's to enforce)."""

class _W:
    pass


from xml.dom.expatbuilder import parseFragment
try:
    parseFragment(_W(), None)  # file: typed <- wrong-typed
    print("no_typeerror:")  # CPython accepted the wrong-typed arg; mamba must raise
except TypeError as e:
    print("typeerror:", type(e).__name__)
except Exception as e:
    print("setup_or_other:", type(e).__name__)
"###);
    assert!(out == "STRICT_TYPE_REJECTED" || out.starts_with("RUNTIME_REJECTED"),
        "type wall did not hold: {out}");
}

/// Ported from `tests/cpython/type/std-libs/xml_dom_expatbuilder/parseString__string_as_typed_wrong.py`.
#[test]
fn test_gen_type_std_libs_xml_dom_expatbuilder_parseString__string_as_typed_wrong() {
    let out = run_type_wall_fixture(r###"# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "xml_dom_expatbuilder"
# dimension = "type"
# case = "parseString__string_as_typed_wrong"
# subject = "xml.dom.expatbuilder.parseString(string: typed)"
# kind = "semantic"
# mem_carveout = ""
# source = "vendor/typeshed/stdlib/xml/dom/expatbuilder.pyi"
# status = "filled"
# ///
# mamba-strict-type: TypeError
"""Type wall: xml.dom.expatbuilder.parseString(string: typed); call it with the wrong type.

typeshed contract: string is typed. mamba is force-typed, so a wrong-typed
argument MUST raise TypeError (CPython may accept or raise — mamba's to enforce)."""

class _W:
    pass


from xml.dom.expatbuilder import parseString
try:
    parseString(_W())  # string: typed <- wrong-typed
    print("no_typeerror:")  # CPython accepted the wrong-typed arg; mamba must raise
except TypeError as e:
    print("typeerror:", type(e).__name__)
except Exception as e:
    print("setup_or_other:", type(e).__name__)
"###);
    assert!(out == "STRICT_TYPE_REJECTED" || out.starts_with("RUNTIME_REJECTED"),
        "type wall did not hold: {out}");
}

/// Ported from `tests/cpython/type/std-libs/xml_dom_expatbuilder/parse__file_as_typed_wrong.py`.
#[test]
fn test_gen_type_std_libs_xml_dom_expatbuilder_parse__file_as_typed_wrong() {
    let out = run_type_wall_fixture(r###"# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "xml_dom_expatbuilder"
# dimension = "type"
# case = "parse__file_as_typed_wrong"
# subject = "xml.dom.expatbuilder.parse(file: typed)"
# kind = "semantic"
# mem_carveout = ""
# source = "vendor/typeshed/stdlib/xml/dom/expatbuilder.pyi"
# status = "filled"
# ///
# mamba-strict-type: TypeError
"""Type wall: xml.dom.expatbuilder.parse(file: typed); call it with the wrong type.

typeshed contract: file is typed. mamba is force-typed, so a wrong-typed
argument MUST raise TypeError (CPython may accept or raise — mamba's to enforce)."""

class _W:
    pass


from xml.dom.expatbuilder import parse
try:
    parse(_W())  # file: typed <- wrong-typed
    print("no_typeerror:")  # CPython accepted the wrong-typed arg; mamba must raise
except TypeError as e:
    print("typeerror:", type(e).__name__)
except Exception as e:
    print("setup_or_other:", type(e).__name__)
"###);
    assert!(out == "STRICT_TYPE_REJECTED" || out.starts_with("RUNTIME_REJECTED"),
        "type wall did not hold: {out}");
}
