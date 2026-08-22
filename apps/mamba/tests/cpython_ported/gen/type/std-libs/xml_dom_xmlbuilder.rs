use super::super::super::super::harness::*;

/// Ported from `tests/cpython/type/std-libs/xml_dom_xmlbuilder/DOMBuilderFilter__acceptNode__element_as_Node_wrong.py`.
#[test]
fn test_gen_type_std_libs_xml_dom_xmlbuilder_DOMBuilderFilter__acceptNode__element_as_Node_wrong() {
    let out = run_type_wall_fixture(r###"# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "xml_dom_xmlbuilder"
# dimension = "type"
# case = "DOMBuilderFilter__acceptNode__element_as_Node_wrong"
# subject = "xml.dom.xmlbuilder.DOMBuilderFilter.acceptNode(element: Node)"
# kind = "semantic"
# xfail = ""
# mem_carveout = ""
# source = "vendor/typeshed/stdlib/xml/dom/xmlbuilder.pyi"
# status = "filled"
# ///
# mamba-strict-type: TypeError
"""Type wall: xml.dom.xmlbuilder.DOMBuilderFilter.acceptNode(element: Node); call it with the wrong type.

typeshed contract: element is Node. mamba is force-typed, so a wrong-typed
argument MUST raise TypeError (CPython may accept or raise — mamba's to enforce)."""

class _W:
    pass


from xml.dom.xmlbuilder import DOMBuilderFilter
obj = object.__new__(DOMBuilderFilter)
try:
    obj.acceptNode(_W())  # element: Node <- wrong-typed
    print("no_typeerror:")  # CPython accepted the wrong-typed arg; mamba must raise
except TypeError as e:
    print("typeerror:", type(e).__name__)
except Exception as e:
    print("setup_or_other:", type(e).__name__)
"###);
    assert!(out == "STRICT_TYPE_REJECTED" || out.starts_with("RUNTIME_REJECTED"),
        "type wall did not hold: {out}");
}

/// Ported from `tests/cpython/type/std-libs/xml_dom_xmlbuilder/DOMBuilderFilter__startContainer__element_as_Node_wrong.py`.
#[test]
fn test_gen_type_std_libs_xml_dom_xmlbuilder_DOMBuilderFilter__startContainer__element_as_Node_wrong() {
    let out = run_type_wall_fixture(r###"# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "xml_dom_xmlbuilder"
# dimension = "type"
# case = "DOMBuilderFilter__startContainer__element_as_Node_wrong"
# subject = "xml.dom.xmlbuilder.DOMBuilderFilter.startContainer(element: Node)"
# kind = "semantic"
# xfail = ""
# mem_carveout = ""
# source = "vendor/typeshed/stdlib/xml/dom/xmlbuilder.pyi"
# status = "filled"
# ///
# mamba-strict-type: TypeError
"""Type wall: xml.dom.xmlbuilder.DOMBuilderFilter.startContainer(element: Node); call it with the wrong type.

typeshed contract: element is Node. mamba is force-typed, so a wrong-typed
argument MUST raise TypeError (CPython may accept or raise — mamba's to enforce)."""

class _W:
    pass


from xml.dom.xmlbuilder import DOMBuilderFilter
obj = object.__new__(DOMBuilderFilter)
try:
    obj.startContainer(_W())  # element: Node <- wrong-typed
    print("no_typeerror:")  # CPython accepted the wrong-typed arg; mamba must raise
except TypeError as e:
    print("typeerror:", type(e).__name__)
except Exception as e:
    print("setup_or_other:", type(e).__name__)
"###);
    assert!(out == "STRICT_TYPE_REJECTED" || out.starts_with("RUNTIME_REJECTED"),
        "type wall did not hold: {out}");
}

/// Ported from `tests/cpython/type/std-libs/xml_dom_xmlbuilder/DOMBuilder__canSetFeature__name_as_str_wrong.py`.
#[test]
fn test_gen_type_std_libs_xml_dom_xmlbuilder_DOMBuilder__canSetFeature__name_as_str_wrong() {
    let out = run_type_wall_fixture(r###"# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "xml_dom_xmlbuilder"
# dimension = "type"
# case = "DOMBuilder__canSetFeature__name_as_str_wrong"
# subject = "xml.dom.xmlbuilder.DOMBuilder.canSetFeature(name: str)"
# kind = "semantic"
# xfail = ""
# mem_carveout = ""
# source = "vendor/typeshed/stdlib/xml/dom/xmlbuilder.pyi"
# status = "filled"
# ///
# mamba-strict-type: TypeError
"""Type wall: xml.dom.xmlbuilder.DOMBuilder.canSetFeature(name: str); call it with the wrong type.

typeshed contract: name is str. mamba is force-typed, so a wrong-typed
argument MUST raise TypeError (CPython may accept or raise — mamba's to enforce)."""

from xml.dom.xmlbuilder import DOMBuilder
obj = object.__new__(DOMBuilder)
try:
    obj.canSetFeature(12345, None)  # name: str <- wrong-typed
    print("no_typeerror:")  # CPython accepted the wrong-typed arg; mamba must raise
except TypeError as e:
    print("typeerror:", type(e).__name__)
except Exception as e:
    print("setup_or_other:", type(e).__name__)
"###);
    assert!(out == "STRICT_TYPE_REJECTED" || out.starts_with("RUNTIME_REJECTED"),
        "type wall did not hold: {out}");
}

/// Ported from `tests/cpython/type/std-libs/xml_dom_xmlbuilder/DOMBuilder__getFeature__name_as_str_wrong.py`.
#[test]
fn test_gen_type_std_libs_xml_dom_xmlbuilder_DOMBuilder__getFeature__name_as_str_wrong() {
    let out = run_type_wall_fixture(r###"# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "xml_dom_xmlbuilder"
# dimension = "type"
# case = "DOMBuilder__getFeature__name_as_str_wrong"
# subject = "xml.dom.xmlbuilder.DOMBuilder.getFeature(name: str)"
# kind = "semantic"
# xfail = ""
# mem_carveout = ""
# source = "vendor/typeshed/stdlib/xml/dom/xmlbuilder.pyi"
# status = "filled"
# ///
# mamba-strict-type: TypeError
"""Type wall: xml.dom.xmlbuilder.DOMBuilder.getFeature(name: str); call it with the wrong type.

typeshed contract: name is str. mamba is force-typed, so a wrong-typed
argument MUST raise TypeError (CPython may accept or raise — mamba's to enforce)."""

from xml.dom.xmlbuilder import DOMBuilder
obj = object.__new__(DOMBuilder)
try:
    obj.getFeature(12345)  # name: str <- wrong-typed
    print("no_typeerror:")  # CPython accepted the wrong-typed arg; mamba must raise
except TypeError as e:
    print("typeerror:", type(e).__name__)
except Exception as e:
    print("setup_or_other:", type(e).__name__)
"###);
    assert!(out == "STRICT_TYPE_REJECTED" || out.starts_with("RUNTIME_REJECTED"),
        "type wall did not hold: {out}");
}

/// Ported from `tests/cpython/type/std-libs/xml_dom_xmlbuilder/DOMBuilder__parseURI__uri_as_str_wrong.py`.
#[test]
fn test_gen_type_std_libs_xml_dom_xmlbuilder_DOMBuilder__parseURI__uri_as_str_wrong() {
    let out = run_type_wall_fixture(r###"# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "xml_dom_xmlbuilder"
# dimension = "type"
# case = "DOMBuilder__parseURI__uri_as_str_wrong"
# subject = "xml.dom.xmlbuilder.DOMBuilder.parseURI(uri: str)"
# kind = "semantic"
# xfail = ""
# mem_carveout = ""
# source = "vendor/typeshed/stdlib/xml/dom/xmlbuilder.pyi"
# status = "filled"
# ///
# mamba-strict-type: TypeError
"""Type wall: xml.dom.xmlbuilder.DOMBuilder.parseURI(uri: str); call it with the wrong type.

typeshed contract: uri is str. mamba is force-typed, so a wrong-typed
argument MUST raise TypeError (CPython may accept or raise — mamba's to enforce)."""

from xml.dom.xmlbuilder import DOMBuilder
obj = object.__new__(DOMBuilder)
try:
    obj.parseURI(12345)  # uri: str <- wrong-typed
    print("no_typeerror:")  # CPython accepted the wrong-typed arg; mamba must raise
except TypeError as e:
    print("typeerror:", type(e).__name__)
except Exception as e:
    print("setup_or_other:", type(e).__name__)
"###);
    assert!(out == "STRICT_TYPE_REJECTED" || out.starts_with("RUNTIME_REJECTED"),
        "type wall did not hold: {out}");
}

/// Ported from `tests/cpython/type/std-libs/xml_dom_xmlbuilder/DOMBuilder__parseWithContext__input_as_DOMInputSource_wrong.py`.
#[test]
fn test_gen_type_std_libs_xml_dom_xmlbuilder_DOMBuilder__parseWithContext__input_as_DOMInputSource_wrong() {
    let out = run_type_wall_fixture(r###"# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "xml_dom_xmlbuilder"
# dimension = "type"
# case = "DOMBuilder__parseWithContext__input_as_DOMInputSource_wrong"
# subject = "xml.dom.xmlbuilder.DOMBuilder.parseWithContext(input: DOMInputSource)"
# kind = "semantic"
# xfail = ""
# mem_carveout = ""
# source = "vendor/typeshed/stdlib/xml/dom/xmlbuilder.pyi"
# status = "filled"
# ///
# mamba-strict-type: TypeError
"""Type wall: xml.dom.xmlbuilder.DOMBuilder.parseWithContext(input: DOMInputSource); call it with the wrong type.

typeshed contract: input is DOMInputSource. mamba is force-typed, so a wrong-typed
argument MUST raise TypeError (CPython may accept or raise — mamba's to enforce)."""

class _W:
    pass


from xml.dom.xmlbuilder import DOMBuilder
obj = object.__new__(DOMBuilder)
try:
    obj.parseWithContext(_W(), None, None)  # input: DOMInputSource <- wrong-typed
    print("no_typeerror:")  # CPython accepted the wrong-typed arg; mamba must raise
except TypeError as e:
    print("typeerror:", type(e).__name__)
except Exception as e:
    print("setup_or_other:", type(e).__name__)
"###);
    assert!(out == "STRICT_TYPE_REJECTED" || out.starts_with("RUNTIME_REJECTED"),
        "type wall did not hold: {out}");
}

/// Ported from `tests/cpython/type/std-libs/xml_dom_xmlbuilder/DOMBuilder__parse__input_as_DOMInputSource_wrong.py`.
#[test]
fn test_gen_type_std_libs_xml_dom_xmlbuilder_DOMBuilder__parse__input_as_DOMInputSource_wrong() {
    let out = run_type_wall_fixture(r###"# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "xml_dom_xmlbuilder"
# dimension = "type"
# case = "DOMBuilder__parse__input_as_DOMInputSource_wrong"
# subject = "xml.dom.xmlbuilder.DOMBuilder.parse(input: DOMInputSource)"
# kind = "semantic"
# xfail = ""
# mem_carveout = ""
# source = "vendor/typeshed/stdlib/xml/dom/xmlbuilder.pyi"
# status = "filled"
# ///
# mamba-strict-type: TypeError
"""Type wall: xml.dom.xmlbuilder.DOMBuilder.parse(input: DOMInputSource); call it with the wrong type.

typeshed contract: input is DOMInputSource. mamba is force-typed, so a wrong-typed
argument MUST raise TypeError (CPython may accept or raise — mamba's to enforce)."""

class _W:
    pass


from xml.dom.xmlbuilder import DOMBuilder
obj = object.__new__(DOMBuilder)
try:
    obj.parse(_W())  # input: DOMInputSource <- wrong-typed
    print("no_typeerror:")  # CPython accepted the wrong-typed arg; mamba must raise
except TypeError as e:
    print("typeerror:", type(e).__name__)
except Exception as e:
    print("setup_or_other:", type(e).__name__)
"###);
    assert!(out == "STRICT_TYPE_REJECTED" || out.starts_with("RUNTIME_REJECTED"),
        "type wall did not hold: {out}");
}

/// Ported from `tests/cpython/type/std-libs/xml_dom_xmlbuilder/DOMBuilder__setFeature__name_as_str_wrong.py`.
#[test]
fn test_gen_type_std_libs_xml_dom_xmlbuilder_DOMBuilder__setFeature__name_as_str_wrong() {
    let out = run_type_wall_fixture(r###"# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "xml_dom_xmlbuilder"
# dimension = "type"
# case = "DOMBuilder__setFeature__name_as_str_wrong"
# subject = "xml.dom.xmlbuilder.DOMBuilder.setFeature(name: str)"
# kind = "semantic"
# xfail = ""
# mem_carveout = ""
# source = "vendor/typeshed/stdlib/xml/dom/xmlbuilder.pyi"
# status = "filled"
# ///
# mamba-strict-type: TypeError
"""Type wall: xml.dom.xmlbuilder.DOMBuilder.setFeature(name: str); call it with the wrong type.

typeshed contract: name is str. mamba is force-typed, so a wrong-typed
argument MUST raise TypeError (CPython may accept or raise — mamba's to enforce)."""

from xml.dom.xmlbuilder import DOMBuilder
obj = object.__new__(DOMBuilder)
try:
    obj.setFeature(12345, 0)  # name: str <- wrong-typed
    print("no_typeerror:")  # CPython accepted the wrong-typed arg; mamba must raise
except TypeError as e:
    print("typeerror:", type(e).__name__)
except Exception as e:
    print("setup_or_other:", type(e).__name__)
"###);
    assert!(out == "STRICT_TYPE_REJECTED" || out.starts_with("RUNTIME_REJECTED"),
        "type wall did not hold: {out}");
}

/// Ported from `tests/cpython/type/std-libs/xml_dom_xmlbuilder/DOMBuilder__supportsFeature__name_as_str_wrong.py`.
#[test]
fn test_gen_type_std_libs_xml_dom_xmlbuilder_DOMBuilder__supportsFeature__name_as_str_wrong() {
    let out = run_type_wall_fixture(r###"# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "xml_dom_xmlbuilder"
# dimension = "type"
# case = "DOMBuilder__supportsFeature__name_as_str_wrong"
# subject = "xml.dom.xmlbuilder.DOMBuilder.supportsFeature(name: str)"
# kind = "semantic"
# xfail = ""
# mem_carveout = ""
# source = "vendor/typeshed/stdlib/xml/dom/xmlbuilder.pyi"
# status = "filled"
# ///
# mamba-strict-type: TypeError
"""Type wall: xml.dom.xmlbuilder.DOMBuilder.supportsFeature(name: str); call it with the wrong type.

typeshed contract: name is str. mamba is force-typed, so a wrong-typed
argument MUST raise TypeError (CPython may accept or raise — mamba's to enforce)."""

from xml.dom.xmlbuilder import DOMBuilder
obj = object.__new__(DOMBuilder)
try:
    obj.supportsFeature(12345)  # name: str <- wrong-typed
    print("no_typeerror:")  # CPython accepted the wrong-typed arg; mamba must raise
except TypeError as e:
    print("typeerror:", type(e).__name__)
except Exception as e:
    print("setup_or_other:", type(e).__name__)
"###);
    assert!(out == "STRICT_TYPE_REJECTED" || out.starts_with("RUNTIME_REJECTED"),
        "type wall did not hold: {out}");
}

/// Ported from `tests/cpython/type/std-libs/xml_dom_xmlbuilder/DOMEntityResolver__resolveEntity__publicId_as_typed_wrong.py`.
#[test]
fn test_gen_type_std_libs_xml_dom_xmlbuilder_DOMEntityResolver__resolveEntity__publicId_as_typed_wrong() {
    let out = run_type_wall_fixture(r###"# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "xml_dom_xmlbuilder"
# dimension = "type"
# case = "DOMEntityResolver__resolveEntity__publicId_as_typed_wrong"
# subject = "xml.dom.xmlbuilder.DOMEntityResolver.resolveEntity(publicId: typed)"
# kind = "semantic"
# xfail = ""
# mem_carveout = ""
# source = "vendor/typeshed/stdlib/xml/dom/xmlbuilder.pyi"
# status = "filled"
# ///
# mamba-strict-type: TypeError
"""Type wall: xml.dom.xmlbuilder.DOMEntityResolver.resolveEntity(publicId: typed); call it with the wrong type.

typeshed contract: publicId is typed. mamba is force-typed, so a wrong-typed
argument MUST raise TypeError (CPython may accept or raise — mamba's to enforce)."""

class _W:
    pass


from xml.dom.xmlbuilder import DOMEntityResolver
obj = object.__new__(DOMEntityResolver)
try:
    obj.resolveEntity(_W(), "")  # publicId: typed <- wrong-typed
    print("no_typeerror:")  # CPython accepted the wrong-typed arg; mamba must raise
except TypeError as e:
    print("typeerror:", type(e).__name__)
except Exception as e:
    print("setup_or_other:", type(e).__name__)
"###);
    assert!(out == "STRICT_TYPE_REJECTED" || out.starts_with("RUNTIME_REJECTED"),
        "type wall did not hold: {out}");
}

/// Ported from `tests/cpython/type/std-libs/xml_dom_xmlbuilder/DocumentLS__loadXML__source_as_str_wrong.py`.
#[test]
fn test_gen_type_std_libs_xml_dom_xmlbuilder_DocumentLS__loadXML__source_as_str_wrong() {
    let out = run_type_wall_fixture(r###"# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "xml_dom_xmlbuilder"
# dimension = "type"
# case = "DocumentLS__loadXML__source_as_str_wrong"
# subject = "xml.dom.xmlbuilder.DocumentLS.loadXML(source: str)"
# kind = "semantic"
# xfail = ""
# mem_carveout = ""
# source = "vendor/typeshed/stdlib/xml/dom/xmlbuilder.pyi"
# status = "filled"
# ///
# mamba-strict-type: TypeError
"""Type wall: xml.dom.xmlbuilder.DocumentLS.loadXML(source: str); call it with the wrong type.

typeshed contract: source is str. mamba is force-typed, so a wrong-typed
argument MUST raise TypeError (CPython may accept or raise — mamba's to enforce)."""

from xml.dom.xmlbuilder import DocumentLS
obj = object.__new__(DocumentLS)
try:
    obj.loadXML(12345)  # source: str <- wrong-typed
    print("no_typeerror:")  # CPython accepted the wrong-typed arg; mamba must raise
except TypeError as e:
    print("typeerror:", type(e).__name__)
except Exception as e:
    print("setup_or_other:", type(e).__name__)
"###);
    assert!(out == "STRICT_TYPE_REJECTED" || out.starts_with("RUNTIME_REJECTED"),
        "type wall did not hold: {out}");
}

/// Ported from `tests/cpython/type/std-libs/xml_dom_xmlbuilder/DocumentLS__load__uri_as_str_wrong.py`.
#[test]
fn test_gen_type_std_libs_xml_dom_xmlbuilder_DocumentLS__load__uri_as_str_wrong() {
    let out = run_type_wall_fixture(r###"# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "xml_dom_xmlbuilder"
# dimension = "type"
# case = "DocumentLS__load__uri_as_str_wrong"
# subject = "xml.dom.xmlbuilder.DocumentLS.load(uri: str)"
# kind = "semantic"
# xfail = ""
# mem_carveout = ""
# source = "vendor/typeshed/stdlib/xml/dom/xmlbuilder.pyi"
# status = "filled"
# ///
# mamba-strict-type: TypeError
"""Type wall: xml.dom.xmlbuilder.DocumentLS.load(uri: str); call it with the wrong type.

typeshed contract: uri is str. mamba is force-typed, so a wrong-typed
argument MUST raise TypeError (CPython may accept or raise — mamba's to enforce)."""

from xml.dom.xmlbuilder import DocumentLS
obj = object.__new__(DocumentLS)
try:
    obj.load(12345)  # uri: str <- wrong-typed
    print("no_typeerror:")  # CPython accepted the wrong-typed arg; mamba must raise
except TypeError as e:
    print("typeerror:", type(e).__name__)
except Exception as e:
    print("setup_or_other:", type(e).__name__)
"###);
    assert!(out == "STRICT_TYPE_REJECTED" || out.starts_with("RUNTIME_REJECTED"),
        "type wall did not hold: {out}");
}

/// Ported from `tests/cpython/type/std-libs/xml_dom_xmlbuilder/DocumentLS__saveXML__snode_as_typed_wrong.py`.
#[test]
fn test_gen_type_std_libs_xml_dom_xmlbuilder_DocumentLS__saveXML__snode_as_typed_wrong() {
    let out = run_type_wall_fixture(r###"# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "xml_dom_xmlbuilder"
# dimension = "type"
# case = "DocumentLS__saveXML__snode_as_typed_wrong"
# subject = "xml.dom.xmlbuilder.DocumentLS.saveXML(snode: typed)"
# kind = "semantic"
# xfail = ""
# mem_carveout = ""
# source = "vendor/typeshed/stdlib/xml/dom/xmlbuilder.pyi"
# status = "filled"
# ///
# mamba-strict-type: TypeError
"""Type wall: xml.dom.xmlbuilder.DocumentLS.saveXML(snode: typed); call it with the wrong type.

typeshed contract: snode is typed. mamba is force-typed, so a wrong-typed
argument MUST raise TypeError (CPython may accept or raise — mamba's to enforce)."""

class _W:
    pass


from xml.dom.xmlbuilder import DocumentLS
obj = object.__new__(DocumentLS)
try:
    obj.saveXML(_W())  # snode: typed <- wrong-typed
    print("no_typeerror:")  # CPython accepted the wrong-typed arg; mamba must raise
except TypeError as e:
    print("typeerror:", type(e).__name__)
except Exception as e:
    print("setup_or_other:", type(e).__name__)
"###);
    assert!(out == "STRICT_TYPE_REJECTED" || out.starts_with("RUNTIME_REJECTED"),
        "type wall did not hold: {out}");
}
