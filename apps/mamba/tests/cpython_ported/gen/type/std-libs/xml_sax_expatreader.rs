use super::super::super::super::harness::*;

/// Ported from `tests/cpython/type/std-libs/xml_sax_expatreader/ExpatLocator__init__parser_as_ExpatParser_wrong.py`.
#[test]
fn test_gen_type_std_libs_xml_sax_expatreader_ExpatLocator__init__parser_as_ExpatParser_wrong() {
    let out = run_type_wall_fixture(r###"# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "xml_sax_expatreader"
# dimension = "type"
# case = "ExpatLocator__init__parser_as_ExpatParser_wrong"
# subject = "xml.sax.expatreader.ExpatLocator.__init__(parser: ExpatParser)"
# kind = "semantic"
# xfail = ""
# mem_carveout = ""
# source = "vendor/typeshed/stdlib/xml/sax/expatreader.pyi"
# status = "filled"
# ///
# mamba-strict-type: TypeError
"""Type wall: xml.sax.expatreader.ExpatLocator.__init__(parser: ExpatParser); call it with the wrong type.

typeshed contract: parser is ExpatParser. mamba is force-typed, so a wrong-typed
argument MUST raise TypeError (CPython may accept or raise — mamba's to enforce)."""

class _W:
    pass


from xml.sax.expatreader import ExpatLocator
try:
    ExpatLocator(_W())  # parser: ExpatParser <- wrong-typed
    print("no_typeerror:")  # CPython accepted the wrong-typed arg; mamba must raise
except TypeError as e:
    print("typeerror:", type(e).__name__)
except Exception as e:
    print("setup_or_other:", type(e).__name__)
"###);
    assert!(out == "STRICT_TYPE_REJECTED" || out.starts_with("RUNTIME_REJECTED"),
        "type wall did not hold: {out}");
}

/// Ported from `tests/cpython/type/std-libs/xml_sax_expatreader/ExpatParser__character_data__data_as_str_wrong.py`.
#[test]
fn test_gen_type_std_libs_xml_sax_expatreader_ExpatParser__character_data__data_as_str_wrong() {
    let out = run_type_wall_fixture(r###"# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "xml_sax_expatreader"
# dimension = "type"
# case = "ExpatParser__character_data__data_as_str_wrong"
# subject = "xml.sax.expatreader.ExpatParser.character_data(data: str)"
# kind = "semantic"
# xfail = ""
# mem_carveout = ""
# source = "vendor/typeshed/stdlib/xml/sax/expatreader.pyi"
# status = "filled"
# ///
# mamba-strict-type: TypeError
"""Type wall: xml.sax.expatreader.ExpatParser.character_data(data: str); call it with the wrong type.

typeshed contract: data is str. mamba is force-typed, so a wrong-typed
argument MUST raise TypeError (CPython may accept or raise — mamba's to enforce)."""

from xml.sax.expatreader import ExpatParser
obj = object.__new__(ExpatParser)
try:
    obj.character_data(12345)  # data: str <- wrong-typed
    print("no_typeerror:")  # CPython accepted the wrong-typed arg; mamba must raise
except TypeError as e:
    print("typeerror:", type(e).__name__)
except Exception as e:
    print("setup_or_other:", type(e).__name__)
"###);
    assert!(out == "STRICT_TYPE_REJECTED" || out.starts_with("RUNTIME_REJECTED"),
        "type wall did not hold: {out}");
}

/// Ported from `tests/cpython/type/std-libs/xml_sax_expatreader/ExpatParser__end_element__name_as_str_wrong.py`.
#[test]
fn test_gen_type_std_libs_xml_sax_expatreader_ExpatParser__end_element__name_as_str_wrong() {
    let out = run_type_wall_fixture(r###"# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "xml_sax_expatreader"
# dimension = "type"
# case = "ExpatParser__end_element__name_as_str_wrong"
# subject = "xml.sax.expatreader.ExpatParser.end_element(name: str)"
# kind = "semantic"
# xfail = ""
# mem_carveout = ""
# source = "vendor/typeshed/stdlib/xml/sax/expatreader.pyi"
# status = "filled"
# ///
# mamba-strict-type: TypeError
"""Type wall: xml.sax.expatreader.ExpatParser.end_element(name: str); call it with the wrong type.

typeshed contract: name is str. mamba is force-typed, so a wrong-typed
argument MUST raise TypeError (CPython may accept or raise — mamba's to enforce)."""

from xml.sax.expatreader import ExpatParser
obj = object.__new__(ExpatParser)
try:
    obj.end_element(12345)  # name: str <- wrong-typed
    print("no_typeerror:")  # CPython accepted the wrong-typed arg; mamba must raise
except TypeError as e:
    print("typeerror:", type(e).__name__)
except Exception as e:
    print("setup_or_other:", type(e).__name__)
"###);
    assert!(out == "STRICT_TYPE_REJECTED" || out.starts_with("RUNTIME_REJECTED"),
        "type wall did not hold: {out}");
}

/// Ported from `tests/cpython/type/std-libs/xml_sax_expatreader/ExpatParser__end_element_ns__name_as_str_wrong.py`.
#[test]
fn test_gen_type_std_libs_xml_sax_expatreader_ExpatParser__end_element_ns__name_as_str_wrong() {
    let out = run_type_wall_fixture(r###"# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "xml_sax_expatreader"
# dimension = "type"
# case = "ExpatParser__end_element_ns__name_as_str_wrong"
# subject = "xml.sax.expatreader.ExpatParser.end_element_ns(name: str)"
# kind = "semantic"
# xfail = ""
# mem_carveout = ""
# source = "vendor/typeshed/stdlib/xml/sax/expatreader.pyi"
# status = "filled"
# ///
# mamba-strict-type: TypeError
"""Type wall: xml.sax.expatreader.ExpatParser.end_element_ns(name: str); call it with the wrong type.

typeshed contract: name is str. mamba is force-typed, so a wrong-typed
argument MUST raise TypeError (CPython may accept or raise — mamba's to enforce)."""

from xml.sax.expatreader import ExpatParser
obj = object.__new__(ExpatParser)
try:
    obj.end_element_ns(12345)  # name: str <- wrong-typed
    print("no_typeerror:")  # CPython accepted the wrong-typed arg; mamba must raise
except TypeError as e:
    print("typeerror:", type(e).__name__)
except Exception as e:
    print("setup_or_other:", type(e).__name__)
"###);
    assert!(out == "STRICT_TYPE_REJECTED" || out.starts_with("RUNTIME_REJECTED"),
        "type wall did not hold: {out}");
}

/// Ported from `tests/cpython/type/std-libs/xml_sax_expatreader/ExpatParser__end_namespace_decl__prefix_as_typed_wrong.py`.
#[test]
fn test_gen_type_std_libs_xml_sax_expatreader_ExpatParser__end_namespace_decl__prefix_as_typed_wrong() {
    let out = run_type_wall_fixture(r###"# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "xml_sax_expatreader"
# dimension = "type"
# case = "ExpatParser__end_namespace_decl__prefix_as_typed_wrong"
# subject = "xml.sax.expatreader.ExpatParser.end_namespace_decl(prefix: typed)"
# kind = "semantic"
# xfail = ""
# mem_carveout = ""
# source = "vendor/typeshed/stdlib/xml/sax/expatreader.pyi"
# status = "filled"
# ///
# mamba-strict-type: TypeError
"""Type wall: xml.sax.expatreader.ExpatParser.end_namespace_decl(prefix: typed); call it with the wrong type.

typeshed contract: prefix is typed. mamba is force-typed, so a wrong-typed
argument MUST raise TypeError (CPython may accept or raise — mamba's to enforce)."""

class _W:
    pass


from xml.sax.expatreader import ExpatParser
obj = object.__new__(ExpatParser)
try:
    obj.end_namespace_decl(_W())  # prefix: typed <- wrong-typed
    print("no_typeerror:")  # CPython accepted the wrong-typed arg; mamba must raise
except TypeError as e:
    print("typeerror:", type(e).__name__)
except Exception as e:
    print("setup_or_other:", type(e).__name__)
"###);
    assert!(out == "STRICT_TYPE_REJECTED" || out.starts_with("RUNTIME_REJECTED"),
        "type wall did not hold: {out}");
}

/// Ported from `tests/cpython/type/std-libs/xml_sax_expatreader/ExpatParser__external_entity_ref__context_as_str_wrong.py`.
#[test]
fn test_gen_type_std_libs_xml_sax_expatreader_ExpatParser__external_entity_ref__context_as_str_wrong() {
    let out = run_type_wall_fixture(r###"# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "xml_sax_expatreader"
# dimension = "type"
# case = "ExpatParser__external_entity_ref__context_as_str_wrong"
# subject = "xml.sax.expatreader.ExpatParser.external_entity_ref(context: str)"
# kind = "semantic"
# xfail = ""
# mem_carveout = ""
# source = "vendor/typeshed/stdlib/xml/sax/expatreader.pyi"
# status = "filled"
# ///
# mamba-strict-type: TypeError
"""Type wall: xml.sax.expatreader.ExpatParser.external_entity_ref(context: str); call it with the wrong type.

typeshed contract: context is str. mamba is force-typed, so a wrong-typed
argument MUST raise TypeError (CPython may accept or raise — mamba's to enforce)."""

from xml.sax.expatreader import ExpatParser
obj = object.__new__(ExpatParser)
try:
    obj.external_entity_ref(12345, None, "", None)  # context: str <- wrong-typed
    print("no_typeerror:")  # CPython accepted the wrong-typed arg; mamba must raise
except TypeError as e:
    print("typeerror:", type(e).__name__)
except Exception as e:
    print("setup_or_other:", type(e).__name__)
"###);
    assert!(out == "STRICT_TYPE_REJECTED" || out.starts_with("RUNTIME_REJECTED"),
        "type wall did not hold: {out}");
}

/// Ported from `tests/cpython/type/std-libs/xml_sax_expatreader/ExpatParser__feed__data_as_typed_wrong.py`.
#[test]
fn test_gen_type_std_libs_xml_sax_expatreader_ExpatParser__feed__data_as_typed_wrong() {
    let out = run_type_wall_fixture(r###"# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "xml_sax_expatreader"
# dimension = "type"
# case = "ExpatParser__feed__data_as_typed_wrong"
# subject = "xml.sax.expatreader.ExpatParser.feed(data: typed)"
# kind = "semantic"
# xfail = ""
# mem_carveout = ""
# source = "vendor/typeshed/stdlib/xml/sax/expatreader.pyi"
# status = "filled"
# ///
# mamba-strict-type: TypeError
"""Type wall: xml.sax.expatreader.ExpatParser.feed(data: typed); call it with the wrong type.

typeshed contract: data is typed. mamba is force-typed, so a wrong-typed
argument MUST raise TypeError (CPython may accept or raise — mamba's to enforce)."""

class _W:
    pass


from xml.sax.expatreader import ExpatParser
obj = object.__new__(ExpatParser)
try:
    obj.feed(_W())  # data: typed <- wrong-typed
    print("no_typeerror:")  # CPython accepted the wrong-typed arg; mamba must raise
except TypeError as e:
    print("typeerror:", type(e).__name__)
except Exception as e:
    print("setup_or_other:", type(e).__name__)
"###);
    assert!(out == "STRICT_TYPE_REJECTED" || out.starts_with("RUNTIME_REJECTED"),
        "type wall did not hold: {out}");
}

/// Ported from `tests/cpython/type/std-libs/xml_sax_expatreader/ExpatParser__getFeature__name_as_str_wrong.py`.
#[test]
fn test_gen_type_std_libs_xml_sax_expatreader_ExpatParser__getFeature__name_as_str_wrong() {
    let out = run_type_wall_fixture(r###"# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "xml_sax_expatreader"
# dimension = "type"
# case = "ExpatParser__getFeature__name_as_str_wrong"
# subject = "xml.sax.expatreader.ExpatParser.getFeature(name: str)"
# kind = "semantic"
# xfail = ""
# mem_carveout = ""
# source = "vendor/typeshed/stdlib/xml/sax/expatreader.pyi"
# status = "filled"
# ///
# mamba-strict-type: TypeError
"""Type wall: xml.sax.expatreader.ExpatParser.getFeature(name: str); call it with the wrong type.

typeshed contract: name is str. mamba is force-typed, so a wrong-typed
argument MUST raise TypeError (CPython may accept or raise — mamba's to enforce)."""

from xml.sax.expatreader import ExpatParser
obj = object.__new__(ExpatParser)
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

/// Ported from `tests/cpython/type/std-libs/xml_sax_expatreader/ExpatParser__init__namespaceHandling_as__BoolType_wrong.py`.
#[test]
fn test_gen_type_std_libs_xml_sax_expatreader_ExpatParser__init__namespaceHandling_as__BoolType_wrong() {
    let out = run_type_wall_fixture(r###"# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "xml_sax_expatreader"
# dimension = "type"
# case = "ExpatParser__init__namespaceHandling_as__BoolType_wrong"
# subject = "xml.sax.expatreader.ExpatParser.__init__(namespaceHandling: _BoolType)"
# kind = "semantic"
# xfail = ""
# mem_carveout = ""
# source = "vendor/typeshed/stdlib/xml/sax/expatreader.pyi"
# status = "filled"
# ///
# mamba-strict-type: TypeError
"""Type wall: xml.sax.expatreader.ExpatParser.__init__(namespaceHandling: _BoolType); call it with the wrong type.

typeshed contract: namespaceHandling is _BoolType. mamba is force-typed, so a wrong-typed
argument MUST raise TypeError (CPython may accept or raise — mamba's to enforce)."""

class _W:
    pass


from xml.sax.expatreader import ExpatParser
try:
    ExpatParser(_W())  # namespaceHandling: _BoolType <- wrong-typed
    print("no_typeerror:")  # CPython accepted the wrong-typed arg; mamba must raise
except TypeError as e:
    print("typeerror:", type(e).__name__)
except Exception as e:
    print("setup_or_other:", type(e).__name__)
"###);
    assert!(out == "STRICT_TYPE_REJECTED" || out.starts_with("RUNTIME_REJECTED"),
        "type wall did not hold: {out}");
}

/// Ported from `tests/cpython/type/std-libs/xml_sax_expatreader/ExpatParser__notation_decl__name_as_str_wrong.py`.
#[test]
fn test_gen_type_std_libs_xml_sax_expatreader_ExpatParser__notation_decl__name_as_str_wrong() {
    let out = run_type_wall_fixture(r###"# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "xml_sax_expatreader"
# dimension = "type"
# case = "ExpatParser__notation_decl__name_as_str_wrong"
# subject = "xml.sax.expatreader.ExpatParser.notation_decl(name: str)"
# kind = "semantic"
# xfail = ""
# mem_carveout = ""
# source = "vendor/typeshed/stdlib/xml/sax/expatreader.pyi"
# status = "filled"
# ///
# mamba-strict-type: TypeError
"""Type wall: xml.sax.expatreader.ExpatParser.notation_decl(name: str); call it with the wrong type.

typeshed contract: name is str. mamba is force-typed, so a wrong-typed
argument MUST raise TypeError (CPython may accept or raise — mamba's to enforce)."""

from xml.sax.expatreader import ExpatParser
obj = object.__new__(ExpatParser)
try:
    obj.notation_decl(12345, None, "", None)  # name: str <- wrong-typed
    print("no_typeerror:")  # CPython accepted the wrong-typed arg; mamba must raise
except TypeError as e:
    print("typeerror:", type(e).__name__)
except Exception as e:
    print("setup_or_other:", type(e).__name__)
"###);
    assert!(out == "STRICT_TYPE_REJECTED" || out.starts_with("RUNTIME_REJECTED"),
        "type wall did not hold: {out}");
}

/// Ported from `tests/cpython/type/std-libs/xml_sax_expatreader/ExpatParser__parse__source_as_typed_wrong.py`.
#[test]
fn test_gen_type_std_libs_xml_sax_expatreader_ExpatParser__parse__source_as_typed_wrong() {
    let out = run_type_wall_fixture(r###"# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "xml_sax_expatreader"
# dimension = "type"
# case = "ExpatParser__parse__source_as_typed_wrong"
# subject = "xml.sax.expatreader.ExpatParser.parse(source: typed)"
# kind = "semantic"
# xfail = ""
# mem_carveout = ""
# source = "vendor/typeshed/stdlib/xml/sax/expatreader.pyi"
# status = "filled"
# ///
# mamba-strict-type: TypeError
"""Type wall: xml.sax.expatreader.ExpatParser.parse(source: typed); call it with the wrong type.

typeshed contract: source is typed. mamba is force-typed, so a wrong-typed
argument MUST raise TypeError (CPython may accept or raise — mamba's to enforce)."""

class _W:
    pass


from xml.sax.expatreader import ExpatParser
obj = object.__new__(ExpatParser)
try:
    obj.parse(_W())  # source: typed <- wrong-typed
    print("no_typeerror:")  # CPython accepted the wrong-typed arg; mamba must raise
except TypeError as e:
    print("typeerror:", type(e).__name__)
except Exception as e:
    print("setup_or_other:", type(e).__name__)
"###);
    assert!(out == "STRICT_TYPE_REJECTED" || out.starts_with("RUNTIME_REJECTED"),
        "type wall did not hold: {out}");
}

/// Ported from `tests/cpython/type/std-libs/xml_sax_expatreader/ExpatParser__prepareParser__source_as_InputSource_wrong.py`.
#[test]
fn test_gen_type_std_libs_xml_sax_expatreader_ExpatParser__prepareParser__source_as_InputSource_wrong() {
    let out = run_type_wall_fixture(r###"# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "xml_sax_expatreader"
# dimension = "type"
# case = "ExpatParser__prepareParser__source_as_InputSource_wrong"
# subject = "xml.sax.expatreader.ExpatParser.prepareParser(source: InputSource)"
# kind = "semantic"
# xfail = ""
# mem_carveout = ""
# source = "vendor/typeshed/stdlib/xml/sax/expatreader.pyi"
# status = "filled"
# ///
# mamba-strict-type: TypeError
"""Type wall: xml.sax.expatreader.ExpatParser.prepareParser(source: InputSource); call it with the wrong type.

typeshed contract: source is InputSource. mamba is force-typed, so a wrong-typed
argument MUST raise TypeError (CPython may accept or raise — mamba's to enforce)."""

class _W:
    pass


from xml.sax.expatreader import ExpatParser
obj = object.__new__(ExpatParser)
try:
    obj.prepareParser(_W())  # source: InputSource <- wrong-typed
    print("no_typeerror:")  # CPython accepted the wrong-typed arg; mamba must raise
except TypeError as e:
    print("typeerror:", type(e).__name__)
except Exception as e:
    print("setup_or_other:", type(e).__name__)
"###);
    assert!(out == "STRICT_TYPE_REJECTED" || out.starts_with("RUNTIME_REJECTED"),
        "type wall did not hold: {out}");
}

/// Ported from `tests/cpython/type/std-libs/xml_sax_expatreader/ExpatParser__processing_instruction__target_as_str_wrong.py`.
#[test]
fn test_gen_type_std_libs_xml_sax_expatreader_ExpatParser__processing_instruction__target_as_str_wrong() {
    let out = run_type_wall_fixture(r###"# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "xml_sax_expatreader"
# dimension = "type"
# case = "ExpatParser__processing_instruction__target_as_str_wrong"
# subject = "xml.sax.expatreader.ExpatParser.processing_instruction(target: str)"
# kind = "semantic"
# xfail = ""
# mem_carveout = ""
# source = "vendor/typeshed/stdlib/xml/sax/expatreader.pyi"
# status = "filled"
# ///
# mamba-strict-type: TypeError
"""Type wall: xml.sax.expatreader.ExpatParser.processing_instruction(target: str); call it with the wrong type.

typeshed contract: target is str. mamba is force-typed, so a wrong-typed
argument MUST raise TypeError (CPython may accept or raise — mamba's to enforce)."""

from xml.sax.expatreader import ExpatParser
obj = object.__new__(ExpatParser)
try:
    obj.processing_instruction(12345, "")  # target: str <- wrong-typed
    print("no_typeerror:")  # CPython accepted the wrong-typed arg; mamba must raise
except TypeError as e:
    print("typeerror:", type(e).__name__)
except Exception as e:
    print("setup_or_other:", type(e).__name__)
"###);
    assert!(out == "STRICT_TYPE_REJECTED" || out.starts_with("RUNTIME_REJECTED"),
        "type wall did not hold: {out}");
}

/// Ported from `tests/cpython/type/std-libs/xml_sax_expatreader/ExpatParser__setContentHandler__handler_as__ContentHandlerProtocol_wrong.py`.
#[test]
fn test_gen_type_std_libs_xml_sax_expatreader_ExpatParser__setContentHandler__handler_as__ContentHandlerProtocol_wrong() {
    let out = run_type_wall_fixture(r###"# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "xml_sax_expatreader"
# dimension = "type"
# case = "ExpatParser__setContentHandler__handler_as__ContentHandlerProtocol_wrong"
# subject = "xml.sax.expatreader.ExpatParser.setContentHandler(handler: _ContentHandlerProtocol)"
# kind = "semantic"
# xfail = ""
# mem_carveout = ""
# source = "vendor/typeshed/stdlib/xml/sax/expatreader.pyi"
# status = "filled"
# ///
# mamba-strict-type: TypeError
"""Type wall: xml.sax.expatreader.ExpatParser.setContentHandler(handler: _ContentHandlerProtocol); call it with the wrong type.

typeshed contract: handler is _ContentHandlerProtocol. mamba is force-typed, so a wrong-typed
argument MUST raise TypeError (CPython may accept or raise — mamba's to enforce)."""

class _W:
    pass


from xml.sax.expatreader import ExpatParser
obj = object.__new__(ExpatParser)
try:
    obj.setContentHandler(_W())  # handler: _ContentHandlerProtocol <- wrong-typed
    print("no_typeerror:")  # CPython accepted the wrong-typed arg; mamba must raise
except TypeError as e:
    print("typeerror:", type(e).__name__)
except Exception as e:
    print("setup_or_other:", type(e).__name__)
"###);
    assert!(out == "STRICT_TYPE_REJECTED" || out.starts_with("RUNTIME_REJECTED"),
        "type wall did not hold: {out}");
}

/// Ported from `tests/cpython/type/std-libs/xml_sax_expatreader/ExpatParser__setFeature__name_as_str_wrong.py`.
#[test]
fn test_gen_type_std_libs_xml_sax_expatreader_ExpatParser__setFeature__name_as_str_wrong() {
    let out = run_type_wall_fixture(r###"# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "xml_sax_expatreader"
# dimension = "type"
# case = "ExpatParser__setFeature__name_as_str_wrong"
# subject = "xml.sax.expatreader.ExpatParser.setFeature(name: str)"
# kind = "semantic"
# xfail = ""
# mem_carveout = ""
# source = "vendor/typeshed/stdlib/xml/sax/expatreader.pyi"
# status = "filled"
# ///
# mamba-strict-type: TypeError
"""Type wall: xml.sax.expatreader.ExpatParser.setFeature(name: str); call it with the wrong type.

typeshed contract: name is str. mamba is force-typed, so a wrong-typed
argument MUST raise TypeError (CPython may accept or raise — mamba's to enforce)."""

from xml.sax.expatreader import ExpatParser
obj = object.__new__(ExpatParser)
try:
    obj.setFeature(12345, None)  # name: str <- wrong-typed
    print("no_typeerror:")  # CPython accepted the wrong-typed arg; mamba must raise
except TypeError as e:
    print("typeerror:", type(e).__name__)
except Exception as e:
    print("setup_or_other:", type(e).__name__)
"###);
    assert!(out == "STRICT_TYPE_REJECTED" || out.starts_with("RUNTIME_REJECTED"),
        "type wall did not hold: {out}");
}

/// Ported from `tests/cpython/type/std-libs/xml_sax_expatreader/ExpatParser__skipped_entity_handler__name_as_str_wrong.py`.
#[test]
fn test_gen_type_std_libs_xml_sax_expatreader_ExpatParser__skipped_entity_handler__name_as_str_wrong() {
    let out = run_type_wall_fixture(r###"# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "xml_sax_expatreader"
# dimension = "type"
# case = "ExpatParser__skipped_entity_handler__name_as_str_wrong"
# subject = "xml.sax.expatreader.ExpatParser.skipped_entity_handler(name: str)"
# kind = "semantic"
# xfail = ""
# mem_carveout = ""
# source = "vendor/typeshed/stdlib/xml/sax/expatreader.pyi"
# status = "filled"
# ///
# mamba-strict-type: TypeError
"""Type wall: xml.sax.expatreader.ExpatParser.skipped_entity_handler(name: str); call it with the wrong type.

typeshed contract: name is str. mamba is force-typed, so a wrong-typed
argument MUST raise TypeError (CPython may accept or raise — mamba's to enforce)."""

from xml.sax.expatreader import ExpatParser
obj = object.__new__(ExpatParser)
try:
    obj.skipped_entity_handler(12345, True)  # name: str <- wrong-typed
    print("no_typeerror:")  # CPython accepted the wrong-typed arg; mamba must raise
except TypeError as e:
    print("typeerror:", type(e).__name__)
except Exception as e:
    print("setup_or_other:", type(e).__name__)
"###);
    assert!(out == "STRICT_TYPE_REJECTED" || out.starts_with("RUNTIME_REJECTED"),
        "type wall did not hold: {out}");
}

/// Ported from `tests/cpython/type/std-libs/xml_sax_expatreader/ExpatParser__start_doctype_decl__name_as_str_wrong.py`.
#[test]
fn test_gen_type_std_libs_xml_sax_expatreader_ExpatParser__start_doctype_decl__name_as_str_wrong() {
    let out = run_type_wall_fixture(r###"# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "xml_sax_expatreader"
# dimension = "type"
# case = "ExpatParser__start_doctype_decl__name_as_str_wrong"
# subject = "xml.sax.expatreader.ExpatParser.start_doctype_decl(name: str)"
# kind = "semantic"
# xfail = ""
# mem_carveout = ""
# source = "vendor/typeshed/stdlib/xml/sax/expatreader.pyi"
# status = "filled"
# ///
# mamba-strict-type: TypeError
"""Type wall: xml.sax.expatreader.ExpatParser.start_doctype_decl(name: str); call it with the wrong type.

typeshed contract: name is str. mamba is force-typed, so a wrong-typed
argument MUST raise TypeError (CPython may accept or raise — mamba's to enforce)."""

from xml.sax.expatreader import ExpatParser
obj = object.__new__(ExpatParser)
try:
    obj.start_doctype_decl(12345, None, None, True)  # name: str <- wrong-typed
    print("no_typeerror:")  # CPython accepted the wrong-typed arg; mamba must raise
except TypeError as e:
    print("typeerror:", type(e).__name__)
except Exception as e:
    print("setup_or_other:", type(e).__name__)
"###);
    assert!(out == "STRICT_TYPE_REJECTED" || out.starts_with("RUNTIME_REJECTED"),
        "type wall did not hold: {out}");
}

/// Ported from `tests/cpython/type/std-libs/xml_sax_expatreader/ExpatParser__start_element__name_as_str_wrong.py`.
#[test]
fn test_gen_type_std_libs_xml_sax_expatreader_ExpatParser__start_element__name_as_str_wrong() {
    let out = run_type_wall_fixture(r###"# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "xml_sax_expatreader"
# dimension = "type"
# case = "ExpatParser__start_element__name_as_str_wrong"
# subject = "xml.sax.expatreader.ExpatParser.start_element(name: str)"
# kind = "semantic"
# xfail = ""
# mem_carveout = ""
# source = "vendor/typeshed/stdlib/xml/sax/expatreader.pyi"
# status = "filled"
# ///
# mamba-strict-type: TypeError
"""Type wall: xml.sax.expatreader.ExpatParser.start_element(name: str); call it with the wrong type.

typeshed contract: name is str. mamba is force-typed, so a wrong-typed
argument MUST raise TypeError (CPython may accept or raise — mamba's to enforce)."""

from xml.sax.expatreader import ExpatParser
obj = object.__new__(ExpatParser)
try:
    obj.start_element(12345, None)  # name: str <- wrong-typed
    print("no_typeerror:")  # CPython accepted the wrong-typed arg; mamba must raise
except TypeError as e:
    print("typeerror:", type(e).__name__)
except Exception as e:
    print("setup_or_other:", type(e).__name__)
"###);
    assert!(out == "STRICT_TYPE_REJECTED" || out.starts_with("RUNTIME_REJECTED"),
        "type wall did not hold: {out}");
}

/// Ported from `tests/cpython/type/std-libs/xml_sax_expatreader/ExpatParser__start_element_ns__name_as_str_wrong.py`.
#[test]
fn test_gen_type_std_libs_xml_sax_expatreader_ExpatParser__start_element_ns__name_as_str_wrong() {
    let out = run_type_wall_fixture(r###"# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "xml_sax_expatreader"
# dimension = "type"
# case = "ExpatParser__start_element_ns__name_as_str_wrong"
# subject = "xml.sax.expatreader.ExpatParser.start_element_ns(name: str)"
# kind = "semantic"
# xfail = ""
# mem_carveout = ""
# source = "vendor/typeshed/stdlib/xml/sax/expatreader.pyi"
# status = "filled"
# ///
# mamba-strict-type: TypeError
"""Type wall: xml.sax.expatreader.ExpatParser.start_element_ns(name: str); call it with the wrong type.

typeshed contract: name is str. mamba is force-typed, so a wrong-typed
argument MUST raise TypeError (CPython may accept or raise — mamba's to enforce)."""

from xml.sax.expatreader import ExpatParser
obj = object.__new__(ExpatParser)
try:
    obj.start_element_ns(12345, None)  # name: str <- wrong-typed
    print("no_typeerror:")  # CPython accepted the wrong-typed arg; mamba must raise
except TypeError as e:
    print("typeerror:", type(e).__name__)
except Exception as e:
    print("setup_or_other:", type(e).__name__)
"###);
    assert!(out == "STRICT_TYPE_REJECTED" || out.starts_with("RUNTIME_REJECTED"),
        "type wall did not hold: {out}");
}

/// Ported from `tests/cpython/type/std-libs/xml_sax_expatreader/ExpatParser__start_namespace_decl__prefix_as_typed_wrong.py`.
#[test]
fn test_gen_type_std_libs_xml_sax_expatreader_ExpatParser__start_namespace_decl__prefix_as_typed_wrong() {
    let out = run_type_wall_fixture(r###"# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "xml_sax_expatreader"
# dimension = "type"
# case = "ExpatParser__start_namespace_decl__prefix_as_typed_wrong"
# subject = "xml.sax.expatreader.ExpatParser.start_namespace_decl(prefix: typed)"
# kind = "semantic"
# xfail = ""
# mem_carveout = ""
# source = "vendor/typeshed/stdlib/xml/sax/expatreader.pyi"
# status = "filled"
# ///
# mamba-strict-type: TypeError
"""Type wall: xml.sax.expatreader.ExpatParser.start_namespace_decl(prefix: typed); call it with the wrong type.

typeshed contract: prefix is typed. mamba is force-typed, so a wrong-typed
argument MUST raise TypeError (CPython may accept or raise — mamba's to enforce)."""

class _W:
    pass


from xml.sax.expatreader import ExpatParser
obj = object.__new__(ExpatParser)
try:
    obj.start_namespace_decl(_W(), "")  # prefix: typed <- wrong-typed
    print("no_typeerror:")  # CPython accepted the wrong-typed arg; mamba must raise
except TypeError as e:
    print("typeerror:", type(e).__name__)
except Exception as e:
    print("setup_or_other:", type(e).__name__)
"###);
    assert!(out == "STRICT_TYPE_REJECTED" || out.starts_with("RUNTIME_REJECTED"),
        "type wall did not hold: {out}");
}

/// Ported from `tests/cpython/type/std-libs/xml_sax_expatreader/ExpatParser__unparsed_entity_decl__name_as_str_wrong.py`.
#[test]
fn test_gen_type_std_libs_xml_sax_expatreader_ExpatParser__unparsed_entity_decl__name_as_str_wrong() {
    let out = run_type_wall_fixture(r###"# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "xml_sax_expatreader"
# dimension = "type"
# case = "ExpatParser__unparsed_entity_decl__name_as_str_wrong"
# subject = "xml.sax.expatreader.ExpatParser.unparsed_entity_decl(name: str)"
# kind = "semantic"
# xfail = ""
# mem_carveout = ""
# source = "vendor/typeshed/stdlib/xml/sax/expatreader.pyi"
# status = "filled"
# ///
# mamba-strict-type: TypeError
"""Type wall: xml.sax.expatreader.ExpatParser.unparsed_entity_decl(name: str); call it with the wrong type.

typeshed contract: name is str. mamba is force-typed, so a wrong-typed
argument MUST raise TypeError (CPython may accept or raise — mamba's to enforce)."""

from xml.sax.expatreader import ExpatParser
obj = object.__new__(ExpatParser)
try:
    obj.unparsed_entity_decl(12345, None, "", None, "")  # name: str <- wrong-typed
    print("no_typeerror:")  # CPython accepted the wrong-typed arg; mamba must raise
except TypeError as e:
    print("typeerror:", type(e).__name__)
except Exception as e:
    print("setup_or_other:", type(e).__name__)
"###);
    assert!(out == "STRICT_TYPE_REJECTED" || out.starts_with("RUNTIME_REJECTED"),
        "type wall did not hold: {out}");
}

/// Ported from `tests/cpython/type/std-libs/xml_sax_expatreader/create_parser__namespaceHandling_as_int_wrong.py`.
#[test]
fn test_gen_type_std_libs_xml_sax_expatreader_create_parser__namespaceHandling_as_int_wrong() {
    let out = run_type_wall_fixture(r###"# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "xml_sax_expatreader"
# dimension = "type"
# case = "create_parser__namespaceHandling_as_int_wrong"
# subject = "xml.sax.expatreader.create_parser(namespaceHandling: int)"
# kind = "semantic"
# xfail = ""
# mem_carveout = ""
# source = "vendor/typeshed/stdlib/xml/sax/expatreader.pyi"
# status = "filled"
# ///
# mamba-strict-type: TypeError
"""Type wall: xml.sax.expatreader.create_parser(namespaceHandling: int); call it with the wrong type.

typeshed contract: namespaceHandling is int. mamba is force-typed, so a wrong-typed
argument MUST raise TypeError (CPython may accept or raise — mamba's to enforce)."""

from xml.sax.expatreader import create_parser
try:
    create_parser("not_an_int")  # namespaceHandling: int <- wrong-typed
    print("no_typeerror:")  # CPython accepted the wrong-typed arg; mamba must raise
except TypeError as e:
    print("typeerror:", type(e).__name__)
except Exception as e:
    print("setup_or_other:", type(e).__name__)
"###);
    assert!(out == "STRICT_TYPE_REJECTED" || out.starts_with("RUNTIME_REJECTED"),
        "type wall did not hold: {out}");
}
