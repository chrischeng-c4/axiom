use super::super::super::super::harness::*;

/// Ported from `tests/cpython/type/std-libs/xml_dom_minidom/Attr__init__qName_as_str_wrong.py`.
#[test]
fn test_gen_type_std_libs_xml_dom_minidom_Attr__init__qName_as_str_wrong() {
    let out = run_type_wall_fixture(r###"# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "xml_dom_minidom"
# dimension = "type"
# case = "Attr__init__qName_as_str_wrong"
# subject = "xml.dom.minidom.Attr.__init__(qName: str)"
# kind = "semantic"
# xfail = ""
# mem_carveout = ""
# source = "vendor/typeshed/stdlib/xml/dom/minidom.pyi"
# status = "filled"
# ///
# mamba-strict-type: TypeError
"""Type wall: xml.dom.minidom.Attr.__init__(qName: str); call it with the wrong type.

typeshed contract: qName is str. mamba is force-typed, so a wrong-typed
argument MUST raise TypeError (CPython may accept or raise — mamba's to enforce)."""

from xml.dom.minidom import Attr
try:
    Attr(12345)  # qName: str <- wrong-typed
    print("no_typeerror:")  # CPython accepted the wrong-typed arg; mamba must raise
except TypeError as e:
    print("typeerror:", type(e).__name__)
except Exception as e:
    print("setup_or_other:", type(e).__name__)
"###);
    assert!(out == "STRICT_TYPE_REJECTED" || out.starts_with("RUNTIME_REJECTED"),
        "type wall did not hold: {out}");
}

/// Ported from `tests/cpython/type/std-libs/xml_dom_minidom/CDATASection__writexml__writer_as_SupportsWrite_wrong.py`.
#[test]
fn test_gen_type_std_libs_xml_dom_minidom_CDATASection__writexml__writer_as_SupportsWrite_wrong() {
    let out = run_type_wall_fixture(r###"# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "xml_dom_minidom"
# dimension = "type"
# case = "CDATASection__writexml__writer_as_SupportsWrite_wrong"
# subject = "xml.dom.minidom.CDATASection.writexml(writer: SupportsWrite)"
# kind = "semantic"
# xfail = ""
# mem_carveout = ""
# source = "vendor/typeshed/stdlib/xml/dom/minidom.pyi"
# status = "filled"
# ///
# mamba-strict-type: TypeError
"""Type wall: xml.dom.minidom.CDATASection.writexml(writer: SupportsWrite); call it with the wrong type.

typeshed contract: writer is SupportsWrite. mamba is force-typed, so a wrong-typed
argument MUST raise TypeError (CPython may accept or raise — mamba's to enforce)."""

class _W:
    pass


from xml.dom.minidom import CDATASection
obj = object.__new__(CDATASection)
try:
    obj.writexml(_W())  # writer: SupportsWrite <- wrong-typed
    print("no_typeerror:")  # CPython accepted the wrong-typed arg; mamba must raise
except TypeError as e:
    print("typeerror:", type(e).__name__)
except Exception as e:
    print("setup_or_other:", type(e).__name__)
"###);
    assert!(out == "STRICT_TYPE_REJECTED" || out.starts_with("RUNTIME_REJECTED"),
        "type wall did not hold: {out}");
}

/// Ported from `tests/cpython/type/std-libs/xml_dom_minidom/CharacterData__appendData__arg_as_str_wrong.py`.
#[test]
fn test_gen_type_std_libs_xml_dom_minidom_CharacterData__appendData__arg_as_str_wrong() {
    let out = run_type_wall_fixture(r###"# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "xml_dom_minidom"
# dimension = "type"
# case = "CharacterData__appendData__arg_as_str_wrong"
# subject = "xml.dom.minidom.CharacterData.appendData(arg: str)"
# kind = "semantic"
# xfail = ""
# mem_carveout = ""
# source = "vendor/typeshed/stdlib/xml/dom/minidom.pyi"
# status = "filled"
# ///
# mamba-strict-type: TypeError
"""Type wall: xml.dom.minidom.CharacterData.appendData(arg: str); call it with the wrong type.

typeshed contract: arg is str. mamba is force-typed, so a wrong-typed
argument MUST raise TypeError (CPython may accept or raise — mamba's to enforce)."""

from xml.dom.minidom import CharacterData
obj = object.__new__(CharacterData)
try:
    obj.appendData(12345)  # arg: str <- wrong-typed
    print("no_typeerror:")  # CPython accepted the wrong-typed arg; mamba must raise
except TypeError as e:
    print("typeerror:", type(e).__name__)
except Exception as e:
    print("setup_or_other:", type(e).__name__)
"###);
    assert!(out == "STRICT_TYPE_REJECTED" || out.starts_with("RUNTIME_REJECTED"),
        "type wall did not hold: {out}");
}

/// Ported from `tests/cpython/type/std-libs/xml_dom_minidom/CharacterData__deleteData__offset_as_int_wrong.py`.
#[test]
fn test_gen_type_std_libs_xml_dom_minidom_CharacterData__deleteData__offset_as_int_wrong() {
    let out = run_type_wall_fixture(r###"# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "xml_dom_minidom"
# dimension = "type"
# case = "CharacterData__deleteData__offset_as_int_wrong"
# subject = "xml.dom.minidom.CharacterData.deleteData(offset: int)"
# kind = "semantic"
# xfail = ""
# mem_carveout = ""
# source = "vendor/typeshed/stdlib/xml/dom/minidom.pyi"
# status = "filled"
# ///
# mamba-strict-type: TypeError
"""Type wall: xml.dom.minidom.CharacterData.deleteData(offset: int); call it with the wrong type.

typeshed contract: offset is int. mamba is force-typed, so a wrong-typed
argument MUST raise TypeError (CPython may accept or raise — mamba's to enforce)."""

from xml.dom.minidom import CharacterData
obj = object.__new__(CharacterData)
try:
    obj.deleteData("not_an_int", 0)  # offset: int <- wrong-typed
    print("no_typeerror:")  # CPython accepted the wrong-typed arg; mamba must raise
except TypeError as e:
    print("typeerror:", type(e).__name__)
except Exception as e:
    print("setup_or_other:", type(e).__name__)
"###);
    assert!(out == "STRICT_TYPE_REJECTED" || out.starts_with("RUNTIME_REJECTED"),
        "type wall did not hold: {out}");
}

/// Ported from `tests/cpython/type/std-libs/xml_dom_minidom/CharacterData__insertData__offset_as_int_wrong.py`.
#[test]
fn test_gen_type_std_libs_xml_dom_minidom_CharacterData__insertData__offset_as_int_wrong() {
    let out = run_type_wall_fixture(r###"# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "xml_dom_minidom"
# dimension = "type"
# case = "CharacterData__insertData__offset_as_int_wrong"
# subject = "xml.dom.minidom.CharacterData.insertData(offset: int)"
# kind = "semantic"
# xfail = ""
# mem_carveout = ""
# source = "vendor/typeshed/stdlib/xml/dom/minidom.pyi"
# status = "filled"
# ///
# mamba-strict-type: TypeError
"""Type wall: xml.dom.minidom.CharacterData.insertData(offset: int); call it with the wrong type.

typeshed contract: offset is int. mamba is force-typed, so a wrong-typed
argument MUST raise TypeError (CPython may accept or raise — mamba's to enforce)."""

from xml.dom.minidom import CharacterData
obj = object.__new__(CharacterData)
try:
    obj.insertData("not_an_int", "")  # offset: int <- wrong-typed
    print("no_typeerror:")  # CPython accepted the wrong-typed arg; mamba must raise
except TypeError as e:
    print("typeerror:", type(e).__name__)
except Exception as e:
    print("setup_or_other:", type(e).__name__)
"###);
    assert!(out == "STRICT_TYPE_REJECTED" || out.starts_with("RUNTIME_REJECTED"),
        "type wall did not hold: {out}");
}

/// Ported from `tests/cpython/type/std-libs/xml_dom_minidom/CharacterData__replaceData__offset_as_int_wrong.py`.
#[test]
fn test_gen_type_std_libs_xml_dom_minidom_CharacterData__replaceData__offset_as_int_wrong() {
    let out = run_type_wall_fixture(r###"# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "xml_dom_minidom"
# dimension = "type"
# case = "CharacterData__replaceData__offset_as_int_wrong"
# subject = "xml.dom.minidom.CharacterData.replaceData(offset: int)"
# kind = "semantic"
# xfail = ""
# mem_carveout = ""
# source = "vendor/typeshed/stdlib/xml/dom/minidom.pyi"
# status = "filled"
# ///
# mamba-strict-type: TypeError
"""Type wall: xml.dom.minidom.CharacterData.replaceData(offset: int); call it with the wrong type.

typeshed contract: offset is int. mamba is force-typed, so a wrong-typed
argument MUST raise TypeError (CPython may accept or raise — mamba's to enforce)."""

from xml.dom.minidom import CharacterData
obj = object.__new__(CharacterData)
try:
    obj.replaceData("not_an_int", 0, "")  # offset: int <- wrong-typed
    print("no_typeerror:")  # CPython accepted the wrong-typed arg; mamba must raise
except TypeError as e:
    print("typeerror:", type(e).__name__)
except Exception as e:
    print("setup_or_other:", type(e).__name__)
"###);
    assert!(out == "STRICT_TYPE_REJECTED" || out.starts_with("RUNTIME_REJECTED"),
        "type wall did not hold: {out}");
}

/// Ported from `tests/cpython/type/std-libs/xml_dom_minidom/CharacterData__substringData__offset_as_int_wrong.py`.
#[test]
fn test_gen_type_std_libs_xml_dom_minidom_CharacterData__substringData__offset_as_int_wrong() {
    let out = run_type_wall_fixture(r###"# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "xml_dom_minidom"
# dimension = "type"
# case = "CharacterData__substringData__offset_as_int_wrong"
# subject = "xml.dom.minidom.CharacterData.substringData(offset: int)"
# kind = "semantic"
# xfail = ""
# mem_carveout = ""
# source = "vendor/typeshed/stdlib/xml/dom/minidom.pyi"
# status = "filled"
# ///
# mamba-strict-type: TypeError
"""Type wall: xml.dom.minidom.CharacterData.substringData(offset: int); call it with the wrong type.

typeshed contract: offset is int. mamba is force-typed, so a wrong-typed
argument MUST raise TypeError (CPython may accept or raise — mamba's to enforce)."""

from xml.dom.minidom import CharacterData
obj = object.__new__(CharacterData)
try:
    obj.substringData("not_an_int", 0)  # offset: int <- wrong-typed
    print("no_typeerror:")  # CPython accepted the wrong-typed arg; mamba must raise
except TypeError as e:
    print("typeerror:", type(e).__name__)
except Exception as e:
    print("setup_or_other:", type(e).__name__)
"###);
    assert!(out == "STRICT_TYPE_REJECTED" || out.starts_with("RUNTIME_REJECTED"),
        "type wall did not hold: {out}");
}

/// Ported from `tests/cpython/type/std-libs/xml_dom_minidom/Childless__appendChild__node_as_typed_wrong.py`.
#[test]
fn test_gen_type_std_libs_xml_dom_minidom_Childless__appendChild__node_as_typed_wrong() {
    let out = run_type_wall_fixture(r###"# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "xml_dom_minidom"
# dimension = "type"
# case = "Childless__appendChild__node_as_typed_wrong"
# subject = "xml.dom.minidom.Childless.appendChild(node: typed)"
# kind = "semantic"
# xfail = ""
# mem_carveout = ""
# source = "vendor/typeshed/stdlib/xml/dom/minidom.pyi"
# status = "filled"
# ///
# mamba-strict-type: TypeError
"""Type wall: xml.dom.minidom.Childless.appendChild(node: typed); call it with the wrong type.

typeshed contract: node is typed. mamba is force-typed, so a wrong-typed
argument MUST raise TypeError (CPython may accept or raise — mamba's to enforce)."""

class _W:
    pass


from xml.dom.minidom import Childless
obj = object.__new__(Childless)
try:
    obj.appendChild(_W())  # node: typed <- wrong-typed
    print("no_typeerror:")  # CPython accepted the wrong-typed arg; mamba must raise
except TypeError as e:
    print("typeerror:", type(e).__name__)
except Exception as e:
    print("setup_or_other:", type(e).__name__)
"###);
    assert!(out == "STRICT_TYPE_REJECTED" || out.starts_with("RUNTIME_REJECTED"),
        "type wall did not hold: {out}");
}

/// Ported from `tests/cpython/type/std-libs/xml_dom_minidom/Childless__insertBefore__newChild_as_typed_wrong.py`.
#[test]
fn test_gen_type_std_libs_xml_dom_minidom_Childless__insertBefore__newChild_as_typed_wrong() {
    let out = run_type_wall_fixture(r###"# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "xml_dom_minidom"
# dimension = "type"
# case = "Childless__insertBefore__newChild_as_typed_wrong"
# subject = "xml.dom.minidom.Childless.insertBefore(newChild: typed)"
# kind = "semantic"
# xfail = ""
# mem_carveout = ""
# source = "vendor/typeshed/stdlib/xml/dom/minidom.pyi"
# status = "filled"
# ///
# mamba-strict-type: TypeError
"""Type wall: xml.dom.minidom.Childless.insertBefore(newChild: typed); call it with the wrong type.

typeshed contract: newChild is typed. mamba is force-typed, so a wrong-typed
argument MUST raise TypeError (CPython may accept or raise — mamba's to enforce)."""

class _W:
    pass


from xml.dom.minidom import Childless
obj = object.__new__(Childless)
try:
    obj.insertBefore(_W(), None)  # newChild: typed <- wrong-typed
    print("no_typeerror:")  # CPython accepted the wrong-typed arg; mamba must raise
except TypeError as e:
    print("typeerror:", type(e).__name__)
except Exception as e:
    print("setup_or_other:", type(e).__name__)
"###);
    assert!(out == "STRICT_TYPE_REJECTED" || out.starts_with("RUNTIME_REJECTED"),
        "type wall did not hold: {out}");
}

/// Ported from `tests/cpython/type/std-libs/xml_dom_minidom/Childless__removeChild__oldChild_as__NodesThatAreChildren_wrong.py`.
#[test]
fn test_gen_type_std_libs_xml_dom_minidom_Childless__removeChild__oldChild_as__NodesThatAreChildren_wrong() {
    let out = run_type_wall_fixture(r###"# /// script
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
"###);
    assert!(out == "STRICT_TYPE_REJECTED" || out.starts_with("RUNTIME_REJECTED"),
        "type wall did not hold: {out}");
}

/// Ported from `tests/cpython/type/std-libs/xml_dom_minidom/Childless__replaceChild__newChild_as_typed_wrong.py`.
#[test]
fn test_gen_type_std_libs_xml_dom_minidom_Childless__replaceChild__newChild_as_typed_wrong() {
    let out = run_type_wall_fixture(r###"# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "xml_dom_minidom"
# dimension = "type"
# case = "Childless__replaceChild__newChild_as_typed_wrong"
# subject = "xml.dom.minidom.Childless.replaceChild(newChild: typed)"
# kind = "semantic"
# xfail = ""
# mem_carveout = ""
# source = "vendor/typeshed/stdlib/xml/dom/minidom.pyi"
# status = "filled"
# ///
# mamba-strict-type: TypeError
"""Type wall: xml.dom.minidom.Childless.replaceChild(newChild: typed); call it with the wrong type.

typeshed contract: newChild is typed. mamba is force-typed, so a wrong-typed
argument MUST raise TypeError (CPython may accept or raise — mamba's to enforce)."""

class _W:
    pass


from xml.dom.minidom import Childless
obj = object.__new__(Childless)
try:
    obj.replaceChild(_W(), None)  # newChild: typed <- wrong-typed
    print("no_typeerror:")  # CPython accepted the wrong-typed arg; mamba must raise
except TypeError as e:
    print("typeerror:", type(e).__name__)
except Exception as e:
    print("setup_or_other:", type(e).__name__)
"###);
    assert!(out == "STRICT_TYPE_REJECTED" || out.starts_with("RUNTIME_REJECTED"),
        "type wall did not hold: {out}");
}

/// Ported from `tests/cpython/type/std-libs/xml_dom_minidom/Comment__init__data_as_str_wrong.py`.
#[test]
fn test_gen_type_std_libs_xml_dom_minidom_Comment__init__data_as_str_wrong() {
    let out = run_type_wall_fixture(r###"# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "xml_dom_minidom"
# dimension = "type"
# case = "Comment__init__data_as_str_wrong"
# subject = "xml.dom.minidom.Comment.__init__(data: str)"
# kind = "semantic"
# xfail = ""
# mem_carveout = ""
# source = "vendor/typeshed/stdlib/xml/dom/minidom.pyi"
# status = "filled"
# ///
# mamba-strict-type: TypeError
"""Type wall: xml.dom.minidom.Comment.__init__(data: str); call it with the wrong type.

typeshed contract: data is str. mamba is force-typed, so a wrong-typed
argument MUST raise TypeError (CPython may accept or raise — mamba's to enforce)."""

from xml.dom.minidom import Comment
try:
    Comment(12345)  # data: str <- wrong-typed
    print("no_typeerror:")  # CPython accepted the wrong-typed arg; mamba must raise
except TypeError as e:
    print("typeerror:", type(e).__name__)
except Exception as e:
    print("setup_or_other:", type(e).__name__)
"###);
    assert!(out == "STRICT_TYPE_REJECTED" || out.starts_with("RUNTIME_REJECTED"),
        "type wall did not hold: {out}");
}

/// Ported from `tests/cpython/type/std-libs/xml_dom_minidom/Comment__writexml__writer_as_SupportsWrite_wrong.py`.
#[test]
fn test_gen_type_std_libs_xml_dom_minidom_Comment__writexml__writer_as_SupportsWrite_wrong() {
    let out = run_type_wall_fixture(r###"# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "xml_dom_minidom"
# dimension = "type"
# case = "Comment__writexml__writer_as_SupportsWrite_wrong"
# subject = "xml.dom.minidom.Comment.writexml(writer: SupportsWrite)"
# kind = "semantic"
# xfail = ""
# mem_carveout = ""
# source = "vendor/typeshed/stdlib/xml/dom/minidom.pyi"
# status = "filled"
# ///
# mamba-strict-type: TypeError
"""Type wall: xml.dom.minidom.Comment.writexml(writer: SupportsWrite); call it with the wrong type.

typeshed contract: writer is SupportsWrite. mamba is force-typed, so a wrong-typed
argument MUST raise TypeError (CPython may accept or raise — mamba's to enforce)."""

class _W:
    pass


from xml.dom.minidom import Comment
obj = object.__new__(Comment)
try:
    obj.writexml(_W())  # writer: SupportsWrite <- wrong-typed
    print("no_typeerror:")  # CPython accepted the wrong-typed arg; mamba must raise
except TypeError as e:
    print("typeerror:", type(e).__name__)
except Exception as e:
    print("setup_or_other:", type(e).__name__)
"###);
    assert!(out == "STRICT_TYPE_REJECTED" || out.starts_with("RUNTIME_REJECTED"),
        "type wall did not hold: {out}");
}

/// Ported from `tests/cpython/type/std-libs/xml_dom_minidom/DOMImplementation__createDocumentType__qualifiedName_as_typed_wrong.py`.
#[test]
fn test_gen_type_std_libs_xml_dom_minidom_DOMImplementation__createDocumentType__qualifiedName_as_typed_wrong() {
    let out = run_type_wall_fixture(r###"# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "xml_dom_minidom"
# dimension = "type"
# case = "DOMImplementation__createDocumentType__qualifiedName_as_typed_wrong"
# subject = "xml.dom.minidom.DOMImplementation.createDocumentType(qualifiedName: typed)"
# kind = "semantic"
# xfail = ""
# mem_carveout = ""
# source = "vendor/typeshed/stdlib/xml/dom/minidom.pyi"
# status = "filled"
# ///
# mamba-strict-type: TypeError
"""Type wall: xml.dom.minidom.DOMImplementation.createDocumentType(qualifiedName: typed); call it with the wrong type.

typeshed contract: qualifiedName is typed. mamba is force-typed, so a wrong-typed
argument MUST raise TypeError (CPython may accept or raise — mamba's to enforce)."""

class _W:
    pass


from xml.dom.minidom import DOMImplementation
obj = object.__new__(DOMImplementation)
try:
    obj.createDocumentType(_W(), None, None)  # qualifiedName: typed <- wrong-typed
    print("no_typeerror:")  # CPython accepted the wrong-typed arg; mamba must raise
except TypeError as e:
    print("typeerror:", type(e).__name__)
except Exception as e:
    print("setup_or_other:", type(e).__name__)
"###);
    assert!(out == "STRICT_TYPE_REJECTED" || out.starts_with("RUNTIME_REJECTED"),
        "type wall did not hold: {out}");
}

/// Ported from `tests/cpython/type/std-libs/xml_dom_minidom/DOMImplementation__createDocument__namespaceURI_as_typed_wrong.py`.
#[test]
fn test_gen_type_std_libs_xml_dom_minidom_DOMImplementation__createDocument__namespaceURI_as_typed_wrong() {
    let out = run_type_wall_fixture(r###"# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "xml_dom_minidom"
# dimension = "type"
# case = "DOMImplementation__createDocument__namespaceURI_as_typed_wrong"
# subject = "xml.dom.minidom.DOMImplementation.createDocument(namespaceURI: typed)"
# kind = "semantic"
# xfail = ""
# mem_carveout = ""
# source = "vendor/typeshed/stdlib/xml/dom/minidom.pyi"
# status = "filled"
# ///
# mamba-strict-type: TypeError
"""Type wall: xml.dom.minidom.DOMImplementation.createDocument(namespaceURI: typed); call it with the wrong type.

typeshed contract: namespaceURI is typed. mamba is force-typed, so a wrong-typed
argument MUST raise TypeError (CPython may accept or raise — mamba's to enforce)."""

class _W:
    pass


from xml.dom.minidom import DOMImplementation
obj = object.__new__(DOMImplementation)
try:
    obj.createDocument(_W(), None, None)  # namespaceURI: typed <- wrong-typed
    print("no_typeerror:")  # CPython accepted the wrong-typed arg; mamba must raise
except TypeError as e:
    print("typeerror:", type(e).__name__)
except Exception as e:
    print("setup_or_other:", type(e).__name__)
"###);
    assert!(out == "STRICT_TYPE_REJECTED" || out.starts_with("RUNTIME_REJECTED"),
        "type wall did not hold: {out}");
}

/// Ported from `tests/cpython/type/std-libs/xml_dom_minidom/DOMImplementation__getInterface__feature_as_str_wrong.py`.
#[test]
fn test_gen_type_std_libs_xml_dom_minidom_DOMImplementation__getInterface__feature_as_str_wrong() {
    let out = run_type_wall_fixture(r###"# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "xml_dom_minidom"
# dimension = "type"
# case = "DOMImplementation__getInterface__feature_as_str_wrong"
# subject = "xml.dom.minidom.DOMImplementation.getInterface(feature: str)"
# kind = "semantic"
# xfail = ""
# mem_carveout = ""
# source = "vendor/typeshed/stdlib/xml/dom/minidom.pyi"
# status = "filled"
# ///
# mamba-strict-type: TypeError
"""Type wall: xml.dom.minidom.DOMImplementation.getInterface(feature: str); call it with the wrong type.

typeshed contract: feature is str. mamba is force-typed, so a wrong-typed
argument MUST raise TypeError (CPython may accept or raise — mamba's to enforce)."""

from xml.dom.minidom import DOMImplementation
obj = object.__new__(DOMImplementation)
try:
    obj.getInterface(12345)  # feature: str <- wrong-typed
    print("no_typeerror:")  # CPython accepted the wrong-typed arg; mamba must raise
except TypeError as e:
    print("typeerror:", type(e).__name__)
except Exception as e:
    print("setup_or_other:", type(e).__name__)
"###);
    assert!(out == "STRICT_TYPE_REJECTED" || out.starts_with("RUNTIME_REJECTED"),
        "type wall did not hold: {out}");
}

/// Ported from `tests/cpython/type/std-libs/xml_dom_minidom/DOMImplementation__hasFeature__feature_as_str_wrong.py`.
#[test]
fn test_gen_type_std_libs_xml_dom_minidom_DOMImplementation__hasFeature__feature_as_str_wrong() {
    let out = run_type_wall_fixture(r###"# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "xml_dom_minidom"
# dimension = "type"
# case = "DOMImplementation__hasFeature__feature_as_str_wrong"
# subject = "xml.dom.minidom.DOMImplementation.hasFeature(feature: str)"
# kind = "semantic"
# xfail = ""
# mem_carveout = ""
# source = "vendor/typeshed/stdlib/xml/dom/minidom.pyi"
# status = "filled"
# ///
# mamba-strict-type: TypeError
"""Type wall: xml.dom.minidom.DOMImplementation.hasFeature(feature: str); call it with the wrong type.

typeshed contract: feature is str. mamba is force-typed, so a wrong-typed
argument MUST raise TypeError (CPython may accept or raise — mamba's to enforce)."""

from xml.dom.minidom import DOMImplementation
obj = object.__new__(DOMImplementation)
try:
    obj.hasFeature(12345, None)  # feature: str <- wrong-typed
    print("no_typeerror:")  # CPython accepted the wrong-typed arg; mamba must raise
except TypeError as e:
    print("typeerror:", type(e).__name__)
except Exception as e:
    print("setup_or_other:", type(e).__name__)
"###);
    assert!(out == "STRICT_TYPE_REJECTED" || out.starts_with("RUNTIME_REJECTED"),
        "type wall did not hold: {out}");
}

/// Ported from `tests/cpython/type/std-libs/xml_dom_minidom/DocumentType__init__qualifiedName_as_typed_wrong.py`.
#[test]
fn test_gen_type_std_libs_xml_dom_minidom_DocumentType__init__qualifiedName_as_typed_wrong() {
    let out = run_type_wall_fixture(r###"# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "xml_dom_minidom"
# dimension = "type"
# case = "DocumentType__init__qualifiedName_as_typed_wrong"
# subject = "xml.dom.minidom.DocumentType.__init__(qualifiedName: typed)"
# kind = "semantic"
# xfail = ""
# mem_carveout = ""
# source = "vendor/typeshed/stdlib/xml/dom/minidom.pyi"
# status = "filled"
# ///
# mamba-strict-type: TypeError
"""Type wall: xml.dom.minidom.DocumentType.__init__(qualifiedName: typed); call it with the wrong type.

typeshed contract: qualifiedName is typed. mamba is force-typed, so a wrong-typed
argument MUST raise TypeError (CPython may accept or raise — mamba's to enforce)."""

class _W:
    pass


from xml.dom.minidom import DocumentType
try:
    DocumentType(_W())  # qualifiedName: typed <- wrong-typed
    print("no_typeerror:")  # CPython accepted the wrong-typed arg; mamba must raise
except TypeError as e:
    print("typeerror:", type(e).__name__)
except Exception as e:
    print("setup_or_other:", type(e).__name__)
"###);
    assert!(out == "STRICT_TYPE_REJECTED" || out.starts_with("RUNTIME_REJECTED"),
        "type wall did not hold: {out}");
}

/// Ported from `tests/cpython/type/std-libs/xml_dom_minidom/DocumentType__writexml__writer_as_SupportsWrite_wrong.py`.
#[test]
fn test_gen_type_std_libs_xml_dom_minidom_DocumentType__writexml__writer_as_SupportsWrite_wrong() {
    let out = run_type_wall_fixture(r###"# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "xml_dom_minidom"
# dimension = "type"
# case = "DocumentType__writexml__writer_as_SupportsWrite_wrong"
# subject = "xml.dom.minidom.DocumentType.writexml(writer: SupportsWrite)"
# kind = "semantic"
# xfail = ""
# mem_carveout = ""
# source = "vendor/typeshed/stdlib/xml/dom/minidom.pyi"
# status = "filled"
# ///
# mamba-strict-type: TypeError
"""Type wall: xml.dom.minidom.DocumentType.writexml(writer: SupportsWrite); call it with the wrong type.

typeshed contract: writer is SupportsWrite. mamba is force-typed, so a wrong-typed
argument MUST raise TypeError (CPython may accept or raise — mamba's to enforce)."""

class _W:
    pass


from xml.dom.minidom import DocumentType
obj = object.__new__(DocumentType)
try:
    obj.writexml(_W())  # writer: SupportsWrite <- wrong-typed
    print("no_typeerror:")  # CPython accepted the wrong-typed arg; mamba must raise
except TypeError as e:
    print("typeerror:", type(e).__name__)
except Exception as e:
    print("setup_or_other:", type(e).__name__)
"###);
    assert!(out == "STRICT_TYPE_REJECTED" || out.starts_with("RUNTIME_REJECTED"),
        "type wall did not hold: {out}");
}

/// Ported from `tests/cpython/type/std-libs/xml_dom_minidom/Document__createAttributeNS__namespaceURI_as_typed_wrong.py`.
#[test]
fn test_gen_type_std_libs_xml_dom_minidom_Document__createAttributeNS__namespaceURI_as_typed_wrong() {
    let out = run_type_wall_fixture(r###"# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "xml_dom_minidom"
# dimension = "type"
# case = "Document__createAttributeNS__namespaceURI_as_typed_wrong"
# subject = "xml.dom.minidom.Document.createAttributeNS(namespaceURI: typed)"
# kind = "semantic"
# xfail = ""
# mem_carveout = ""
# source = "vendor/typeshed/stdlib/xml/dom/minidom.pyi"
# status = "filled"
# ///
# mamba-strict-type: TypeError
"""Type wall: xml.dom.minidom.Document.createAttributeNS(namespaceURI: typed); call it with the wrong type.

typeshed contract: namespaceURI is typed. mamba is force-typed, so a wrong-typed
argument MUST raise TypeError (CPython may accept or raise — mamba's to enforce)."""

class _W:
    pass


from xml.dom.minidom import Document
obj = object.__new__(Document)
try:
    obj.createAttributeNS(_W(), "")  # namespaceURI: typed <- wrong-typed
    print("no_typeerror:")  # CPython accepted the wrong-typed arg; mamba must raise
except TypeError as e:
    print("typeerror:", type(e).__name__)
except Exception as e:
    print("setup_or_other:", type(e).__name__)
"###);
    assert!(out == "STRICT_TYPE_REJECTED" || out.starts_with("RUNTIME_REJECTED"),
        "type wall did not hold: {out}");
}

/// Ported from `tests/cpython/type/std-libs/xml_dom_minidom/Document__createAttribute__qName_as_str_wrong.py`.
#[test]
fn test_gen_type_std_libs_xml_dom_minidom_Document__createAttribute__qName_as_str_wrong() {
    let out = run_type_wall_fixture(r###"# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "xml_dom_minidom"
# dimension = "type"
# case = "Document__createAttribute__qName_as_str_wrong"
# subject = "xml.dom.minidom.Document.createAttribute(qName: str)"
# kind = "semantic"
# xfail = ""
# mem_carveout = ""
# source = "vendor/typeshed/stdlib/xml/dom/minidom.pyi"
# status = "filled"
# ///
# mamba-strict-type: TypeError
"""Type wall: xml.dom.minidom.Document.createAttribute(qName: str); call it with the wrong type.

typeshed contract: qName is str. mamba is force-typed, so a wrong-typed
argument MUST raise TypeError (CPython may accept or raise — mamba's to enforce)."""

from xml.dom.minidom import Document
obj = object.__new__(Document)
try:
    obj.createAttribute(12345)  # qName: str <- wrong-typed
    print("no_typeerror:")  # CPython accepted the wrong-typed arg; mamba must raise
except TypeError as e:
    print("typeerror:", type(e).__name__)
except Exception as e:
    print("setup_or_other:", type(e).__name__)
"###);
    assert!(out == "STRICT_TYPE_REJECTED" || out.starts_with("RUNTIME_REJECTED"),
        "type wall did not hold: {out}");
}

/// Ported from `tests/cpython/type/std-libs/xml_dom_minidom/Document__createCDATASection__data_as_str_wrong.py`.
#[test]
fn test_gen_type_std_libs_xml_dom_minidom_Document__createCDATASection__data_as_str_wrong() {
    let out = run_type_wall_fixture(r###"# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "xml_dom_minidom"
# dimension = "type"
# case = "Document__createCDATASection__data_as_str_wrong"
# subject = "xml.dom.minidom.Document.createCDATASection(data: str)"
# kind = "semantic"
# xfail = ""
# mem_carveout = ""
# source = "vendor/typeshed/stdlib/xml/dom/minidom.pyi"
# status = "filled"
# ///
# mamba-strict-type: TypeError
"""Type wall: xml.dom.minidom.Document.createCDATASection(data: str); call it with the wrong type.

typeshed contract: data is str. mamba is force-typed, so a wrong-typed
argument MUST raise TypeError (CPython may accept or raise — mamba's to enforce)."""

from xml.dom.minidom import Document
obj = object.__new__(Document)
try:
    obj.createCDATASection(12345)  # data: str <- wrong-typed
    print("no_typeerror:")  # CPython accepted the wrong-typed arg; mamba must raise
except TypeError as e:
    print("typeerror:", type(e).__name__)
except Exception as e:
    print("setup_or_other:", type(e).__name__)
"###);
    assert!(out == "STRICT_TYPE_REJECTED" || out.starts_with("RUNTIME_REJECTED"),
        "type wall did not hold: {out}");
}

/// Ported from `tests/cpython/type/std-libs/xml_dom_minidom/Document__createComment__data_as_str_wrong.py`.
#[test]
fn test_gen_type_std_libs_xml_dom_minidom_Document__createComment__data_as_str_wrong() {
    let out = run_type_wall_fixture(r###"# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "xml_dom_minidom"
# dimension = "type"
# case = "Document__createComment__data_as_str_wrong"
# subject = "xml.dom.minidom.Document.createComment(data: str)"
# kind = "semantic"
# xfail = ""
# mem_carveout = ""
# source = "vendor/typeshed/stdlib/xml/dom/minidom.pyi"
# status = "filled"
# ///
# mamba-strict-type: TypeError
"""Type wall: xml.dom.minidom.Document.createComment(data: str); call it with the wrong type.

typeshed contract: data is str. mamba is force-typed, so a wrong-typed
argument MUST raise TypeError (CPython may accept or raise — mamba's to enforce)."""

from xml.dom.minidom import Document
obj = object.__new__(Document)
try:
    obj.createComment(12345)  # data: str <- wrong-typed
    print("no_typeerror:")  # CPython accepted the wrong-typed arg; mamba must raise
except TypeError as e:
    print("typeerror:", type(e).__name__)
except Exception as e:
    print("setup_or_other:", type(e).__name__)
"###);
    assert!(out == "STRICT_TYPE_REJECTED" || out.starts_with("RUNTIME_REJECTED"),
        "type wall did not hold: {out}");
}

/// Ported from `tests/cpython/type/std-libs/xml_dom_minidom/Document__createElementNS__namespaceURI_as_typed_wrong.py`.
#[test]
fn test_gen_type_std_libs_xml_dom_minidom_Document__createElementNS__namespaceURI_as_typed_wrong() {
    let out = run_type_wall_fixture(r###"# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "xml_dom_minidom"
# dimension = "type"
# case = "Document__createElementNS__namespaceURI_as_typed_wrong"
# subject = "xml.dom.minidom.Document.createElementNS(namespaceURI: typed)"
# kind = "semantic"
# xfail = ""
# mem_carveout = ""
# source = "vendor/typeshed/stdlib/xml/dom/minidom.pyi"
# status = "filled"
# ///
# mamba-strict-type: TypeError
"""Type wall: xml.dom.minidom.Document.createElementNS(namespaceURI: typed); call it with the wrong type.

typeshed contract: namespaceURI is typed. mamba is force-typed, so a wrong-typed
argument MUST raise TypeError (CPython may accept or raise — mamba's to enforce)."""

class _W:
    pass


from xml.dom.minidom import Document
obj = object.__new__(Document)
try:
    obj.createElementNS(_W(), "")  # namespaceURI: typed <- wrong-typed
    print("no_typeerror:")  # CPython accepted the wrong-typed arg; mamba must raise
except TypeError as e:
    print("typeerror:", type(e).__name__)
except Exception as e:
    print("setup_or_other:", type(e).__name__)
"###);
    assert!(out == "STRICT_TYPE_REJECTED" || out.starts_with("RUNTIME_REJECTED"),
        "type wall did not hold: {out}");
}

/// Ported from `tests/cpython/type/std-libs/xml_dom_minidom/Document__createElement__tagName_as_str_wrong.py`.
#[test]
fn test_gen_type_std_libs_xml_dom_minidom_Document__createElement__tagName_as_str_wrong() {
    let out = run_type_wall_fixture(r###"# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "xml_dom_minidom"
# dimension = "type"
# case = "Document__createElement__tagName_as_str_wrong"
# subject = "xml.dom.minidom.Document.createElement(tagName: str)"
# kind = "semantic"
# xfail = ""
# mem_carveout = ""
# source = "vendor/typeshed/stdlib/xml/dom/minidom.pyi"
# status = "filled"
# ///
# mamba-strict-type: TypeError
"""Type wall: xml.dom.minidom.Document.createElement(tagName: str); call it with the wrong type.

typeshed contract: tagName is str. mamba is force-typed, so a wrong-typed
argument MUST raise TypeError (CPython may accept or raise — mamba's to enforce)."""

from xml.dom.minidom import Document
obj = object.__new__(Document)
try:
    obj.createElement(12345)  # tagName: str <- wrong-typed
    print("no_typeerror:")  # CPython accepted the wrong-typed arg; mamba must raise
except TypeError as e:
    print("typeerror:", type(e).__name__)
except Exception as e:
    print("setup_or_other:", type(e).__name__)
"###);
    assert!(out == "STRICT_TYPE_REJECTED" || out.starts_with("RUNTIME_REJECTED"),
        "type wall did not hold: {out}");
}

/// Ported from `tests/cpython/type/std-libs/xml_dom_minidom/Document__createProcessingInstruction__target_as_str_wrong.py`.
#[test]
fn test_gen_type_std_libs_xml_dom_minidom_Document__createProcessingInstruction__target_as_str_wrong() {
    let out = run_type_wall_fixture(r###"# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "xml_dom_minidom"
# dimension = "type"
# case = "Document__createProcessingInstruction__target_as_str_wrong"
# subject = "xml.dom.minidom.Document.createProcessingInstruction(target: str)"
# kind = "semantic"
# xfail = ""
# mem_carveout = ""
# source = "vendor/typeshed/stdlib/xml/dom/minidom.pyi"
# status = "filled"
# ///
# mamba-strict-type: TypeError
"""Type wall: xml.dom.minidom.Document.createProcessingInstruction(target: str); call it with the wrong type.

typeshed contract: target is str. mamba is force-typed, so a wrong-typed
argument MUST raise TypeError (CPython may accept or raise — mamba's to enforce)."""

from xml.dom.minidom import Document
obj = object.__new__(Document)
try:
    obj.createProcessingInstruction(12345, "")  # target: str <- wrong-typed
    print("no_typeerror:")  # CPython accepted the wrong-typed arg; mamba must raise
except TypeError as e:
    print("typeerror:", type(e).__name__)
except Exception as e:
    print("setup_or_other:", type(e).__name__)
"###);
    assert!(out == "STRICT_TYPE_REJECTED" || out.starts_with("RUNTIME_REJECTED"),
        "type wall did not hold: {out}");
}

/// Ported from `tests/cpython/type/std-libs/xml_dom_minidom/Document__createTextNode__data_as_str_wrong.py`.
#[test]
fn test_gen_type_std_libs_xml_dom_minidom_Document__createTextNode__data_as_str_wrong() {
    let out = run_type_wall_fixture(r###"# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "xml_dom_minidom"
# dimension = "type"
# case = "Document__createTextNode__data_as_str_wrong"
# subject = "xml.dom.minidom.Document.createTextNode(data: str)"
# kind = "semantic"
# xfail = ""
# mem_carveout = ""
# source = "vendor/typeshed/stdlib/xml/dom/minidom.pyi"
# status = "filled"
# ///
# mamba-strict-type: TypeError
"""Type wall: xml.dom.minidom.Document.createTextNode(data: str); call it with the wrong type.

typeshed contract: data is str. mamba is force-typed, so a wrong-typed
argument MUST raise TypeError (CPython may accept or raise — mamba's to enforce)."""

from xml.dom.minidom import Document
obj = object.__new__(Document)
try:
    obj.createTextNode(12345)  # data: str <- wrong-typed
    print("no_typeerror:")  # CPython accepted the wrong-typed arg; mamba must raise
except TypeError as e:
    print("typeerror:", type(e).__name__)
except Exception as e:
    print("setup_or_other:", type(e).__name__)
"###);
    assert!(out == "STRICT_TYPE_REJECTED" || out.starts_with("RUNTIME_REJECTED"),
        "type wall did not hold: {out}");
}

/// Ported from `tests/cpython/type/std-libs/xml_dom_minidom/Document__getElementById__id_as_str_wrong.py`.
#[test]
fn test_gen_type_std_libs_xml_dom_minidom_Document__getElementById__id_as_str_wrong() {
    let out = run_type_wall_fixture(r###"# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "xml_dom_minidom"
# dimension = "type"
# case = "Document__getElementById__id_as_str_wrong"
# subject = "xml.dom.minidom.Document.getElementById(id: str)"
# kind = "semantic"
# xfail = ""
# mem_carveout = ""
# source = "vendor/typeshed/stdlib/xml/dom/minidom.pyi"
# status = "filled"
# ///
# mamba-strict-type: TypeError
"""Type wall: xml.dom.minidom.Document.getElementById(id: str); call it with the wrong type.

typeshed contract: id is str. mamba is force-typed, so a wrong-typed
argument MUST raise TypeError (CPython may accept or raise — mamba's to enforce)."""

from xml.dom.minidom import Document
obj = object.__new__(Document)
try:
    obj.getElementById(12345)  # id: str <- wrong-typed
    print("no_typeerror:")  # CPython accepted the wrong-typed arg; mamba must raise
except TypeError as e:
    print("typeerror:", type(e).__name__)
except Exception as e:
    print("setup_or_other:", type(e).__name__)
"###);
    assert!(out == "STRICT_TYPE_REJECTED" || out.starts_with("RUNTIME_REJECTED"),
        "type wall did not hold: {out}");
}

/// Ported from `tests/cpython/type/std-libs/xml_dom_minidom/Document__getElementsByTagNameNS__namespaceURI_as_typed_wrong.py`.
#[test]
fn test_gen_type_std_libs_xml_dom_minidom_Document__getElementsByTagNameNS__namespaceURI_as_typed_wrong() {
    let out = run_type_wall_fixture(r###"# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "xml_dom_minidom"
# dimension = "type"
# case = "Document__getElementsByTagNameNS__namespaceURI_as_typed_wrong"
# subject = "xml.dom.minidom.Document.getElementsByTagNameNS(namespaceURI: typed)"
# kind = "semantic"
# xfail = ""
# mem_carveout = ""
# source = "vendor/typeshed/stdlib/xml/dom/minidom.pyi"
# status = "filled"
# ///
# mamba-strict-type: TypeError
"""Type wall: xml.dom.minidom.Document.getElementsByTagNameNS(namespaceURI: typed); call it with the wrong type.

typeshed contract: namespaceURI is typed. mamba is force-typed, so a wrong-typed
argument MUST raise TypeError (CPython may accept or raise — mamba's to enforce)."""

class _W:
    pass


from xml.dom.minidom import Document
obj = object.__new__(Document)
try:
    obj.getElementsByTagNameNS(_W(), "")  # namespaceURI: typed <- wrong-typed
    print("no_typeerror:")  # CPython accepted the wrong-typed arg; mamba must raise
except TypeError as e:
    print("typeerror:", type(e).__name__)
except Exception as e:
    print("setup_or_other:", type(e).__name__)
"###);
    assert!(out == "STRICT_TYPE_REJECTED" || out.starts_with("RUNTIME_REJECTED"),
        "type wall did not hold: {out}");
}

/// Ported from `tests/cpython/type/std-libs/xml_dom_minidom/Document__getElementsByTagName__name_as_str_wrong.py`.
#[test]
fn test_gen_type_std_libs_xml_dom_minidom_Document__getElementsByTagName__name_as_str_wrong() {
    let out = run_type_wall_fixture(r###"# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "xml_dom_minidom"
# dimension = "type"
# case = "Document__getElementsByTagName__name_as_str_wrong"
# subject = "xml.dom.minidom.Document.getElementsByTagName(name: str)"
# kind = "semantic"
# xfail = ""
# mem_carveout = ""
# source = "vendor/typeshed/stdlib/xml/dom/minidom.pyi"
# status = "filled"
# ///
# mamba-strict-type: TypeError
"""Type wall: xml.dom.minidom.Document.getElementsByTagName(name: str); call it with the wrong type.

typeshed contract: name is str. mamba is force-typed, so a wrong-typed
argument MUST raise TypeError (CPython may accept or raise — mamba's to enforce)."""

from xml.dom.minidom import Document
obj = object.__new__(Document)
try:
    obj.getElementsByTagName(12345)  # name: str <- wrong-typed
    print("no_typeerror:")  # CPython accepted the wrong-typed arg; mamba must raise
except TypeError as e:
    print("typeerror:", type(e).__name__)
except Exception as e:
    print("setup_or_other:", type(e).__name__)
"###);
    assert!(out == "STRICT_TYPE_REJECTED" || out.starts_with("RUNTIME_REJECTED"),
        "type wall did not hold: {out}");
}

/// Ported from `tests/cpython/type/std-libs/xml_dom_minidom/Document__isSupported__feature_as_str_wrong.py`.
#[test]
fn test_gen_type_std_libs_xml_dom_minidom_Document__isSupported__feature_as_str_wrong() {
    let out = run_type_wall_fixture(r###"# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "xml_dom_minidom"
# dimension = "type"
# case = "Document__isSupported__feature_as_str_wrong"
# subject = "xml.dom.minidom.Document.isSupported(feature: str)"
# kind = "semantic"
# xfail = ""
# mem_carveout = ""
# source = "vendor/typeshed/stdlib/xml/dom/minidom.pyi"
# status = "filled"
# ///
# mamba-strict-type: TypeError
"""Type wall: xml.dom.minidom.Document.isSupported(feature: str); call it with the wrong type.

typeshed contract: feature is str. mamba is force-typed, so a wrong-typed
argument MUST raise TypeError (CPython may accept or raise — mamba's to enforce)."""

from xml.dom.minidom import Document
obj = object.__new__(Document)
try:
    obj.isSupported(12345, None)  # feature: str <- wrong-typed
    print("no_typeerror:")  # CPython accepted the wrong-typed arg; mamba must raise
except TypeError as e:
    print("typeerror:", type(e).__name__)
except Exception as e:
    print("setup_or_other:", type(e).__name__)
"###);
    assert!(out == "STRICT_TYPE_REJECTED" || out.starts_with("RUNTIME_REJECTED"),
        "type wall did not hold: {out}");
}

/// Ported from `tests/cpython/type/std-libs/xml_dom_minidom/Document__writexml__writer_as_SupportsWrite_wrong.py`.
#[test]
fn test_gen_type_std_libs_xml_dom_minidom_Document__writexml__writer_as_SupportsWrite_wrong() {
    let out = run_type_wall_fixture(r###"# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "xml_dom_minidom"
# dimension = "type"
# case = "Document__writexml__writer_as_SupportsWrite_wrong"
# subject = "xml.dom.minidom.Document.writexml(writer: SupportsWrite)"
# kind = "semantic"
# xfail = ""
# mem_carveout = ""
# source = "vendor/typeshed/stdlib/xml/dom/minidom.pyi"
# status = "filled"
# ///
# mamba-strict-type: TypeError
"""Type wall: xml.dom.minidom.Document.writexml(writer: SupportsWrite); call it with the wrong type.

typeshed contract: writer is SupportsWrite. mamba is force-typed, so a wrong-typed
argument MUST raise TypeError (CPython may accept or raise — mamba's to enforce)."""

class _W:
    pass


from xml.dom.minidom import Document
obj = object.__new__(Document)
try:
    obj.writexml(_W())  # writer: SupportsWrite <- wrong-typed
    print("no_typeerror:")  # CPython accepted the wrong-typed arg; mamba must raise
except TypeError as e:
    print("typeerror:", type(e).__name__)
except Exception as e:
    print("setup_or_other:", type(e).__name__)
"###);
    assert!(out == "STRICT_TYPE_REJECTED" || out.starts_with("RUNTIME_REJECTED"),
        "type wall did not hold: {out}");
}

/// Ported from `tests/cpython/type/std-libs/xml_dom_minidom/ElementInfo__getAttributeTypeNS__namespaceURI_as_typed_wrong.py`.
#[test]
fn test_gen_type_std_libs_xml_dom_minidom_ElementInfo__getAttributeTypeNS__namespaceURI_as_typed_wrong() {
    let out = run_type_wall_fixture(r###"# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "xml_dom_minidom"
# dimension = "type"
# case = "ElementInfo__getAttributeTypeNS__namespaceURI_as_typed_wrong"
# subject = "xml.dom.minidom.ElementInfo.getAttributeTypeNS(namespaceURI: typed)"
# kind = "semantic"
# xfail = ""
# mem_carveout = ""
# source = "vendor/typeshed/stdlib/xml/dom/minidom.pyi"
# status = "filled"
# ///
# mamba-strict-type: TypeError
"""Type wall: xml.dom.minidom.ElementInfo.getAttributeTypeNS(namespaceURI: typed); call it with the wrong type.

typeshed contract: namespaceURI is typed. mamba is force-typed, so a wrong-typed
argument MUST raise TypeError (CPython may accept or raise — mamba's to enforce)."""

class _W:
    pass


from xml.dom.minidom import ElementInfo
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

/// Ported from `tests/cpython/type/std-libs/xml_dom_minidom/ElementInfo__getAttributeType__aname_as_str_wrong.py`.
#[test]
fn test_gen_type_std_libs_xml_dom_minidom_ElementInfo__getAttributeType__aname_as_str_wrong() {
    let out = run_type_wall_fixture(r###"# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "xml_dom_minidom"
# dimension = "type"
# case = "ElementInfo__getAttributeType__aname_as_str_wrong"
# subject = "xml.dom.minidom.ElementInfo.getAttributeType(aname: str)"
# kind = "semantic"
# xfail = ""
# mem_carveout = ""
# source = "vendor/typeshed/stdlib/xml/dom/minidom.pyi"
# status = "filled"
# ///
# mamba-strict-type: TypeError
"""Type wall: xml.dom.minidom.ElementInfo.getAttributeType(aname: str); call it with the wrong type.

typeshed contract: aname is str. mamba is force-typed, so a wrong-typed
argument MUST raise TypeError (CPython may accept or raise — mamba's to enforce)."""

from xml.dom.minidom import ElementInfo
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

/// Ported from `tests/cpython/type/std-libs/xml_dom_minidom/ElementInfo__init__name_as_str_wrong.py`.
#[test]
fn test_gen_type_std_libs_xml_dom_minidom_ElementInfo__init__name_as_str_wrong() {
    let out = run_type_wall_fixture(r###"# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "xml_dom_minidom"
# dimension = "type"
# case = "ElementInfo__init__name_as_str_wrong"
# subject = "xml.dom.minidom.ElementInfo.__init__(name: str)"
# kind = "semantic"
# xfail = ""
# mem_carveout = ""
# source = "vendor/typeshed/stdlib/xml/dom/minidom.pyi"
# status = "filled"
# ///
# mamba-strict-type: TypeError
"""Type wall: xml.dom.minidom.ElementInfo.__init__(name: str); call it with the wrong type.

typeshed contract: name is str. mamba is force-typed, so a wrong-typed
argument MUST raise TypeError (CPython may accept or raise — mamba's to enforce)."""

from xml.dom.minidom import ElementInfo
try:
    ElementInfo(12345)  # name: str <- wrong-typed
    print("no_typeerror:")  # CPython accepted the wrong-typed arg; mamba must raise
except TypeError as e:
    print("typeerror:", type(e).__name__)
except Exception as e:
    print("setup_or_other:", type(e).__name__)
"###);
    assert!(out == "STRICT_TYPE_REJECTED" || out.starts_with("RUNTIME_REJECTED"),
        "type wall did not hold: {out}");
}

/// Ported from `tests/cpython/type/std-libs/xml_dom_minidom/ElementInfo__isIdNS__namespaceURI_as_typed_wrong.py`.
#[test]
fn test_gen_type_std_libs_xml_dom_minidom_ElementInfo__isIdNS__namespaceURI_as_typed_wrong() {
    let out = run_type_wall_fixture(r###"# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "xml_dom_minidom"
# dimension = "type"
# case = "ElementInfo__isIdNS__namespaceURI_as_typed_wrong"
# subject = "xml.dom.minidom.ElementInfo.isIdNS(namespaceURI: typed)"
# kind = "semantic"
# xfail = ""
# mem_carveout = ""
# source = "vendor/typeshed/stdlib/xml/dom/minidom.pyi"
# status = "filled"
# ///
# mamba-strict-type: TypeError
"""Type wall: xml.dom.minidom.ElementInfo.isIdNS(namespaceURI: typed); call it with the wrong type.

typeshed contract: namespaceURI is typed. mamba is force-typed, so a wrong-typed
argument MUST raise TypeError (CPython may accept or raise — mamba's to enforce)."""

class _W:
    pass


from xml.dom.minidom import ElementInfo
obj = object.__new__(ElementInfo)
try:
    obj.isIdNS(_W(), "")  # namespaceURI: typed <- wrong-typed
    print("no_typeerror:")  # CPython accepted the wrong-typed arg; mamba must raise
except TypeError as e:
    print("typeerror:", type(e).__name__)
except Exception as e:
    print("setup_or_other:", type(e).__name__)
"###);
    assert!(out == "STRICT_TYPE_REJECTED" || out.starts_with("RUNTIME_REJECTED"),
        "type wall did not hold: {out}");
}

/// Ported from `tests/cpython/type/std-libs/xml_dom_minidom/ElementInfo__isId__aname_as_str_wrong.py`.
#[test]
fn test_gen_type_std_libs_xml_dom_minidom_ElementInfo__isId__aname_as_str_wrong() {
    let out = run_type_wall_fixture(r###"# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "xml_dom_minidom"
# dimension = "type"
# case = "ElementInfo__isId__aname_as_str_wrong"
# subject = "xml.dom.minidom.ElementInfo.isId(aname: str)"
# kind = "semantic"
# xfail = ""
# mem_carveout = ""
# source = "vendor/typeshed/stdlib/xml/dom/minidom.pyi"
# status = "filled"
# ///
# mamba-strict-type: TypeError
"""Type wall: xml.dom.minidom.ElementInfo.isId(aname: str); call it with the wrong type.

typeshed contract: aname is str. mamba is force-typed, so a wrong-typed
argument MUST raise TypeError (CPython may accept or raise — mamba's to enforce)."""

from xml.dom.minidom import ElementInfo
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

/// Ported from `tests/cpython/type/std-libs/xml_dom_minidom/Element__getAttributeNS__namespaceURI_as_typed_wrong.py`.
#[test]
fn test_gen_type_std_libs_xml_dom_minidom_Element__getAttributeNS__namespaceURI_as_typed_wrong() {
    let out = run_type_wall_fixture(r###"# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "xml_dom_minidom"
# dimension = "type"
# case = "Element__getAttributeNS__namespaceURI_as_typed_wrong"
# subject = "xml.dom.minidom.Element.getAttributeNS(namespaceURI: typed)"
# kind = "semantic"
# xfail = ""
# mem_carveout = ""
# source = "vendor/typeshed/stdlib/xml/dom/minidom.pyi"
# status = "filled"
# ///
# mamba-strict-type: TypeError
"""Type wall: xml.dom.minidom.Element.getAttributeNS(namespaceURI: typed); call it with the wrong type.

typeshed contract: namespaceURI is typed. mamba is force-typed, so a wrong-typed
argument MUST raise TypeError (CPython may accept or raise — mamba's to enforce)."""

class _W:
    pass


from xml.dom.minidom import Element
obj = object.__new__(Element)
try:
    obj.getAttributeNS(_W(), "")  # namespaceURI: typed <- wrong-typed
    print("no_typeerror:")  # CPython accepted the wrong-typed arg; mamba must raise
except TypeError as e:
    print("typeerror:", type(e).__name__)
except Exception as e:
    print("setup_or_other:", type(e).__name__)
"###);
    assert!(out == "STRICT_TYPE_REJECTED" || out.starts_with("RUNTIME_REJECTED"),
        "type wall did not hold: {out}");
}

/// Ported from `tests/cpython/type/std-libs/xml_dom_minidom/Element__getAttributeNodeNS__namespaceURI_as_typed_wrong.py`.
#[test]
fn test_gen_type_std_libs_xml_dom_minidom_Element__getAttributeNodeNS__namespaceURI_as_typed_wrong() {
    let out = run_type_wall_fixture(r###"# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "xml_dom_minidom"
# dimension = "type"
# case = "Element__getAttributeNodeNS__namespaceURI_as_typed_wrong"
# subject = "xml.dom.minidom.Element.getAttributeNodeNS(namespaceURI: typed)"
# kind = "semantic"
# xfail = ""
# mem_carveout = ""
# source = "vendor/typeshed/stdlib/xml/dom/minidom.pyi"
# status = "filled"
# ///
# mamba-strict-type: TypeError
"""Type wall: xml.dom.minidom.Element.getAttributeNodeNS(namespaceURI: typed); call it with the wrong type.

typeshed contract: namespaceURI is typed. mamba is force-typed, so a wrong-typed
argument MUST raise TypeError (CPython may accept or raise — mamba's to enforce)."""

class _W:
    pass


from xml.dom.minidom import Element
obj = object.__new__(Element)
try:
    obj.getAttributeNodeNS(_W(), "")  # namespaceURI: typed <- wrong-typed
    print("no_typeerror:")  # CPython accepted the wrong-typed arg; mamba must raise
except TypeError as e:
    print("typeerror:", type(e).__name__)
except Exception as e:
    print("setup_or_other:", type(e).__name__)
"###);
    assert!(out == "STRICT_TYPE_REJECTED" || out.starts_with("RUNTIME_REJECTED"),
        "type wall did not hold: {out}");
}

/// Ported from `tests/cpython/type/std-libs/xml_dom_minidom/Element__getAttributeNode__attrname_as_str_wrong.py`.
#[test]
fn test_gen_type_std_libs_xml_dom_minidom_Element__getAttributeNode__attrname_as_str_wrong() {
    let out = run_type_wall_fixture(r###"# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "xml_dom_minidom"
# dimension = "type"
# case = "Element__getAttributeNode__attrname_as_str_wrong"
# subject = "xml.dom.minidom.Element.getAttributeNode(attrname: str)"
# kind = "semantic"
# xfail = ""
# mem_carveout = ""
# source = "vendor/typeshed/stdlib/xml/dom/minidom.pyi"
# status = "filled"
# ///
# mamba-strict-type: TypeError
"""Type wall: xml.dom.minidom.Element.getAttributeNode(attrname: str); call it with the wrong type.

typeshed contract: attrname is str. mamba is force-typed, so a wrong-typed
argument MUST raise TypeError (CPython may accept or raise — mamba's to enforce)."""

from xml.dom.minidom import Element
obj = object.__new__(Element)
try:
    obj.getAttributeNode(12345)  # attrname: str <- wrong-typed
    print("no_typeerror:")  # CPython accepted the wrong-typed arg; mamba must raise
except TypeError as e:
    print("typeerror:", type(e).__name__)
except Exception as e:
    print("setup_or_other:", type(e).__name__)
"###);
    assert!(out == "STRICT_TYPE_REJECTED" || out.starts_with("RUNTIME_REJECTED"),
        "type wall did not hold: {out}");
}

/// Ported from `tests/cpython/type/std-libs/xml_dom_minidom/Element__getAttribute__attname_as_str_wrong.py`.
#[test]
fn test_gen_type_std_libs_xml_dom_minidom_Element__getAttribute__attname_as_str_wrong() {
    let out = run_type_wall_fixture(r###"# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "xml_dom_minidom"
# dimension = "type"
# case = "Element__getAttribute__attname_as_str_wrong"
# subject = "xml.dom.minidom.Element.getAttribute(attname: str)"
# kind = "semantic"
# xfail = ""
# mem_carveout = ""
# source = "vendor/typeshed/stdlib/xml/dom/minidom.pyi"
# status = "filled"
# ///
# mamba-strict-type: TypeError
"""Type wall: xml.dom.minidom.Element.getAttribute(attname: str); call it with the wrong type.

typeshed contract: attname is str. mamba is force-typed, so a wrong-typed
argument MUST raise TypeError (CPython may accept or raise — mamba's to enforce)."""

from xml.dom.minidom import Element
obj = object.__new__(Element)
try:
    obj.getAttribute(12345)  # attname: str <- wrong-typed
    print("no_typeerror:")  # CPython accepted the wrong-typed arg; mamba must raise
except TypeError as e:
    print("typeerror:", type(e).__name__)
except Exception as e:
    print("setup_or_other:", type(e).__name__)
"###);
    assert!(out == "STRICT_TYPE_REJECTED" || out.starts_with("RUNTIME_REJECTED"),
        "type wall did not hold: {out}");
}

/// Ported from `tests/cpython/type/std-libs/xml_dom_minidom/Element__getElementsByTagNameNS__namespaceURI_as_typed_wrong.py`.
#[test]
fn test_gen_type_std_libs_xml_dom_minidom_Element__getElementsByTagNameNS__namespaceURI_as_typed_wrong() {
    let out = run_type_wall_fixture(r###"# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "xml_dom_minidom"
# dimension = "type"
# case = "Element__getElementsByTagNameNS__namespaceURI_as_typed_wrong"
# subject = "xml.dom.minidom.Element.getElementsByTagNameNS(namespaceURI: typed)"
# kind = "semantic"
# xfail = ""
# mem_carveout = ""
# source = "vendor/typeshed/stdlib/xml/dom/minidom.pyi"
# status = "filled"
# ///
# mamba-strict-type: TypeError
"""Type wall: xml.dom.minidom.Element.getElementsByTagNameNS(namespaceURI: typed); call it with the wrong type.

typeshed contract: namespaceURI is typed. mamba is force-typed, so a wrong-typed
argument MUST raise TypeError (CPython may accept or raise — mamba's to enforce)."""

class _W:
    pass


from xml.dom.minidom import Element
obj = object.__new__(Element)
try:
    obj.getElementsByTagNameNS(_W(), "")  # namespaceURI: typed <- wrong-typed
    print("no_typeerror:")  # CPython accepted the wrong-typed arg; mamba must raise
except TypeError as e:
    print("typeerror:", type(e).__name__)
except Exception as e:
    print("setup_or_other:", type(e).__name__)
"###);
    assert!(out == "STRICT_TYPE_REJECTED" || out.starts_with("RUNTIME_REJECTED"),
        "type wall did not hold: {out}");
}

/// Ported from `tests/cpython/type/std-libs/xml_dom_minidom/Element__getElementsByTagName__name_as_str_wrong.py`.
#[test]
fn test_gen_type_std_libs_xml_dom_minidom_Element__getElementsByTagName__name_as_str_wrong() {
    let out = run_type_wall_fixture(r###"# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "xml_dom_minidom"
# dimension = "type"
# case = "Element__getElementsByTagName__name_as_str_wrong"
# subject = "xml.dom.minidom.Element.getElementsByTagName(name: str)"
# kind = "semantic"
# xfail = ""
# mem_carveout = ""
# source = "vendor/typeshed/stdlib/xml/dom/minidom.pyi"
# status = "filled"
# ///
# mamba-strict-type: TypeError
"""Type wall: xml.dom.minidom.Element.getElementsByTagName(name: str); call it with the wrong type.

typeshed contract: name is str. mamba is force-typed, so a wrong-typed
argument MUST raise TypeError (CPython may accept or raise — mamba's to enforce)."""

from xml.dom.minidom import Element
obj = object.__new__(Element)
try:
    obj.getElementsByTagName(12345)  # name: str <- wrong-typed
    print("no_typeerror:")  # CPython accepted the wrong-typed arg; mamba must raise
except TypeError as e:
    print("typeerror:", type(e).__name__)
except Exception as e:
    print("setup_or_other:", type(e).__name__)
"###);
    assert!(out == "STRICT_TYPE_REJECTED" || out.starts_with("RUNTIME_REJECTED"),
        "type wall did not hold: {out}");
}

/// Ported from `tests/cpython/type/std-libs/xml_dom_minidom/Element__hasAttributeNS__namespaceURI_as_typed_wrong.py`.
#[test]
fn test_gen_type_std_libs_xml_dom_minidom_Element__hasAttributeNS__namespaceURI_as_typed_wrong() {
    let out = run_type_wall_fixture(r###"# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "xml_dom_minidom"
# dimension = "type"
# case = "Element__hasAttributeNS__namespaceURI_as_typed_wrong"
# subject = "xml.dom.minidom.Element.hasAttributeNS(namespaceURI: typed)"
# kind = "semantic"
# xfail = ""
# mem_carveout = ""
# source = "vendor/typeshed/stdlib/xml/dom/minidom.pyi"
# status = "filled"
# ///
# mamba-strict-type: TypeError
"""Type wall: xml.dom.minidom.Element.hasAttributeNS(namespaceURI: typed); call it with the wrong type.

typeshed contract: namespaceURI is typed. mamba is force-typed, so a wrong-typed
argument MUST raise TypeError (CPython may accept or raise — mamba's to enforce)."""

class _W:
    pass


from xml.dom.minidom import Element
obj = object.__new__(Element)
try:
    obj.hasAttributeNS(_W(), "")  # namespaceURI: typed <- wrong-typed
    print("no_typeerror:")  # CPython accepted the wrong-typed arg; mamba must raise
except TypeError as e:
    print("typeerror:", type(e).__name__)
except Exception as e:
    print("setup_or_other:", type(e).__name__)
"###);
    assert!(out == "STRICT_TYPE_REJECTED" || out.starts_with("RUNTIME_REJECTED"),
        "type wall did not hold: {out}");
}

/// Ported from `tests/cpython/type/std-libs/xml_dom_minidom/Element__hasAttribute__name_as_str_wrong.py`.
#[test]
fn test_gen_type_std_libs_xml_dom_minidom_Element__hasAttribute__name_as_str_wrong() {
    let out = run_type_wall_fixture(r###"# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "xml_dom_minidom"
# dimension = "type"
# case = "Element__hasAttribute__name_as_str_wrong"
# subject = "xml.dom.minidom.Element.hasAttribute(name: str)"
# kind = "semantic"
# xfail = ""
# mem_carveout = ""
# source = "vendor/typeshed/stdlib/xml/dom/minidom.pyi"
# status = "filled"
# ///
# mamba-strict-type: TypeError
"""Type wall: xml.dom.minidom.Element.hasAttribute(name: str); call it with the wrong type.

typeshed contract: name is str. mamba is force-typed, so a wrong-typed
argument MUST raise TypeError (CPython may accept or raise — mamba's to enforce)."""

from xml.dom.minidom import Element
obj = object.__new__(Element)
try:
    obj.hasAttribute(12345)  # name: str <- wrong-typed
    print("no_typeerror:")  # CPython accepted the wrong-typed arg; mamba must raise
except TypeError as e:
    print("typeerror:", type(e).__name__)
except Exception as e:
    print("setup_or_other:", type(e).__name__)
"###);
    assert!(out == "STRICT_TYPE_REJECTED" || out.starts_with("RUNTIME_REJECTED"),
        "type wall did not hold: {out}");
}

/// Ported from `tests/cpython/type/std-libs/xml_dom_minidom/Element__init__tagName_as_str_wrong.py`.
#[test]
fn test_gen_type_std_libs_xml_dom_minidom_Element__init__tagName_as_str_wrong() {
    let out = run_type_wall_fixture(r###"# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "xml_dom_minidom"
# dimension = "type"
# case = "Element__init__tagName_as_str_wrong"
# subject = "xml.dom.minidom.Element.__init__(tagName: str)"
# kind = "semantic"
# xfail = ""
# mem_carveout = ""
# source = "vendor/typeshed/stdlib/xml/dom/minidom.pyi"
# status = "filled"
# ///
# mamba-strict-type: TypeError
"""Type wall: xml.dom.minidom.Element.__init__(tagName: str); call it with the wrong type.

typeshed contract: tagName is str. mamba is force-typed, so a wrong-typed
argument MUST raise TypeError (CPython may accept or raise — mamba's to enforce)."""

from xml.dom.minidom import Element
try:
    Element(12345)  # tagName: str <- wrong-typed
    print("no_typeerror:")  # CPython accepted the wrong-typed arg; mamba must raise
except TypeError as e:
    print("typeerror:", type(e).__name__)
except Exception as e:
    print("setup_or_other:", type(e).__name__)
"###);
    assert!(out == "STRICT_TYPE_REJECTED" || out.starts_with("RUNTIME_REJECTED"),
        "type wall did not hold: {out}");
}

/// Ported from `tests/cpython/type/std-libs/xml_dom_minidom/Element__removeAttributeNS__namespaceURI_as_typed_wrong.py`.
#[test]
fn test_gen_type_std_libs_xml_dom_minidom_Element__removeAttributeNS__namespaceURI_as_typed_wrong() {
    let out = run_type_wall_fixture(r###"# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "xml_dom_minidom"
# dimension = "type"
# case = "Element__removeAttributeNS__namespaceURI_as_typed_wrong"
# subject = "xml.dom.minidom.Element.removeAttributeNS(namespaceURI: typed)"
# kind = "semantic"
# xfail = ""
# mem_carveout = ""
# source = "vendor/typeshed/stdlib/xml/dom/minidom.pyi"
# status = "filled"
# ///
# mamba-strict-type: TypeError
"""Type wall: xml.dom.minidom.Element.removeAttributeNS(namespaceURI: typed); call it with the wrong type.

typeshed contract: namespaceURI is typed. mamba is force-typed, so a wrong-typed
argument MUST raise TypeError (CPython may accept or raise — mamba's to enforce)."""

class _W:
    pass


from xml.dom.minidom import Element
obj = object.__new__(Element)
try:
    obj.removeAttributeNS(_W(), "")  # namespaceURI: typed <- wrong-typed
    print("no_typeerror:")  # CPython accepted the wrong-typed arg; mamba must raise
except TypeError as e:
    print("typeerror:", type(e).__name__)
except Exception as e:
    print("setup_or_other:", type(e).__name__)
"###);
    assert!(out == "STRICT_TYPE_REJECTED" || out.starts_with("RUNTIME_REJECTED"),
        "type wall did not hold: {out}");
}

/// Ported from `tests/cpython/type/std-libs/xml_dom_minidom/Element__removeAttributeNode__node_as_Attr_wrong.py`.
#[test]
fn test_gen_type_std_libs_xml_dom_minidom_Element__removeAttributeNode__node_as_Attr_wrong() {
    let out = run_type_wall_fixture(r###"# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "xml_dom_minidom"
# dimension = "type"
# case = "Element__removeAttributeNode__node_as_Attr_wrong"
# subject = "xml.dom.minidom.Element.removeAttributeNode(node: Attr)"
# kind = "semantic"
# xfail = ""
# mem_carveout = ""
# source = "vendor/typeshed/stdlib/xml/dom/minidom.pyi"
# status = "filled"
# ///
# mamba-strict-type: TypeError
"""Type wall: xml.dom.minidom.Element.removeAttributeNode(node: Attr); call it with the wrong type.

typeshed contract: node is Attr. mamba is force-typed, so a wrong-typed
argument MUST raise TypeError (CPython may accept or raise — mamba's to enforce)."""

class _W:
    pass


from xml.dom.minidom import Element
obj = object.__new__(Element)
try:
    obj.removeAttributeNode(_W())  # node: Attr <- wrong-typed
    print("no_typeerror:")  # CPython accepted the wrong-typed arg; mamba must raise
except TypeError as e:
    print("typeerror:", type(e).__name__)
except Exception as e:
    print("setup_or_other:", type(e).__name__)
"###);
    assert!(out == "STRICT_TYPE_REJECTED" || out.starts_with("RUNTIME_REJECTED"),
        "type wall did not hold: {out}");
}

/// Ported from `tests/cpython/type/std-libs/xml_dom_minidom/Element__removeAttribute__name_as_str_wrong.py`.
#[test]
fn test_gen_type_std_libs_xml_dom_minidom_Element__removeAttribute__name_as_str_wrong() {
    let out = run_type_wall_fixture(r###"# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "xml_dom_minidom"
# dimension = "type"
# case = "Element__removeAttribute__name_as_str_wrong"
# subject = "xml.dom.minidom.Element.removeAttribute(name: str)"
# kind = "semantic"
# xfail = ""
# mem_carveout = ""
# source = "vendor/typeshed/stdlib/xml/dom/minidom.pyi"
# status = "filled"
# ///
# mamba-strict-type: TypeError
"""Type wall: xml.dom.minidom.Element.removeAttribute(name: str); call it with the wrong type.

typeshed contract: name is str. mamba is force-typed, so a wrong-typed
argument MUST raise TypeError (CPython may accept or raise — mamba's to enforce)."""

from xml.dom.minidom import Element
obj = object.__new__(Element)
try:
    obj.removeAttribute(12345)  # name: str <- wrong-typed
    print("no_typeerror:")  # CPython accepted the wrong-typed arg; mamba must raise
except TypeError as e:
    print("typeerror:", type(e).__name__)
except Exception as e:
    print("setup_or_other:", type(e).__name__)
"###);
    assert!(out == "STRICT_TYPE_REJECTED" || out.starts_with("RUNTIME_REJECTED"),
        "type wall did not hold: {out}");
}

/// Ported from `tests/cpython/type/std-libs/xml_dom_minidom/Element__setAttributeNS__namespaceURI_as_typed_wrong.py`.
#[test]
fn test_gen_type_std_libs_xml_dom_minidom_Element__setAttributeNS__namespaceURI_as_typed_wrong() {
    let out = run_type_wall_fixture(r###"# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "xml_dom_minidom"
# dimension = "type"
# case = "Element__setAttributeNS__namespaceURI_as_typed_wrong"
# subject = "xml.dom.minidom.Element.setAttributeNS(namespaceURI: typed)"
# kind = "semantic"
# xfail = ""
# mem_carveout = ""
# source = "vendor/typeshed/stdlib/xml/dom/minidom.pyi"
# status = "filled"
# ///
# mamba-strict-type: TypeError
"""Type wall: xml.dom.minidom.Element.setAttributeNS(namespaceURI: typed); call it with the wrong type.

typeshed contract: namespaceURI is typed. mamba is force-typed, so a wrong-typed
argument MUST raise TypeError (CPython may accept or raise — mamba's to enforce)."""

class _W:
    pass


from xml.dom.minidom import Element
obj = object.__new__(Element)
try:
    obj.setAttributeNS(_W(), "", "")  # namespaceURI: typed <- wrong-typed
    print("no_typeerror:")  # CPython accepted the wrong-typed arg; mamba must raise
except TypeError as e:
    print("typeerror:", type(e).__name__)
except Exception as e:
    print("setup_or_other:", type(e).__name__)
"###);
    assert!(out == "STRICT_TYPE_REJECTED" || out.starts_with("RUNTIME_REJECTED"),
        "type wall did not hold: {out}");
}

/// Ported from `tests/cpython/type/std-libs/xml_dom_minidom/Element__setAttributeNode__attr_as_Attr_wrong.py`.
#[test]
fn test_gen_type_std_libs_xml_dom_minidom_Element__setAttributeNode__attr_as_Attr_wrong() {
    let out = run_type_wall_fixture(r###"# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "xml_dom_minidom"
# dimension = "type"
# case = "Element__setAttributeNode__attr_as_Attr_wrong"
# subject = "xml.dom.minidom.Element.setAttributeNode(attr: Attr)"
# kind = "semantic"
# xfail = ""
# mem_carveout = ""
# source = "vendor/typeshed/stdlib/xml/dom/minidom.pyi"
# status = "filled"
# ///
# mamba-strict-type: TypeError
"""Type wall: xml.dom.minidom.Element.setAttributeNode(attr: Attr); call it with the wrong type.

typeshed contract: attr is Attr. mamba is force-typed, so a wrong-typed
argument MUST raise TypeError (CPython may accept or raise — mamba's to enforce)."""

class _W:
    pass


from xml.dom.minidom import Element
obj = object.__new__(Element)
try:
    obj.setAttributeNode(_W())  # attr: Attr <- wrong-typed
    print("no_typeerror:")  # CPython accepted the wrong-typed arg; mamba must raise
except TypeError as e:
    print("typeerror:", type(e).__name__)
except Exception as e:
    print("setup_or_other:", type(e).__name__)
"###);
    assert!(out == "STRICT_TYPE_REJECTED" || out.starts_with("RUNTIME_REJECTED"),
        "type wall did not hold: {out}");
}

/// Ported from `tests/cpython/type/std-libs/xml_dom_minidom/Element__setAttribute__attname_as_str_wrong.py`.
#[test]
fn test_gen_type_std_libs_xml_dom_minidom_Element__setAttribute__attname_as_str_wrong() {
    let out = run_type_wall_fixture(r###"# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "xml_dom_minidom"
# dimension = "type"
# case = "Element__setAttribute__attname_as_str_wrong"
# subject = "xml.dom.minidom.Element.setAttribute(attname: str)"
# kind = "semantic"
# xfail = ""
# mem_carveout = ""
# source = "vendor/typeshed/stdlib/xml/dom/minidom.pyi"
# status = "filled"
# ///
# mamba-strict-type: TypeError
"""Type wall: xml.dom.minidom.Element.setAttribute(attname: str); call it with the wrong type.

typeshed contract: attname is str. mamba is force-typed, so a wrong-typed
argument MUST raise TypeError (CPython may accept or raise — mamba's to enforce)."""

from xml.dom.minidom import Element
obj = object.__new__(Element)
try:
    obj.setAttribute(12345, "")  # attname: str <- wrong-typed
    print("no_typeerror:")  # CPython accepted the wrong-typed arg; mamba must raise
except TypeError as e:
    print("typeerror:", type(e).__name__)
except Exception as e:
    print("setup_or_other:", type(e).__name__)
"###);
    assert!(out == "STRICT_TYPE_REJECTED" || out.starts_with("RUNTIME_REJECTED"),
        "type wall did not hold: {out}");
}

/// Ported from `tests/cpython/type/std-libs/xml_dom_minidom/Element__setIdAttributeNS__namespaceURI_as_typed_wrong.py`.
#[test]
fn test_gen_type_std_libs_xml_dom_minidom_Element__setIdAttributeNS__namespaceURI_as_typed_wrong() {
    let out = run_type_wall_fixture(r###"# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "xml_dom_minidom"
# dimension = "type"
# case = "Element__setIdAttributeNS__namespaceURI_as_typed_wrong"
# subject = "xml.dom.minidom.Element.setIdAttributeNS(namespaceURI: typed)"
# kind = "semantic"
# xfail = ""
# mem_carveout = ""
# source = "vendor/typeshed/stdlib/xml/dom/minidom.pyi"
# status = "filled"
# ///
# mamba-strict-type: TypeError
"""Type wall: xml.dom.minidom.Element.setIdAttributeNS(namespaceURI: typed); call it with the wrong type.

typeshed contract: namespaceURI is typed. mamba is force-typed, so a wrong-typed
argument MUST raise TypeError (CPython may accept or raise — mamba's to enforce)."""

class _W:
    pass


from xml.dom.minidom import Element
obj = object.__new__(Element)
try:
    obj.setIdAttributeNS(_W(), "")  # namespaceURI: typed <- wrong-typed
    print("no_typeerror:")  # CPython accepted the wrong-typed arg; mamba must raise
except TypeError as e:
    print("typeerror:", type(e).__name__)
except Exception as e:
    print("setup_or_other:", type(e).__name__)
"###);
    assert!(out == "STRICT_TYPE_REJECTED" || out.starts_with("RUNTIME_REJECTED"),
        "type wall did not hold: {out}");
}

/// Ported from `tests/cpython/type/std-libs/xml_dom_minidom/Element__setIdAttributeNode__idAttr_as_Attr_wrong.py`.
#[test]
fn test_gen_type_std_libs_xml_dom_minidom_Element__setIdAttributeNode__idAttr_as_Attr_wrong() {
    let out = run_type_wall_fixture(r###"# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "xml_dom_minidom"
# dimension = "type"
# case = "Element__setIdAttributeNode__idAttr_as_Attr_wrong"
# subject = "xml.dom.minidom.Element.setIdAttributeNode(idAttr: Attr)"
# kind = "semantic"
# xfail = ""
# mem_carveout = ""
# source = "vendor/typeshed/stdlib/xml/dom/minidom.pyi"
# status = "filled"
# ///
# mamba-strict-type: TypeError
"""Type wall: xml.dom.minidom.Element.setIdAttributeNode(idAttr: Attr); call it with the wrong type.

typeshed contract: idAttr is Attr. mamba is force-typed, so a wrong-typed
argument MUST raise TypeError (CPython may accept or raise — mamba's to enforce)."""

class _W:
    pass


from xml.dom.minidom import Element
obj = object.__new__(Element)
try:
    obj.setIdAttributeNode(_W())  # idAttr: Attr <- wrong-typed
    print("no_typeerror:")  # CPython accepted the wrong-typed arg; mamba must raise
except TypeError as e:
    print("typeerror:", type(e).__name__)
except Exception as e:
    print("setup_or_other:", type(e).__name__)
"###);
    assert!(out == "STRICT_TYPE_REJECTED" || out.starts_with("RUNTIME_REJECTED"),
        "type wall did not hold: {out}");
}

/// Ported from `tests/cpython/type/std-libs/xml_dom_minidom/Element__setIdAttribute__name_as_str_wrong.py`.
#[test]
fn test_gen_type_std_libs_xml_dom_minidom_Element__setIdAttribute__name_as_str_wrong() {
    let out = run_type_wall_fixture(r###"# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "xml_dom_minidom"
# dimension = "type"
# case = "Element__setIdAttribute__name_as_str_wrong"
# subject = "xml.dom.minidom.Element.setIdAttribute(name: str)"
# kind = "semantic"
# xfail = ""
# mem_carveout = ""
# source = "vendor/typeshed/stdlib/xml/dom/minidom.pyi"
# status = "filled"
# ///
# mamba-strict-type: TypeError
"""Type wall: xml.dom.minidom.Element.setIdAttribute(name: str); call it with the wrong type.

typeshed contract: name is str. mamba is force-typed, so a wrong-typed
argument MUST raise TypeError (CPython may accept or raise — mamba's to enforce)."""

from xml.dom.minidom import Element
obj = object.__new__(Element)
try:
    obj.setIdAttribute(12345)  # name: str <- wrong-typed
    print("no_typeerror:")  # CPython accepted the wrong-typed arg; mamba must raise
except TypeError as e:
    print("typeerror:", type(e).__name__)
except Exception as e:
    print("setup_or_other:", type(e).__name__)
"###);
    assert!(out == "STRICT_TYPE_REJECTED" || out.starts_with("RUNTIME_REJECTED"),
        "type wall did not hold: {out}");
}

/// Ported from `tests/cpython/type/std-libs/xml_dom_minidom/Element__writexml__writer_as_SupportsWrite_wrong.py`.
#[test]
fn test_gen_type_std_libs_xml_dom_minidom_Element__writexml__writer_as_SupportsWrite_wrong() {
    let out = run_type_wall_fixture(r###"# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "xml_dom_minidom"
# dimension = "type"
# case = "Element__writexml__writer_as_SupportsWrite_wrong"
# subject = "xml.dom.minidom.Element.writexml(writer: SupportsWrite)"
# kind = "semantic"
# xfail = ""
# mem_carveout = ""
# source = "vendor/typeshed/stdlib/xml/dom/minidom.pyi"
# status = "filled"
# ///
# mamba-strict-type: TypeError
"""Type wall: xml.dom.minidom.Element.writexml(writer: SupportsWrite); call it with the wrong type.

typeshed contract: writer is SupportsWrite. mamba is force-typed, so a wrong-typed
argument MUST raise TypeError (CPython may accept or raise — mamba's to enforce)."""

class _W:
    pass


from xml.dom.minidom import Element
obj = object.__new__(Element)
try:
    obj.writexml(_W())  # writer: SupportsWrite <- wrong-typed
    print("no_typeerror:")  # CPython accepted the wrong-typed arg; mamba must raise
except TypeError as e:
    print("typeerror:", type(e).__name__)
except Exception as e:
    print("setup_or_other:", type(e).__name__)
"###);
    assert!(out == "STRICT_TYPE_REJECTED" || out.starts_with("RUNTIME_REJECTED"),
        "type wall did not hold: {out}");
}

/// Ported from `tests/cpython/type/std-libs/xml_dom_minidom/Entity__appendChild__newChild_as__EntityChildren_wrong.py`.
#[test]
fn test_gen_type_std_libs_xml_dom_minidom_Entity__appendChild__newChild_as__EntityChildren_wrong() {
    let out = run_type_wall_fixture(r###"# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "xml_dom_minidom"
# dimension = "type"
# case = "Entity__appendChild__newChild_as__EntityChildren_wrong"
# subject = "xml.dom.minidom.Entity.appendChild(newChild: _EntityChildren)"
# kind = "semantic"
# xfail = ""
# mem_carveout = ""
# source = "vendor/typeshed/stdlib/xml/dom/minidom.pyi"
# status = "filled"
# ///
# mamba-strict-type: TypeError
"""Type wall: xml.dom.minidom.Entity.appendChild(newChild: _EntityChildren); call it with the wrong type.

typeshed contract: newChild is _EntityChildren. mamba is force-typed, so a wrong-typed
argument MUST raise TypeError (CPython may accept or raise — mamba's to enforce)."""

class _W:
    pass


from xml.dom.minidom import Entity
obj = object.__new__(Entity)
try:
    obj.appendChild(_W())  # newChild: _EntityChildren <- wrong-typed
    print("no_typeerror:")  # CPython accepted the wrong-typed arg; mamba must raise
except TypeError as e:
    print("typeerror:", type(e).__name__)
except Exception as e:
    print("setup_or_other:", type(e).__name__)
"###);
    assert!(out == "STRICT_TYPE_REJECTED" || out.starts_with("RUNTIME_REJECTED"),
        "type wall did not hold: {out}");
}

/// Ported from `tests/cpython/type/std-libs/xml_dom_minidom/Entity__init__name_as_str_wrong.py`.
#[test]
fn test_gen_type_std_libs_xml_dom_minidom_Entity__init__name_as_str_wrong() {
    let out = run_type_wall_fixture(r###"# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "xml_dom_minidom"
# dimension = "type"
# case = "Entity__init__name_as_str_wrong"
# subject = "xml.dom.minidom.Entity.__init__(name: str)"
# kind = "semantic"
# xfail = ""
# mem_carveout = ""
# source = "vendor/typeshed/stdlib/xml/dom/minidom.pyi"
# status = "filled"
# ///
# mamba-strict-type: TypeError
"""Type wall: xml.dom.minidom.Entity.__init__(name: str); call it with the wrong type.

typeshed contract: name is str. mamba is force-typed, so a wrong-typed
argument MUST raise TypeError (CPython may accept or raise — mamba's to enforce)."""

from xml.dom.minidom import Entity
try:
    Entity(12345, None, None, None)  # name: str <- wrong-typed
    print("no_typeerror:")  # CPython accepted the wrong-typed arg; mamba must raise
except TypeError as e:
    print("typeerror:", type(e).__name__)
except Exception as e:
    print("setup_or_other:", type(e).__name__)
"###);
    assert!(out == "STRICT_TYPE_REJECTED" || out.starts_with("RUNTIME_REJECTED"),
        "type wall did not hold: {out}");
}

/// Ported from `tests/cpython/type/std-libs/xml_dom_minidom/Entity__insertBefore__newChild_as__EntityChildren_wrong.py`.
#[test]
fn test_gen_type_std_libs_xml_dom_minidom_Entity__insertBefore__newChild_as__EntityChildren_wrong() {
    let out = run_type_wall_fixture(r###"# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "xml_dom_minidom"
# dimension = "type"
# case = "Entity__insertBefore__newChild_as__EntityChildren_wrong"
# subject = "xml.dom.minidom.Entity.insertBefore(newChild: _EntityChildren)"
# kind = "semantic"
# xfail = ""
# mem_carveout = ""
# source = "vendor/typeshed/stdlib/xml/dom/minidom.pyi"
# status = "filled"
# ///
# mamba-strict-type: TypeError
"""Type wall: xml.dom.minidom.Entity.insertBefore(newChild: _EntityChildren); call it with the wrong type.

typeshed contract: newChild is _EntityChildren. mamba is force-typed, so a wrong-typed
argument MUST raise TypeError (CPython may accept or raise — mamba's to enforce)."""

class _W:
    pass


from xml.dom.minidom import Entity
obj = object.__new__(Entity)
try:
    obj.insertBefore(_W(), None)  # newChild: _EntityChildren <- wrong-typed
    print("no_typeerror:")  # CPython accepted the wrong-typed arg; mamba must raise
except TypeError as e:
    print("typeerror:", type(e).__name__)
except Exception as e:
    print("setup_or_other:", type(e).__name__)
"###);
    assert!(out == "STRICT_TYPE_REJECTED" || out.starts_with("RUNTIME_REJECTED"),
        "type wall did not hold: {out}");
}

/// Ported from `tests/cpython/type/std-libs/xml_dom_minidom/Entity__removeChild__oldChild_as__EntityChildren_wrong.py`.
#[test]
fn test_gen_type_std_libs_xml_dom_minidom_Entity__removeChild__oldChild_as__EntityChildren_wrong() {
    let out = run_type_wall_fixture(r###"# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "xml_dom_minidom"
# dimension = "type"
# case = "Entity__removeChild__oldChild_as__EntityChildren_wrong"
# subject = "xml.dom.minidom.Entity.removeChild(oldChild: _EntityChildren)"
# kind = "semantic"
# xfail = ""
# mem_carveout = ""
# source = "vendor/typeshed/stdlib/xml/dom/minidom.pyi"
# status = "filled"
# ///
# mamba-strict-type: TypeError
"""Type wall: xml.dom.minidom.Entity.removeChild(oldChild: _EntityChildren); call it with the wrong type.

typeshed contract: oldChild is _EntityChildren. mamba is force-typed, so a wrong-typed
argument MUST raise TypeError (CPython may accept or raise — mamba's to enforce)."""

class _W:
    pass


from xml.dom.minidom import Entity
obj = object.__new__(Entity)
try:
    obj.removeChild(_W())  # oldChild: _EntityChildren <- wrong-typed
    print("no_typeerror:")  # CPython accepted the wrong-typed arg; mamba must raise
except TypeError as e:
    print("typeerror:", type(e).__name__)
except Exception as e:
    print("setup_or_other:", type(e).__name__)
"###);
    assert!(out == "STRICT_TYPE_REJECTED" || out.starts_with("RUNTIME_REJECTED"),
        "type wall did not hold: {out}");
}

/// Ported from `tests/cpython/type/std-libs/xml_dom_minidom/Entity__replaceChild__newChild_as__EntityChildren_wrong.py`.
#[test]
fn test_gen_type_std_libs_xml_dom_minidom_Entity__replaceChild__newChild_as__EntityChildren_wrong() {
    let out = run_type_wall_fixture(r###"# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "xml_dom_minidom"
# dimension = "type"
# case = "Entity__replaceChild__newChild_as__EntityChildren_wrong"
# subject = "xml.dom.minidom.Entity.replaceChild(newChild: _EntityChildren)"
# kind = "semantic"
# xfail = ""
# mem_carveout = ""
# source = "vendor/typeshed/stdlib/xml/dom/minidom.pyi"
# status = "filled"
# ///
# mamba-strict-type: TypeError
"""Type wall: xml.dom.minidom.Entity.replaceChild(newChild: _EntityChildren); call it with the wrong type.

typeshed contract: newChild is _EntityChildren. mamba is force-typed, so a wrong-typed
argument MUST raise TypeError (CPython may accept or raise — mamba's to enforce)."""

class _W:
    pass


from xml.dom.minidom import Entity
obj = object.__new__(Entity)
try:
    obj.replaceChild(_W(), None)  # newChild: _EntityChildren <- wrong-typed
    print("no_typeerror:")  # CPython accepted the wrong-typed arg; mamba must raise
except TypeError as e:
    print("typeerror:", type(e).__name__)
except Exception as e:
    print("setup_or_other:", type(e).__name__)
"###);
    assert!(out == "STRICT_TYPE_REJECTED" || out.starts_with("RUNTIME_REJECTED"),
        "type wall did not hold: {out}");
}

/// Ported from `tests/cpython/type/std-libs/xml_dom_minidom/NamedNodeMap____contains____key_as_typed_wrong.py`.
#[test]
fn test_gen_type_std_libs_xml_dom_minidom_NamedNodeMap____contains____key_as_typed_wrong() {
    let out = run_type_wall_fixture(r###"# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "xml_dom_minidom"
# dimension = "type"
# case = "NamedNodeMap____contains____key_as_typed_wrong"
# subject = "xml.dom.minidom.NamedNodeMap.__contains__(key: typed)"
# kind = "semantic"
# xfail = ""
# mem_carveout = ""
# source = "vendor/typeshed/stdlib/xml/dom/minidom.pyi"
# status = "filled"
# ///
# mamba-strict-type: TypeError
"""Type wall: xml.dom.minidom.NamedNodeMap.__contains__(key: typed); call it with the wrong type.

typeshed contract: key is typed. mamba is force-typed, so a wrong-typed
argument MUST raise TypeError (CPython may accept or raise — mamba's to enforce)."""

class _W:
    pass


from xml.dom.minidom import NamedNodeMap
obj = object.__new__(NamedNodeMap)
try:
    obj.__contains__(_W())  # key: typed <- wrong-typed
    print("no_typeerror:")  # CPython accepted the wrong-typed arg; mamba must raise
except TypeError as e:
    print("typeerror:", type(e).__name__)
except Exception as e:
    print("setup_or_other:", type(e).__name__)
"###);
    assert!(out == "STRICT_TYPE_REJECTED" || out.starts_with("RUNTIME_REJECTED"),
        "type wall did not hold: {out}");
}

/// Ported from `tests/cpython/type/std-libs/xml_dom_minidom/NamedNodeMap____delitem____attname_or_tuple_as_typed_wrong.py`.
#[test]
fn test_gen_type_std_libs_xml_dom_minidom_NamedNodeMap____delitem____attname_or_tuple_as_typed_wrong() {
    let out = run_type_wall_fixture(r###"# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "xml_dom_minidom"
# dimension = "type"
# case = "NamedNodeMap____delitem____attname_or_tuple_as_typed_wrong"
# subject = "xml.dom.minidom.NamedNodeMap.__delitem__(attname_or_tuple: typed)"
# kind = "semantic"
# xfail = ""
# mem_carveout = ""
# source = "vendor/typeshed/stdlib/xml/dom/minidom.pyi"
# status = "filled"
# ///
# mamba-strict-type: TypeError
"""Type wall: xml.dom.minidom.NamedNodeMap.__delitem__(attname_or_tuple: typed); call it with the wrong type.

typeshed contract: attname_or_tuple is typed. mamba is force-typed, so a wrong-typed
argument MUST raise TypeError (CPython may accept or raise — mamba's to enforce)."""

class _W:
    pass


from xml.dom.minidom import NamedNodeMap
obj = object.__new__(NamedNodeMap)
try:
    obj.__delitem__(_W())  # attname_or_tuple: typed <- wrong-typed
    print("no_typeerror:")  # CPython accepted the wrong-typed arg; mamba must raise
except TypeError as e:
    print("typeerror:", type(e).__name__)
except Exception as e:
    print("setup_or_other:", type(e).__name__)
"###);
    assert!(out == "STRICT_TYPE_REJECTED" || out.starts_with("RUNTIME_REJECTED"),
        "type wall did not hold: {out}");
}

/// Ported from `tests/cpython/type/std-libs/xml_dom_minidom/NamedNodeMap____ge____other_as_NamedNodeMap_wrong.py`.
#[test]
fn test_gen_type_std_libs_xml_dom_minidom_NamedNodeMap____ge____other_as_NamedNodeMap_wrong() {
    let out = run_type_wall_fixture(r###"# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "xml_dom_minidom"
# dimension = "type"
# case = "NamedNodeMap____ge____other_as_NamedNodeMap_wrong"
# subject = "xml.dom.minidom.NamedNodeMap.__ge__(other: NamedNodeMap)"
# kind = "semantic"
# xfail = ""
# mem_carveout = ""
# source = "vendor/typeshed/stdlib/xml/dom/minidom.pyi"
# status = "filled"
# ///
# mamba-strict-type: TypeError
"""Type wall: xml.dom.minidom.NamedNodeMap.__ge__(other: NamedNodeMap); call it with the wrong type.

typeshed contract: other is NamedNodeMap. mamba is force-typed, so a wrong-typed
argument MUST raise TypeError (CPython may accept or raise — mamba's to enforce)."""

class _W:
    pass


from xml.dom.minidom import NamedNodeMap
obj = object.__new__(NamedNodeMap)
try:
    obj.__ge__(_W())  # other: NamedNodeMap <- wrong-typed
    print("no_typeerror:")  # CPython accepted the wrong-typed arg; mamba must raise
except TypeError as e:
    print("typeerror:", type(e).__name__)
except Exception as e:
    print("setup_or_other:", type(e).__name__)
"###);
    assert!(out == "STRICT_TYPE_REJECTED" || out.starts_with("RUNTIME_REJECTED"),
        "type wall did not hold: {out}");
}

/// Ported from `tests/cpython/type/std-libs/xml_dom_minidom/NamedNodeMap____getitem____attname_or_tuple_as_typed_wrong.py`.
#[test]
fn test_gen_type_std_libs_xml_dom_minidom_NamedNodeMap____getitem____attname_or_tuple_as_typed_wrong() {
    let out = run_type_wall_fixture(r###"# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "xml_dom_minidom"
# dimension = "type"
# case = "NamedNodeMap____getitem____attname_or_tuple_as_typed_wrong"
# subject = "xml.dom.minidom.NamedNodeMap.__getitem__(attname_or_tuple: typed)"
# kind = "semantic"
# xfail = ""
# mem_carveout = ""
# source = "vendor/typeshed/stdlib/xml/dom/minidom.pyi"
# status = "filled"
# ///
# mamba-strict-type: TypeError
"""Type wall: xml.dom.minidom.NamedNodeMap.__getitem__(attname_or_tuple: typed); call it with the wrong type.

typeshed contract: attname_or_tuple is typed. mamba is force-typed, so a wrong-typed
argument MUST raise TypeError (CPython may accept or raise — mamba's to enforce)."""

class _W:
    pass


from xml.dom.minidom import NamedNodeMap
obj = object.__new__(NamedNodeMap)
try:
    obj.__getitem__(_W())  # attname_or_tuple: typed <- wrong-typed
    print("no_typeerror:")  # CPython accepted the wrong-typed arg; mamba must raise
except TypeError as e:
    print("typeerror:", type(e).__name__)
except Exception as e:
    print("setup_or_other:", type(e).__name__)
"###);
    assert!(out == "STRICT_TYPE_REJECTED" || out.starts_with("RUNTIME_REJECTED"),
        "type wall did not hold: {out}");
}

/// Ported from `tests/cpython/type/std-libs/xml_dom_minidom/NamedNodeMap____gt____other_as_NamedNodeMap_wrong.py`.
#[test]
fn test_gen_type_std_libs_xml_dom_minidom_NamedNodeMap____gt____other_as_NamedNodeMap_wrong() {
    let out = run_type_wall_fixture(r###"# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "xml_dom_minidom"
# dimension = "type"
# case = "NamedNodeMap____gt____other_as_NamedNodeMap_wrong"
# subject = "xml.dom.minidom.NamedNodeMap.__gt__(other: NamedNodeMap)"
# kind = "semantic"
# xfail = ""
# mem_carveout = ""
# source = "vendor/typeshed/stdlib/xml/dom/minidom.pyi"
# status = "filled"
# ///
# mamba-strict-type: TypeError
"""Type wall: xml.dom.minidom.NamedNodeMap.__gt__(other: NamedNodeMap); call it with the wrong type.

typeshed contract: other is NamedNodeMap. mamba is force-typed, so a wrong-typed
argument MUST raise TypeError (CPython may accept or raise — mamba's to enforce)."""

class _W:
    pass


from xml.dom.minidom import NamedNodeMap
obj = object.__new__(NamedNodeMap)
try:
    obj.__gt__(_W())  # other: NamedNodeMap <- wrong-typed
    print("no_typeerror:")  # CPython accepted the wrong-typed arg; mamba must raise
except TypeError as e:
    print("typeerror:", type(e).__name__)
except Exception as e:
    print("setup_or_other:", type(e).__name__)
"###);
    assert!(out == "STRICT_TYPE_REJECTED" || out.starts_with("RUNTIME_REJECTED"),
        "type wall did not hold: {out}");
}

/// Ported from `tests/cpython/type/std-libs/xml_dom_minidom/NamedNodeMap____le____other_as_NamedNodeMap_wrong.py`.
#[test]
fn test_gen_type_std_libs_xml_dom_minidom_NamedNodeMap____le____other_as_NamedNodeMap_wrong() {
    let out = run_type_wall_fixture(r###"# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "xml_dom_minidom"
# dimension = "type"
# case = "NamedNodeMap____le____other_as_NamedNodeMap_wrong"
# subject = "xml.dom.minidom.NamedNodeMap.__le__(other: NamedNodeMap)"
# kind = "semantic"
# xfail = ""
# mem_carveout = ""
# source = "vendor/typeshed/stdlib/xml/dom/minidom.pyi"
# status = "filled"
# ///
# mamba-strict-type: TypeError
"""Type wall: xml.dom.minidom.NamedNodeMap.__le__(other: NamedNodeMap); call it with the wrong type.

typeshed contract: other is NamedNodeMap. mamba is force-typed, so a wrong-typed
argument MUST raise TypeError (CPython may accept or raise — mamba's to enforce)."""

class _W:
    pass


from xml.dom.minidom import NamedNodeMap
obj = object.__new__(NamedNodeMap)
try:
    obj.__le__(_W())  # other: NamedNodeMap <- wrong-typed
    print("no_typeerror:")  # CPython accepted the wrong-typed arg; mamba must raise
except TypeError as e:
    print("typeerror:", type(e).__name__)
except Exception as e:
    print("setup_or_other:", type(e).__name__)
"###);
    assert!(out == "STRICT_TYPE_REJECTED" || out.starts_with("RUNTIME_REJECTED"),
        "type wall did not hold: {out}");
}

/// Ported from `tests/cpython/type/std-libs/xml_dom_minidom/NamedNodeMap____lt____other_as_NamedNodeMap_wrong.py`.
#[test]
fn test_gen_type_std_libs_xml_dom_minidom_NamedNodeMap____lt____other_as_NamedNodeMap_wrong() {
    let out = run_type_wall_fixture(r###"# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "xml_dom_minidom"
# dimension = "type"
# case = "NamedNodeMap____lt____other_as_NamedNodeMap_wrong"
# subject = "xml.dom.minidom.NamedNodeMap.__lt__(other: NamedNodeMap)"
# kind = "semantic"
# xfail = ""
# mem_carveout = ""
# source = "vendor/typeshed/stdlib/xml/dom/minidom.pyi"
# status = "filled"
# ///
# mamba-strict-type: TypeError
"""Type wall: xml.dom.minidom.NamedNodeMap.__lt__(other: NamedNodeMap); call it with the wrong type.

typeshed contract: other is NamedNodeMap. mamba is force-typed, so a wrong-typed
argument MUST raise TypeError (CPython may accept or raise — mamba's to enforce)."""

class _W:
    pass


from xml.dom.minidom import NamedNodeMap
obj = object.__new__(NamedNodeMap)
try:
    obj.__lt__(_W())  # other: NamedNodeMap <- wrong-typed
    print("no_typeerror:")  # CPython accepted the wrong-typed arg; mamba must raise
except TypeError as e:
    print("typeerror:", type(e).__name__)
except Exception as e:
    print("setup_or_other:", type(e).__name__)
"###);
    assert!(out == "STRICT_TYPE_REJECTED" || out.starts_with("RUNTIME_REJECTED"),
        "type wall did not hold: {out}");
}

/// Ported from `tests/cpython/type/std-libs/xml_dom_minidom/NamedNodeMap____setitem____attname_as_str_wrong.py`.
#[test]
fn test_gen_type_std_libs_xml_dom_minidom_NamedNodeMap____setitem____attname_as_str_wrong() {
    let out = run_type_wall_fixture(r###"# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "xml_dom_minidom"
# dimension = "type"
# case = "NamedNodeMap____setitem____attname_as_str_wrong"
# subject = "xml.dom.minidom.NamedNodeMap.__setitem__(attname: str)"
# kind = "semantic"
# xfail = ""
# mem_carveout = ""
# source = "vendor/typeshed/stdlib/xml/dom/minidom.pyi"
# status = "filled"
# ///
# mamba-strict-type: TypeError
"""Type wall: xml.dom.minidom.NamedNodeMap.__setitem__(attname: str); call it with the wrong type.

typeshed contract: attname is str. mamba is force-typed, so a wrong-typed
argument MUST raise TypeError (CPython may accept or raise — mamba's to enforce)."""

from xml.dom.minidom import NamedNodeMap
obj = object.__new__(NamedNodeMap)
try:
    obj.__setitem__(12345, None)  # attname: str <- wrong-typed
    print("no_typeerror:")  # CPython accepted the wrong-typed arg; mamba must raise
except TypeError as e:
    print("typeerror:", type(e).__name__)
except Exception as e:
    print("setup_or_other:", type(e).__name__)
"###);
    assert!(out == "STRICT_TYPE_REJECTED" || out.starts_with("RUNTIME_REJECTED"),
        "type wall did not hold: {out}");
}

/// Ported from `tests/cpython/type/std-libs/xml_dom_minidom/NamedNodeMap__getNamedItemNS__namespaceURI_as_typed_wrong.py`.
#[test]
fn test_gen_type_std_libs_xml_dom_minidom_NamedNodeMap__getNamedItemNS__namespaceURI_as_typed_wrong() {
    let out = run_type_wall_fixture(r###"# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "xml_dom_minidom"
# dimension = "type"
# case = "NamedNodeMap__getNamedItemNS__namespaceURI_as_typed_wrong"
# subject = "xml.dom.minidom.NamedNodeMap.getNamedItemNS(namespaceURI: typed)"
# kind = "semantic"
# xfail = ""
# mem_carveout = ""
# source = "vendor/typeshed/stdlib/xml/dom/minidom.pyi"
# status = "filled"
# ///
# mamba-strict-type: TypeError
"""Type wall: xml.dom.minidom.NamedNodeMap.getNamedItemNS(namespaceURI: typed); call it with the wrong type.

typeshed contract: namespaceURI is typed. mamba is force-typed, so a wrong-typed
argument MUST raise TypeError (CPython may accept or raise — mamba's to enforce)."""

class _W:
    pass


from xml.dom.minidom import NamedNodeMap
obj = object.__new__(NamedNodeMap)
try:
    obj.getNamedItemNS(_W(), "")  # namespaceURI: typed <- wrong-typed
    print("no_typeerror:")  # CPython accepted the wrong-typed arg; mamba must raise
except TypeError as e:
    print("typeerror:", type(e).__name__)
except Exception as e:
    print("setup_or_other:", type(e).__name__)
"###);
    assert!(out == "STRICT_TYPE_REJECTED" || out.starts_with("RUNTIME_REJECTED"),
        "type wall did not hold: {out}");
}

/// Ported from `tests/cpython/type/std-libs/xml_dom_minidom/NamedNodeMap__getNamedItem__name_as_str_wrong.py`.
#[test]
fn test_gen_type_std_libs_xml_dom_minidom_NamedNodeMap__getNamedItem__name_as_str_wrong() {
    let out = run_type_wall_fixture(r###"# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "xml_dom_minidom"
# dimension = "type"
# case = "NamedNodeMap__getNamedItem__name_as_str_wrong"
# subject = "xml.dom.minidom.NamedNodeMap.getNamedItem(name: str)"
# kind = "semantic"
# xfail = ""
# mem_carveout = ""
# source = "vendor/typeshed/stdlib/xml/dom/minidom.pyi"
# status = "filled"
# ///
# mamba-strict-type: TypeError
"""Type wall: xml.dom.minidom.NamedNodeMap.getNamedItem(name: str); call it with the wrong type.

typeshed contract: name is str. mamba is force-typed, so a wrong-typed
argument MUST raise TypeError (CPython may accept or raise — mamba's to enforce)."""

from xml.dom.minidom import NamedNodeMap
obj = object.__new__(NamedNodeMap)
try:
    obj.getNamedItem(12345)  # name: str <- wrong-typed
    print("no_typeerror:")  # CPython accepted the wrong-typed arg; mamba must raise
except TypeError as e:
    print("typeerror:", type(e).__name__)
except Exception as e:
    print("setup_or_other:", type(e).__name__)
"###);
    assert!(out == "STRICT_TYPE_REJECTED" || out.starts_with("RUNTIME_REJECTED"),
        "type wall did not hold: {out}");
}

/// Ported from `tests/cpython/type/std-libs/xml_dom_minidom/NamedNodeMap__get__name_as_str_wrong.py`.
#[test]
fn test_gen_type_std_libs_xml_dom_minidom_NamedNodeMap__get__name_as_str_wrong() {
    let out = run_type_wall_fixture(r###"# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "xml_dom_minidom"
# dimension = "type"
# case = "NamedNodeMap__get__name_as_str_wrong"
# subject = "xml.dom.minidom.NamedNodeMap.get(name: str)"
# kind = "semantic"
# xfail = ""
# mem_carveout = ""
# source = "vendor/typeshed/stdlib/xml/dom/minidom.pyi"
# status = "filled"
# ///
# mamba-strict-type: TypeError
"""Type wall: xml.dom.minidom.NamedNodeMap.get(name: str); call it with the wrong type.

typeshed contract: name is str. mamba is force-typed, so a wrong-typed
argument MUST raise TypeError (CPython may accept or raise — mamba's to enforce)."""

from xml.dom.minidom import NamedNodeMap
obj = object.__new__(NamedNodeMap)
try:
    obj.get(12345)  # name: str <- wrong-typed
    print("no_typeerror:")  # CPython accepted the wrong-typed arg; mamba must raise
except TypeError as e:
    print("typeerror:", type(e).__name__)
except Exception as e:
    print("setup_or_other:", type(e).__name__)
"###);
    assert!(out == "STRICT_TYPE_REJECTED" || out.starts_with("RUNTIME_REJECTED"),
        "type wall did not hold: {out}");
}

/// Ported from `tests/cpython/type/std-libs/xml_dom_minidom/NamedNodeMap__item__index_as_int_wrong.py`.
#[test]
fn test_gen_type_std_libs_xml_dom_minidom_NamedNodeMap__item__index_as_int_wrong() {
    let out = run_type_wall_fixture(r###"# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "xml_dom_minidom"
# dimension = "type"
# case = "NamedNodeMap__item__index_as_int_wrong"
# subject = "xml.dom.minidom.NamedNodeMap.item(index: int)"
# kind = "semantic"
# xfail = ""
# mem_carveout = ""
# source = "vendor/typeshed/stdlib/xml/dom/minidom.pyi"
# status = "filled"
# ///
# mamba-strict-type: TypeError
"""Type wall: xml.dom.minidom.NamedNodeMap.item(index: int); call it with the wrong type.

typeshed contract: index is int. mamba is force-typed, so a wrong-typed
argument MUST raise TypeError (CPython may accept or raise — mamba's to enforce)."""

from xml.dom.minidom import NamedNodeMap
obj = object.__new__(NamedNodeMap)
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

/// Ported from `tests/cpython/type/std-libs/xml_dom_minidom/NamedNodeMap__removeNamedItemNS__namespaceURI_as_typed_wrong.py`.
#[test]
fn test_gen_type_std_libs_xml_dom_minidom_NamedNodeMap__removeNamedItemNS__namespaceURI_as_typed_wrong() {
    let out = run_type_wall_fixture(r###"# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "xml_dom_minidom"
# dimension = "type"
# case = "NamedNodeMap__removeNamedItemNS__namespaceURI_as_typed_wrong"
# subject = "xml.dom.minidom.NamedNodeMap.removeNamedItemNS(namespaceURI: typed)"
# kind = "semantic"
# xfail = ""
# mem_carveout = ""
# source = "vendor/typeshed/stdlib/xml/dom/minidom.pyi"
# status = "filled"
# ///
# mamba-strict-type: TypeError
"""Type wall: xml.dom.minidom.NamedNodeMap.removeNamedItemNS(namespaceURI: typed); call it with the wrong type.

typeshed contract: namespaceURI is typed. mamba is force-typed, so a wrong-typed
argument MUST raise TypeError (CPython may accept or raise — mamba's to enforce)."""

class _W:
    pass


from xml.dom.minidom import NamedNodeMap
obj = object.__new__(NamedNodeMap)
try:
    obj.removeNamedItemNS(_W(), "")  # namespaceURI: typed <- wrong-typed
    print("no_typeerror:")  # CPython accepted the wrong-typed arg; mamba must raise
except TypeError as e:
    print("typeerror:", type(e).__name__)
except Exception as e:
    print("setup_or_other:", type(e).__name__)
"###);
    assert!(out == "STRICT_TYPE_REJECTED" || out.starts_with("RUNTIME_REJECTED"),
        "type wall did not hold: {out}");
}

/// Ported from `tests/cpython/type/std-libs/xml_dom_minidom/NamedNodeMap__removeNamedItem__name_as_str_wrong.py`.
#[test]
fn test_gen_type_std_libs_xml_dom_minidom_NamedNodeMap__removeNamedItem__name_as_str_wrong() {
    let out = run_type_wall_fixture(r###"# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "xml_dom_minidom"
# dimension = "type"
# case = "NamedNodeMap__removeNamedItem__name_as_str_wrong"
# subject = "xml.dom.minidom.NamedNodeMap.removeNamedItem(name: str)"
# kind = "semantic"
# xfail = ""
# mem_carveout = ""
# source = "vendor/typeshed/stdlib/xml/dom/minidom.pyi"
# status = "filled"
# ///
# mamba-strict-type: TypeError
"""Type wall: xml.dom.minidom.NamedNodeMap.removeNamedItem(name: str); call it with the wrong type.

typeshed contract: name is str. mamba is force-typed, so a wrong-typed
argument MUST raise TypeError (CPython may accept or raise — mamba's to enforce)."""

from xml.dom.minidom import NamedNodeMap
obj = object.__new__(NamedNodeMap)
try:
    obj.removeNamedItem(12345)  # name: str <- wrong-typed
    print("no_typeerror:")  # CPython accepted the wrong-typed arg; mamba must raise
except TypeError as e:
    print("typeerror:", type(e).__name__)
except Exception as e:
    print("setup_or_other:", type(e).__name__)
"###);
    assert!(out == "STRICT_TYPE_REJECTED" || out.starts_with("RUNTIME_REJECTED"),
        "type wall did not hold: {out}");
}

/// Ported from `tests/cpython/type/std-libs/xml_dom_minidom/NamedNodeMap__setNamedItemNS__node_as_Attr_wrong.py`.
#[test]
fn test_gen_type_std_libs_xml_dom_minidom_NamedNodeMap__setNamedItemNS__node_as_Attr_wrong() {
    let out = run_type_wall_fixture(r###"# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "xml_dom_minidom"
# dimension = "type"
# case = "NamedNodeMap__setNamedItemNS__node_as_Attr_wrong"
# subject = "xml.dom.minidom.NamedNodeMap.setNamedItemNS(node: Attr)"
# kind = "semantic"
# xfail = ""
# mem_carveout = ""
# source = "vendor/typeshed/stdlib/xml/dom/minidom.pyi"
# status = "filled"
# ///
# mamba-strict-type: TypeError
"""Type wall: xml.dom.minidom.NamedNodeMap.setNamedItemNS(node: Attr); call it with the wrong type.

typeshed contract: node is Attr. mamba is force-typed, so a wrong-typed
argument MUST raise TypeError (CPython may accept or raise — mamba's to enforce)."""

class _W:
    pass


from xml.dom.minidom import NamedNodeMap
obj = object.__new__(NamedNodeMap)
try:
    obj.setNamedItemNS(_W())  # node: Attr <- wrong-typed
    print("no_typeerror:")  # CPython accepted the wrong-typed arg; mamba must raise
except TypeError as e:
    print("typeerror:", type(e).__name__)
except Exception as e:
    print("setup_or_other:", type(e).__name__)
"###);
    assert!(out == "STRICT_TYPE_REJECTED" || out.starts_with("RUNTIME_REJECTED"),
        "type wall did not hold: {out}");
}

/// Ported from `tests/cpython/type/std-libs/xml_dom_minidom/NamedNodeMap__setNamedItem__node_as_Attr_wrong.py`.
#[test]
fn test_gen_type_std_libs_xml_dom_minidom_NamedNodeMap__setNamedItem__node_as_Attr_wrong() {
    let out = run_type_wall_fixture(r###"# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "xml_dom_minidom"
# dimension = "type"
# case = "NamedNodeMap__setNamedItem__node_as_Attr_wrong"
# subject = "xml.dom.minidom.NamedNodeMap.setNamedItem(node: Attr)"
# kind = "semantic"
# xfail = ""
# mem_carveout = ""
# source = "vendor/typeshed/stdlib/xml/dom/minidom.pyi"
# status = "filled"
# ///
# mamba-strict-type: TypeError
"""Type wall: xml.dom.minidom.NamedNodeMap.setNamedItem(node: Attr); call it with the wrong type.

typeshed contract: node is Attr. mamba is force-typed, so a wrong-typed
argument MUST raise TypeError (CPython may accept or raise — mamba's to enforce)."""

class _W:
    pass


from xml.dom.minidom import NamedNodeMap
obj = object.__new__(NamedNodeMap)
try:
    obj.setNamedItem(_W())  # node: Attr <- wrong-typed
    print("no_typeerror:")  # CPython accepted the wrong-typed arg; mamba must raise
except TypeError as e:
    print("typeerror:", type(e).__name__)
except Exception as e:
    print("setup_or_other:", type(e).__name__)
"###);
    assert!(out == "STRICT_TYPE_REJECTED" || out.starts_with("RUNTIME_REJECTED"),
        "type wall did not hold: {out}");
}

/// Ported from `tests/cpython/type/std-libs/xml_dom_minidom/Node__getInterface__feature_as_str_wrong.py`.
#[test]
fn test_gen_type_std_libs_xml_dom_minidom_Node__getInterface__feature_as_str_wrong() {
    let out = run_type_wall_fixture(r###"# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "xml_dom_minidom"
# dimension = "type"
# case = "Node__getInterface__feature_as_str_wrong"
# subject = "xml.dom.minidom.Node.getInterface(feature: str)"
# kind = "semantic"
# xfail = ""
# mem_carveout = ""
# source = "vendor/typeshed/stdlib/xml/dom/minidom.pyi"
# status = "filled"
# ///
# mamba-strict-type: TypeError
"""Type wall: xml.dom.minidom.Node.getInterface(feature: str); call it with the wrong type.

typeshed contract: feature is str. mamba is force-typed, so a wrong-typed
argument MUST raise TypeError (CPython may accept or raise — mamba's to enforce)."""

from xml.dom.minidom import Node
obj = object.__new__(Node)
try:
    obj.getInterface(12345)  # feature: str <- wrong-typed
    print("no_typeerror:")  # CPython accepted the wrong-typed arg; mamba must raise
except TypeError as e:
    print("typeerror:", type(e).__name__)
except Exception as e:
    print("setup_or_other:", type(e).__name__)
"###);
    assert!(out == "STRICT_TYPE_REJECTED" || out.starts_with("RUNTIME_REJECTED"),
        "type wall did not hold: {out}");
}

/// Ported from `tests/cpython/type/std-libs/xml_dom_minidom/Node__getUserData__key_as_str_wrong.py`.
#[test]
fn test_gen_type_std_libs_xml_dom_minidom_Node__getUserData__key_as_str_wrong() {
    let out = run_type_wall_fixture(r###"# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "xml_dom_minidom"
# dimension = "type"
# case = "Node__getUserData__key_as_str_wrong"
# subject = "xml.dom.minidom.Node.getUserData(key: str)"
# kind = "semantic"
# xfail = ""
# mem_carveout = ""
# source = "vendor/typeshed/stdlib/xml/dom/minidom.pyi"
# status = "filled"
# ///
# mamba-strict-type: TypeError
"""Type wall: xml.dom.minidom.Node.getUserData(key: str); call it with the wrong type.

typeshed contract: key is str. mamba is force-typed, so a wrong-typed
argument MUST raise TypeError (CPython may accept or raise — mamba's to enforce)."""

from xml.dom.minidom import Node
obj = object.__new__(Node)
try:
    obj.getUserData(12345)  # key: str <- wrong-typed
    print("no_typeerror:")  # CPython accepted the wrong-typed arg; mamba must raise
except TypeError as e:
    print("typeerror:", type(e).__name__)
except Exception as e:
    print("setup_or_other:", type(e).__name__)
"###);
    assert!(out == "STRICT_TYPE_REJECTED" || out.starts_with("RUNTIME_REJECTED"),
        "type wall did not hold: {out}");
}

/// Ported from `tests/cpython/type/std-libs/xml_dom_minidom/Node__isSameNode__other_as_Node_wrong.py`.
#[test]
fn test_gen_type_std_libs_xml_dom_minidom_Node__isSameNode__other_as_Node_wrong() {
    let out = run_type_wall_fixture(r###"# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "xml_dom_minidom"
# dimension = "type"
# case = "Node__isSameNode__other_as_Node_wrong"
# subject = "xml.dom.minidom.Node.isSameNode(other: Node)"
# kind = "semantic"
# xfail = ""
# mem_carveout = ""
# source = "vendor/typeshed/stdlib/xml/dom/minidom.pyi"
# status = "filled"
# ///
# mamba-strict-type: TypeError
"""Type wall: xml.dom.minidom.Node.isSameNode(other: Node); call it with the wrong type.

typeshed contract: other is Node. mamba is force-typed, so a wrong-typed
argument MUST raise TypeError (CPython may accept or raise — mamba's to enforce)."""

class _W:
    pass


from xml.dom.minidom import Node
obj = object.__new__(Node)
try:
    obj.isSameNode(_W())  # other: Node <- wrong-typed
    print("no_typeerror:")  # CPython accepted the wrong-typed arg; mamba must raise
except TypeError as e:
    print("typeerror:", type(e).__name__)
except Exception as e:
    print("setup_or_other:", type(e).__name__)
"###);
    assert!(out == "STRICT_TYPE_REJECTED" || out.starts_with("RUNTIME_REJECTED"),
        "type wall did not hold: {out}");
}

/// Ported from `tests/cpython/type/std-libs/xml_dom_minidom/Node__isSupported__feature_as_str_wrong.py`.
#[test]
fn test_gen_type_std_libs_xml_dom_minidom_Node__isSupported__feature_as_str_wrong() {
    let out = run_type_wall_fixture(r###"# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "xml_dom_minidom"
# dimension = "type"
# case = "Node__isSupported__feature_as_str_wrong"
# subject = "xml.dom.minidom.Node.isSupported(feature: str)"
# kind = "semantic"
# xfail = ""
# mem_carveout = ""
# source = "vendor/typeshed/stdlib/xml/dom/minidom.pyi"
# status = "filled"
# ///
# mamba-strict-type: TypeError
"""Type wall: xml.dom.minidom.Node.isSupported(feature: str); call it with the wrong type.

typeshed contract: feature is str. mamba is force-typed, so a wrong-typed
argument MUST raise TypeError (CPython may accept or raise — mamba's to enforce)."""

from xml.dom.minidom import Node
obj = object.__new__(Node)
try:
    obj.isSupported(12345, None)  # feature: str <- wrong-typed
    print("no_typeerror:")  # CPython accepted the wrong-typed arg; mamba must raise
except TypeError as e:
    print("typeerror:", type(e).__name__)
except Exception as e:
    print("setup_or_other:", type(e).__name__)
"###);
    assert!(out == "STRICT_TYPE_REJECTED" || out.starts_with("RUNTIME_REJECTED"),
        "type wall did not hold: {out}");
}

/// Ported from `tests/cpython/type/std-libs/xml_dom_minidom/Node__setUserData__key_as_str_wrong.py`.
#[test]
fn test_gen_type_std_libs_xml_dom_minidom_Node__setUserData__key_as_str_wrong() {
    let out = run_type_wall_fixture(r###"# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "xml_dom_minidom"
# dimension = "type"
# case = "Node__setUserData__key_as_str_wrong"
# subject = "xml.dom.minidom.Node.setUserData(key: str)"
# kind = "semantic"
# xfail = ""
# mem_carveout = ""
# source = "vendor/typeshed/stdlib/xml/dom/minidom.pyi"
# status = "filled"
# ///
# mamba-strict-type: TypeError
"""Type wall: xml.dom.minidom.Node.setUserData(key: str); call it with the wrong type.

typeshed contract: key is str. mamba is force-typed, so a wrong-typed
argument MUST raise TypeError (CPython may accept or raise — mamba's to enforce)."""

from xml.dom.minidom import Node
obj = object.__new__(Node)
try:
    obj.setUserData(12345, None, None)  # key: str <- wrong-typed
    print("no_typeerror:")  # CPython accepted the wrong-typed arg; mamba must raise
except TypeError as e:
    print("typeerror:", type(e).__name__)
except Exception as e:
    print("setup_or_other:", type(e).__name__)
"###);
    assert!(out == "STRICT_TYPE_REJECTED" || out.starts_with("RUNTIME_REJECTED"),
        "type wall did not hold: {out}");
}

/// Ported from `tests/cpython/type/std-libs/xml_dom_minidom/Node__toprettyxml__indent_as_str_wrong.py`.
#[test]
fn test_gen_type_std_libs_xml_dom_minidom_Node__toprettyxml__indent_as_str_wrong() {
    let out = run_type_wall_fixture(r###"# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "xml_dom_minidom"
# dimension = "type"
# case = "Node__toprettyxml__indent_as_str_wrong"
# subject = "xml.dom.minidom.Node.toprettyxml(indent: str)"
# kind = "semantic"
# xfail = ""
# mem_carveout = ""
# source = "vendor/typeshed/stdlib/xml/dom/minidom.pyi"
# status = "filled"
# ///
# mamba-strict-type: TypeError
"""Type wall: xml.dom.minidom.Node.toprettyxml(indent: str); call it with the wrong type.

typeshed contract: indent is str. mamba is force-typed, so a wrong-typed
argument MUST raise TypeError (CPython may accept or raise — mamba's to enforce)."""

from xml.dom.minidom import Node
obj = object.__new__(Node)
try:
    obj.toprettyxml(12345)  # indent: str <- wrong-typed
    print("no_typeerror:")  # CPython accepted the wrong-typed arg; mamba must raise
except TypeError as e:
    print("typeerror:", type(e).__name__)
except Exception as e:
    print("setup_or_other:", type(e).__name__)
"###);
    assert!(out == "STRICT_TYPE_REJECTED" || out.starts_with("RUNTIME_REJECTED"),
        "type wall did not hold: {out}");
}

/// Ported from `tests/cpython/type/std-libs/xml_dom_minidom/Notation__init__name_as_str_wrong.py`.
#[test]
fn test_gen_type_std_libs_xml_dom_minidom_Notation__init__name_as_str_wrong() {
    let out = run_type_wall_fixture(r###"# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "xml_dom_minidom"
# dimension = "type"
# case = "Notation__init__name_as_str_wrong"
# subject = "xml.dom.minidom.Notation.__init__(name: str)"
# kind = "semantic"
# xfail = ""
# mem_carveout = ""
# source = "vendor/typeshed/stdlib/xml/dom/minidom.pyi"
# status = "filled"
# ///
# mamba-strict-type: TypeError
"""Type wall: xml.dom.minidom.Notation.__init__(name: str); call it with the wrong type.

typeshed contract: name is str. mamba is force-typed, so a wrong-typed
argument MUST raise TypeError (CPython may accept or raise — mamba's to enforce)."""

from xml.dom.minidom import Notation
try:
    Notation(12345, None, None)  # name: str <- wrong-typed
    print("no_typeerror:")  # CPython accepted the wrong-typed arg; mamba must raise
except TypeError as e:
    print("typeerror:", type(e).__name__)
except Exception as e:
    print("setup_or_other:", type(e).__name__)
"###);
    assert!(out == "STRICT_TYPE_REJECTED" || out.starts_with("RUNTIME_REJECTED"),
        "type wall did not hold: {out}");
}

/// Ported from `tests/cpython/type/std-libs/xml_dom_minidom/ProcessingInstruction__init__target_as_str_wrong.py`.
#[test]
fn test_gen_type_std_libs_xml_dom_minidom_ProcessingInstruction__init__target_as_str_wrong() {
    let out = run_type_wall_fixture(r###"# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "xml_dom_minidom"
# dimension = "type"
# case = "ProcessingInstruction__init__target_as_str_wrong"
# subject = "xml.dom.minidom.ProcessingInstruction.__init__(target: str)"
# kind = "semantic"
# xfail = ""
# mem_carveout = ""
# source = "vendor/typeshed/stdlib/xml/dom/minidom.pyi"
# status = "filled"
# ///
# mamba-strict-type: TypeError
"""Type wall: xml.dom.minidom.ProcessingInstruction.__init__(target: str); call it with the wrong type.

typeshed contract: target is str. mamba is force-typed, so a wrong-typed
argument MUST raise TypeError (CPython may accept or raise — mamba's to enforce)."""

from xml.dom.minidom import ProcessingInstruction
try:
    ProcessingInstruction(12345, "")  # target: str <- wrong-typed
    print("no_typeerror:")  # CPython accepted the wrong-typed arg; mamba must raise
except TypeError as e:
    print("typeerror:", type(e).__name__)
except Exception as e:
    print("setup_or_other:", type(e).__name__)
"###);
    assert!(out == "STRICT_TYPE_REJECTED" || out.starts_with("RUNTIME_REJECTED"),
        "type wall did not hold: {out}");
}

/// Ported from `tests/cpython/type/std-libs/xml_dom_minidom/ProcessingInstruction__writexml__writer_as_SupportsWrite_wrong.py`.
#[test]
fn test_gen_type_std_libs_xml_dom_minidom_ProcessingInstruction__writexml__writer_as_SupportsWrite_wrong() {
    let out = run_type_wall_fixture(r###"# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "xml_dom_minidom"
# dimension = "type"
# case = "ProcessingInstruction__writexml__writer_as_SupportsWrite_wrong"
# subject = "xml.dom.minidom.ProcessingInstruction.writexml(writer: SupportsWrite)"
# kind = "semantic"
# xfail = ""
# mem_carveout = ""
# source = "vendor/typeshed/stdlib/xml/dom/minidom.pyi"
# status = "filled"
# ///
# mamba-strict-type: TypeError
"""Type wall: xml.dom.minidom.ProcessingInstruction.writexml(writer: SupportsWrite); call it with the wrong type.

typeshed contract: writer is SupportsWrite. mamba is force-typed, so a wrong-typed
argument MUST raise TypeError (CPython may accept or raise — mamba's to enforce)."""

class _W:
    pass


from xml.dom.minidom import ProcessingInstruction
obj = object.__new__(ProcessingInstruction)
try:
    obj.writexml(_W())  # writer: SupportsWrite <- wrong-typed
    print("no_typeerror:")  # CPython accepted the wrong-typed arg; mamba must raise
except TypeError as e:
    print("typeerror:", type(e).__name__)
except Exception as e:
    print("setup_or_other:", type(e).__name__)
"###);
    assert!(out == "STRICT_TYPE_REJECTED" || out.starts_with("RUNTIME_REJECTED"),
        "type wall did not hold: {out}");
}

/// Ported from `tests/cpython/type/std-libs/xml_dom_minidom/ReadOnlySequentialNamedNodeMap____getitem____name_or_tuple_as_typed_wrong.py`.
#[test]
fn test_gen_type_std_libs_xml_dom_minidom_ReadOnlySequentialNamedNodeMap____getitem____name_or_tuple_as_typed_wrong() {
    let out = run_type_wall_fixture(r###"# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "xml_dom_minidom"
# dimension = "type"
# case = "ReadOnlySequentialNamedNodeMap____getitem____name_or_tuple_as_typed_wrong"
# subject = "xml.dom.minidom.ReadOnlySequentialNamedNodeMap.__getitem__(name_or_tuple: typed)"
# kind = "semantic"
# xfail = ""
# mem_carveout = ""
# source = "vendor/typeshed/stdlib/xml/dom/minidom.pyi"
# status = "filled"
# ///
# mamba-strict-type: TypeError
"""Type wall: xml.dom.minidom.ReadOnlySequentialNamedNodeMap.__getitem__(name_or_tuple: typed); call it with the wrong type.

typeshed contract: name_or_tuple is typed. mamba is force-typed, so a wrong-typed
argument MUST raise TypeError (CPython may accept or raise — mamba's to enforce)."""

class _W:
    pass


from xml.dom.minidom import ReadOnlySequentialNamedNodeMap
obj = object.__new__(ReadOnlySequentialNamedNodeMap)
try:
    obj.__getitem__(_W())  # name_or_tuple: typed <- wrong-typed
    print("no_typeerror:")  # CPython accepted the wrong-typed arg; mamba must raise
except TypeError as e:
    print("typeerror:", type(e).__name__)
except Exception as e:
    print("setup_or_other:", type(e).__name__)
"###);
    assert!(out == "STRICT_TYPE_REJECTED" || out.starts_with("RUNTIME_REJECTED"),
        "type wall did not hold: {out}");
}

/// Ported from `tests/cpython/type/std-libs/xml_dom_minidom/ReadOnlySequentialNamedNodeMap__getNamedItemNS__namespaceURI_as_typed_wrong.py`.
#[test]
fn test_gen_type_std_libs_xml_dom_minidom_ReadOnlySequentialNamedNodeMap__getNamedItemNS__namespaceURI_as_typed_wrong() {
    let out = run_type_wall_fixture(r###"# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "xml_dom_minidom"
# dimension = "type"
# case = "ReadOnlySequentialNamedNodeMap__getNamedItemNS__namespaceURI_as_typed_wrong"
# subject = "xml.dom.minidom.ReadOnlySequentialNamedNodeMap.getNamedItemNS(namespaceURI: typed)"
# kind = "semantic"
# xfail = ""
# mem_carveout = ""
# source = "vendor/typeshed/stdlib/xml/dom/minidom.pyi"
# status = "filled"
# ///
# mamba-strict-type: TypeError
"""Type wall: xml.dom.minidom.ReadOnlySequentialNamedNodeMap.getNamedItemNS(namespaceURI: typed); call it with the wrong type.

typeshed contract: namespaceURI is typed. mamba is force-typed, so a wrong-typed
argument MUST raise TypeError (CPython may accept or raise — mamba's to enforce)."""

class _W:
    pass


from xml.dom.minidom import ReadOnlySequentialNamedNodeMap
obj = object.__new__(ReadOnlySequentialNamedNodeMap)
try:
    obj.getNamedItemNS(_W(), "")  # namespaceURI: typed <- wrong-typed
    print("no_typeerror:")  # CPython accepted the wrong-typed arg; mamba must raise
except TypeError as e:
    print("typeerror:", type(e).__name__)
except Exception as e:
    print("setup_or_other:", type(e).__name__)
"###);
    assert!(out == "STRICT_TYPE_REJECTED" || out.starts_with("RUNTIME_REJECTED"),
        "type wall did not hold: {out}");
}

/// Ported from `tests/cpython/type/std-libs/xml_dom_minidom/ReadOnlySequentialNamedNodeMap__getNamedItem__name_as_str_wrong.py`.
#[test]
fn test_gen_type_std_libs_xml_dom_minidom_ReadOnlySequentialNamedNodeMap__getNamedItem__name_as_str_wrong() {
    let out = run_type_wall_fixture(r###"# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "xml_dom_minidom"
# dimension = "type"
# case = "ReadOnlySequentialNamedNodeMap__getNamedItem__name_as_str_wrong"
# subject = "xml.dom.minidom.ReadOnlySequentialNamedNodeMap.getNamedItem(name: str)"
# kind = "semantic"
# xfail = ""
# mem_carveout = ""
# source = "vendor/typeshed/stdlib/xml/dom/minidom.pyi"
# status = "filled"
# ///
# mamba-strict-type: TypeError
"""Type wall: xml.dom.minidom.ReadOnlySequentialNamedNodeMap.getNamedItem(name: str); call it with the wrong type.

typeshed contract: name is str. mamba is force-typed, so a wrong-typed
argument MUST raise TypeError (CPython may accept or raise — mamba's to enforce)."""

from xml.dom.minidom import ReadOnlySequentialNamedNodeMap
obj = object.__new__(ReadOnlySequentialNamedNodeMap)
try:
    obj.getNamedItem(12345)  # name: str <- wrong-typed
    print("no_typeerror:")  # CPython accepted the wrong-typed arg; mamba must raise
except TypeError as e:
    print("typeerror:", type(e).__name__)
except Exception as e:
    print("setup_or_other:", type(e).__name__)
"###);
    assert!(out == "STRICT_TYPE_REJECTED" || out.starts_with("RUNTIME_REJECTED"),
        "type wall did not hold: {out}");
}

/// Ported from `tests/cpython/type/std-libs/xml_dom_minidom/ReadOnlySequentialNamedNodeMap__item__index_as_int_wrong.py`.
#[test]
fn test_gen_type_std_libs_xml_dom_minidom_ReadOnlySequentialNamedNodeMap__item__index_as_int_wrong() {
    let out = run_type_wall_fixture(r###"# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "xml_dom_minidom"
# dimension = "type"
# case = "ReadOnlySequentialNamedNodeMap__item__index_as_int_wrong"
# subject = "xml.dom.minidom.ReadOnlySequentialNamedNodeMap.item(index: int)"
# kind = "semantic"
# xfail = ""
# mem_carveout = ""
# source = "vendor/typeshed/stdlib/xml/dom/minidom.pyi"
# status = "filled"
# ///
# mamba-strict-type: TypeError
"""Type wall: xml.dom.minidom.ReadOnlySequentialNamedNodeMap.item(index: int); call it with the wrong type.

typeshed contract: index is int. mamba is force-typed, so a wrong-typed
argument MUST raise TypeError (CPython may accept or raise — mamba's to enforce)."""

from xml.dom.minidom import ReadOnlySequentialNamedNodeMap
obj = object.__new__(ReadOnlySequentialNamedNodeMap)
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

/// Ported from `tests/cpython/type/std-libs/xml_dom_minidom/ReadOnlySequentialNamedNodeMap__removeNamedItemNS__namespaceURI_as_typed_wrong.py`.
#[test]
fn test_gen_type_std_libs_xml_dom_minidom_ReadOnlySequentialNamedNodeMap__removeNamedItemNS__namespaceURI_as_typed_wrong() {
    let out = run_type_wall_fixture(r###"# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "xml_dom_minidom"
# dimension = "type"
# case = "ReadOnlySequentialNamedNodeMap__removeNamedItemNS__namespaceURI_as_typed_wrong"
# subject = "xml.dom.minidom.ReadOnlySequentialNamedNodeMap.removeNamedItemNS(namespaceURI: typed)"
# kind = "semantic"
# xfail = ""
# mem_carveout = ""
# source = "vendor/typeshed/stdlib/xml/dom/minidom.pyi"
# status = "filled"
# ///
# mamba-strict-type: TypeError
"""Type wall: xml.dom.minidom.ReadOnlySequentialNamedNodeMap.removeNamedItemNS(namespaceURI: typed); call it with the wrong type.

typeshed contract: namespaceURI is typed. mamba is force-typed, so a wrong-typed
argument MUST raise TypeError (CPython may accept or raise — mamba's to enforce)."""

class _W:
    pass


from xml.dom.minidom import ReadOnlySequentialNamedNodeMap
obj = object.__new__(ReadOnlySequentialNamedNodeMap)
try:
    obj.removeNamedItemNS(_W(), "")  # namespaceURI: typed <- wrong-typed
    print("no_typeerror:")  # CPython accepted the wrong-typed arg; mamba must raise
except TypeError as e:
    print("typeerror:", type(e).__name__)
except Exception as e:
    print("setup_or_other:", type(e).__name__)
"###);
    assert!(out == "STRICT_TYPE_REJECTED" || out.starts_with("RUNTIME_REJECTED"),
        "type wall did not hold: {out}");
}

/// Ported from `tests/cpython/type/std-libs/xml_dom_minidom/ReadOnlySequentialNamedNodeMap__removeNamedItem__name_as_str_wrong.py`.
#[test]
fn test_gen_type_std_libs_xml_dom_minidom_ReadOnlySequentialNamedNodeMap__removeNamedItem__name_as_str_wrong() {
    let out = run_type_wall_fixture(r###"# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "xml_dom_minidom"
# dimension = "type"
# case = "ReadOnlySequentialNamedNodeMap__removeNamedItem__name_as_str_wrong"
# subject = "xml.dom.minidom.ReadOnlySequentialNamedNodeMap.removeNamedItem(name: str)"
# kind = "semantic"
# xfail = ""
# mem_carveout = ""
# source = "vendor/typeshed/stdlib/xml/dom/minidom.pyi"
# status = "filled"
# ///
# mamba-strict-type: TypeError
"""Type wall: xml.dom.minidom.ReadOnlySequentialNamedNodeMap.removeNamedItem(name: str); call it with the wrong type.

typeshed contract: name is str. mamba is force-typed, so a wrong-typed
argument MUST raise TypeError (CPython may accept or raise — mamba's to enforce)."""

from xml.dom.minidom import ReadOnlySequentialNamedNodeMap
obj = object.__new__(ReadOnlySequentialNamedNodeMap)
try:
    obj.removeNamedItem(12345)  # name: str <- wrong-typed
    print("no_typeerror:")  # CPython accepted the wrong-typed arg; mamba must raise
except TypeError as e:
    print("typeerror:", type(e).__name__)
except Exception as e:
    print("setup_or_other:", type(e).__name__)
"###);
    assert!(out == "STRICT_TYPE_REJECTED" || out.starts_with("RUNTIME_REJECTED"),
        "type wall did not hold: {out}");
}

/// Ported from `tests/cpython/type/std-libs/xml_dom_minidom/ReadOnlySequentialNamedNodeMap__setNamedItemNS__node_as_Node_wrong.py`.
#[test]
fn test_gen_type_std_libs_xml_dom_minidom_ReadOnlySequentialNamedNodeMap__setNamedItemNS__node_as_Node_wrong() {
    let out = run_type_wall_fixture(r###"# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "xml_dom_minidom"
# dimension = "type"
# case = "ReadOnlySequentialNamedNodeMap__setNamedItemNS__node_as_Node_wrong"
# subject = "xml.dom.minidom.ReadOnlySequentialNamedNodeMap.setNamedItemNS(node: Node)"
# kind = "semantic"
# xfail = ""
# mem_carveout = ""
# source = "vendor/typeshed/stdlib/xml/dom/minidom.pyi"
# status = "filled"
# ///
# mamba-strict-type: TypeError
"""Type wall: xml.dom.minidom.ReadOnlySequentialNamedNodeMap.setNamedItemNS(node: Node); call it with the wrong type.

typeshed contract: node is Node. mamba is force-typed, so a wrong-typed
argument MUST raise TypeError (CPython may accept or raise — mamba's to enforce)."""

class _W:
    pass


from xml.dom.minidom import ReadOnlySequentialNamedNodeMap
obj = object.__new__(ReadOnlySequentialNamedNodeMap)
try:
    obj.setNamedItemNS(_W())  # node: Node <- wrong-typed
    print("no_typeerror:")  # CPython accepted the wrong-typed arg; mamba must raise
except TypeError as e:
    print("typeerror:", type(e).__name__)
except Exception as e:
    print("setup_or_other:", type(e).__name__)
"###);
    assert!(out == "STRICT_TYPE_REJECTED" || out.starts_with("RUNTIME_REJECTED"),
        "type wall did not hold: {out}");
}

/// Ported from `tests/cpython/type/std-libs/xml_dom_minidom/ReadOnlySequentialNamedNodeMap__setNamedItem__node_as_Node_wrong.py`.
#[test]
fn test_gen_type_std_libs_xml_dom_minidom_ReadOnlySequentialNamedNodeMap__setNamedItem__node_as_Node_wrong() {
    let out = run_type_wall_fixture(r###"# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "xml_dom_minidom"
# dimension = "type"
# case = "ReadOnlySequentialNamedNodeMap__setNamedItem__node_as_Node_wrong"
# subject = "xml.dom.minidom.ReadOnlySequentialNamedNodeMap.setNamedItem(node: Node)"
# kind = "semantic"
# xfail = ""
# mem_carveout = ""
# source = "vendor/typeshed/stdlib/xml/dom/minidom.pyi"
# status = "filled"
# ///
# mamba-strict-type: TypeError
"""Type wall: xml.dom.minidom.ReadOnlySequentialNamedNodeMap.setNamedItem(node: Node); call it with the wrong type.

typeshed contract: node is Node. mamba is force-typed, so a wrong-typed
argument MUST raise TypeError (CPython may accept or raise — mamba's to enforce)."""

class _W:
    pass


from xml.dom.minidom import ReadOnlySequentialNamedNodeMap
obj = object.__new__(ReadOnlySequentialNamedNodeMap)
try:
    obj.setNamedItem(_W())  # node: Node <- wrong-typed
    print("no_typeerror:")  # CPython accepted the wrong-typed arg; mamba must raise
except TypeError as e:
    print("typeerror:", type(e).__name__)
except Exception as e:
    print("setup_or_other:", type(e).__name__)
"###);
    assert!(out == "STRICT_TYPE_REJECTED" || out.starts_with("RUNTIME_REJECTED"),
        "type wall did not hold: {out}");
}

/// Ported from `tests/cpython/type/std-libs/xml_dom_minidom/Text__replaceWholeText__content_as_str_wrong.py`.
#[test]
fn test_gen_type_std_libs_xml_dom_minidom_Text__replaceWholeText__content_as_str_wrong() {
    let out = run_type_wall_fixture(r###"# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "xml_dom_minidom"
# dimension = "type"
# case = "Text__replaceWholeText__content_as_str_wrong"
# subject = "xml.dom.minidom.Text.replaceWholeText(content: str)"
# kind = "semantic"
# xfail = ""
# mem_carveout = ""
# source = "vendor/typeshed/stdlib/xml/dom/minidom.pyi"
# status = "filled"
# ///
# mamba-strict-type: TypeError
"""Type wall: xml.dom.minidom.Text.replaceWholeText(content: str); call it with the wrong type.

typeshed contract: content is str. mamba is force-typed, so a wrong-typed
argument MUST raise TypeError (CPython may accept or raise — mamba's to enforce)."""

from xml.dom.minidom import Text
obj = object.__new__(Text)
try:
    obj.replaceWholeText(12345)  # content: str <- wrong-typed
    print("no_typeerror:")  # CPython accepted the wrong-typed arg; mamba must raise
except TypeError as e:
    print("typeerror:", type(e).__name__)
except Exception as e:
    print("setup_or_other:", type(e).__name__)
"###);
    assert!(out == "STRICT_TYPE_REJECTED" || out.starts_with("RUNTIME_REJECTED"),
        "type wall did not hold: {out}");
}

/// Ported from `tests/cpython/type/std-libs/xml_dom_minidom/Text__splitText__offset_as_int_wrong.py`.
#[test]
fn test_gen_type_std_libs_xml_dom_minidom_Text__splitText__offset_as_int_wrong() {
    let out = run_type_wall_fixture(r###"# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "xml_dom_minidom"
# dimension = "type"
# case = "Text__splitText__offset_as_int_wrong"
# subject = "xml.dom.minidom.Text.splitText(offset: int)"
# kind = "semantic"
# xfail = ""
# mem_carveout = ""
# source = "vendor/typeshed/stdlib/xml/dom/minidom.pyi"
# status = "filled"
# ///
# mamba-strict-type: TypeError
"""Type wall: xml.dom.minidom.Text.splitText(offset: int); call it with the wrong type.

typeshed contract: offset is int. mamba is force-typed, so a wrong-typed
argument MUST raise TypeError (CPython may accept or raise — mamba's to enforce)."""

from xml.dom.minidom import Text
obj = object.__new__(Text)
try:
    obj.splitText("not_an_int")  # offset: int <- wrong-typed
    print("no_typeerror:")  # CPython accepted the wrong-typed arg; mamba must raise
except TypeError as e:
    print("typeerror:", type(e).__name__)
except Exception as e:
    print("setup_or_other:", type(e).__name__)
"###);
    assert!(out == "STRICT_TYPE_REJECTED" || out.starts_with("RUNTIME_REJECTED"),
        "type wall did not hold: {out}");
}

/// Ported from `tests/cpython/type/std-libs/xml_dom_minidom/Text__writexml__writer_as_SupportsWrite_wrong.py`.
#[test]
fn test_gen_type_std_libs_xml_dom_minidom_Text__writexml__writer_as_SupportsWrite_wrong() {
    let out = run_type_wall_fixture(r###"# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "xml_dom_minidom"
# dimension = "type"
# case = "Text__writexml__writer_as_SupportsWrite_wrong"
# subject = "xml.dom.minidom.Text.writexml(writer: SupportsWrite)"
# kind = "semantic"
# xfail = ""
# mem_carveout = ""
# source = "vendor/typeshed/stdlib/xml/dom/minidom.pyi"
# status = "filled"
# ///
# mamba-strict-type: TypeError
"""Type wall: xml.dom.minidom.Text.writexml(writer: SupportsWrite); call it with the wrong type.

typeshed contract: writer is SupportsWrite. mamba is force-typed, so a wrong-typed
argument MUST raise TypeError (CPython may accept or raise — mamba's to enforce)."""

class _W:
    pass


from xml.dom.minidom import Text
obj = object.__new__(Text)
try:
    obj.writexml(_W())  # writer: SupportsWrite <- wrong-typed
    print("no_typeerror:")  # CPython accepted the wrong-typed arg; mamba must raise
except TypeError as e:
    print("typeerror:", type(e).__name__)
except Exception as e:
    print("setup_or_other:", type(e).__name__)
"###);
    assert!(out == "STRICT_TYPE_REJECTED" || out.starts_with("RUNTIME_REJECTED"),
        "type wall did not hold: {out}");
}

/// Ported from `tests/cpython/type/std-libs/xml_dom_minidom/parseString__string_as_typed_wrong.py`.
#[test]
fn test_gen_type_std_libs_xml_dom_minidom_parseString__string_as_typed_wrong() {
    let out = run_type_wall_fixture(r###"# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "xml_dom_minidom"
# dimension = "type"
# case = "parseString__string_as_typed_wrong"
# subject = "xml.dom.minidom.parseString(string: typed)"
# kind = "semantic"
# xfail = ""
# mem_carveout = ""
# source = "vendor/typeshed/stdlib/xml/dom/minidom.pyi"
# status = "filled"
# ///
# mamba-strict-type: TypeError
"""Type wall: xml.dom.minidom.parseString(string: typed); call it with the wrong type.

typeshed contract: string is typed. mamba is force-typed, so a wrong-typed
argument MUST raise TypeError (CPython may accept or raise — mamba's to enforce)."""

class _W:
    pass


from xml.dom.minidom import parseString
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

/// Ported from `tests/cpython/type/std-libs/xml_dom_minidom/parse__file_as_typed_wrong.py`.
#[test]
fn test_gen_type_std_libs_xml_dom_minidom_parse__file_as_typed_wrong() {
    let out = run_type_wall_fixture(r###"# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "xml_dom_minidom"
# dimension = "type"
# case = "parse__file_as_typed_wrong"
# subject = "xml.dom.minidom.parse(file: typed)"
# kind = "semantic"
# xfail = ""
# mem_carveout = ""
# source = "vendor/typeshed/stdlib/xml/dom/minidom.pyi"
# status = "filled"
# ///
# mamba-strict-type: TypeError
"""Type wall: xml.dom.minidom.parse(file: typed); call it with the wrong type.

typeshed contract: file is typed. mamba is force-typed, so a wrong-typed
argument MUST raise TypeError (CPython may accept or raise — mamba's to enforce)."""

class _W:
    pass


from xml.dom.minidom import parse
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
