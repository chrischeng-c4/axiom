use super::super::super::super::harness::*;

/// Ported from `tests/cpython/type/std-libs/xml_etree_ElementPath/get_parent_map__context_as__SelectorContext_wrong.py`.
#[test]
fn test_gen_type_std_libs_xml_etree_ElementPath_get_parent_map__context_as__SelectorContext_wrong() {
    let out = run_type_wall_fixture(r###"# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "xml_etree_ElementPath"
# dimension = "type"
# case = "get_parent_map__context_as__SelectorContext_wrong"
# subject = "xml.etree.ElementPath.get_parent_map(context: _SelectorContext)"
# kind = "semantic"
# xfail = ""
# mem_carveout = ""
# source = "vendor/typeshed/stdlib/xml/etree/ElementPath.pyi"
# status = "filled"
# ///
# mamba-strict-type: TypeError
"""Type wall: xml.etree.ElementPath.get_parent_map(context: _SelectorContext); call it with the wrong type.

typeshed contract: context is _SelectorContext. mamba is force-typed, so a wrong-typed
argument MUST raise TypeError (CPython may accept or raise — mamba's to enforce)."""

class _W:
    pass


from xml.etree.ElementPath import get_parent_map
try:
    get_parent_map(_W())  # context: _SelectorContext <- wrong-typed
    print("no_typeerror:")  # CPython accepted the wrong-typed arg; mamba must raise
except TypeError as e:
    print("typeerror:", type(e).__name__)
except Exception as e:
    print("setup_or_other:", type(e).__name__)
"###);
    assert!(out == "STRICT_TYPE_REJECTED" || out.starts_with("RUNTIME_REJECTED"),
        "type wall did not hold: {out}");
}

/// Ported from `tests/cpython/type/std-libs/xml_etree_ElementPath/prepare_child__next_as__Next_wrong.py`.
#[test]
fn test_gen_type_std_libs_xml_etree_ElementPath_prepare_child__next_as__Next_wrong() {
    let out = run_type_wall_fixture(r###"# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "xml_etree_ElementPath"
# dimension = "type"
# case = "prepare_child__next_as__Next_wrong"
# subject = "xml.etree.ElementPath.prepare_child(next: _Next)"
# kind = "semantic"
# xfail = ""
# mem_carveout = ""
# source = "vendor/typeshed/stdlib/xml/etree/ElementPath.pyi"
# status = "filled"
# ///
# mamba-strict-type: TypeError
"""Type wall: xml.etree.ElementPath.prepare_child(next: _Next); call it with the wrong type.

typeshed contract: next is _Next. mamba is force-typed, so a wrong-typed
argument MUST raise TypeError (CPython may accept or raise — mamba's to enforce)."""

class _W:
    pass


from xml.etree.ElementPath import prepare_child
try:
    prepare_child(_W(), None)  # next: _Next <- wrong-typed
    print("no_typeerror:")  # CPython accepted the wrong-typed arg; mamba must raise
except TypeError as e:
    print("typeerror:", type(e).__name__)
except Exception as e:
    print("setup_or_other:", type(e).__name__)
"###);
    assert!(out == "STRICT_TYPE_REJECTED" || out.starts_with("RUNTIME_REJECTED"),
        "type wall did not hold: {out}");
}

/// Ported from `tests/cpython/type/std-libs/xml_etree_ElementPath/prepare_descendant__next_as__Next_wrong.py`.
#[test]
fn test_gen_type_std_libs_xml_etree_ElementPath_prepare_descendant__next_as__Next_wrong() {
    let out = run_type_wall_fixture(r###"# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "xml_etree_ElementPath"
# dimension = "type"
# case = "prepare_descendant__next_as__Next_wrong"
# subject = "xml.etree.ElementPath.prepare_descendant(next: _Next)"
# kind = "semantic"
# xfail = ""
# mem_carveout = ""
# source = "vendor/typeshed/stdlib/xml/etree/ElementPath.pyi"
# status = "filled"
# ///
# mamba-strict-type: TypeError
"""Type wall: xml.etree.ElementPath.prepare_descendant(next: _Next); call it with the wrong type.

typeshed contract: next is _Next. mamba is force-typed, so a wrong-typed
argument MUST raise TypeError (CPython may accept or raise — mamba's to enforce)."""

class _W:
    pass


from xml.etree.ElementPath import prepare_descendant
try:
    prepare_descendant(_W(), None)  # next: _Next <- wrong-typed
    print("no_typeerror:")  # CPython accepted the wrong-typed arg; mamba must raise
except TypeError as e:
    print("typeerror:", type(e).__name__)
except Exception as e:
    print("setup_or_other:", type(e).__name__)
"###);
    assert!(out == "STRICT_TYPE_REJECTED" || out.starts_with("RUNTIME_REJECTED"),
        "type wall did not hold: {out}");
}

/// Ported from `tests/cpython/type/std-libs/xml_etree_ElementPath/prepare_parent__next_as__Next_wrong.py`.
#[test]
fn test_gen_type_std_libs_xml_etree_ElementPath_prepare_parent__next_as__Next_wrong() {
    let out = run_type_wall_fixture(r###"# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "xml_etree_ElementPath"
# dimension = "type"
# case = "prepare_parent__next_as__Next_wrong"
# subject = "xml.etree.ElementPath.prepare_parent(next: _Next)"
# kind = "semantic"
# xfail = ""
# mem_carveout = ""
# source = "vendor/typeshed/stdlib/xml/etree/ElementPath.pyi"
# status = "filled"
# ///
# mamba-strict-type: TypeError
"""Type wall: xml.etree.ElementPath.prepare_parent(next: _Next); call it with the wrong type.

typeshed contract: next is _Next. mamba is force-typed, so a wrong-typed
argument MUST raise TypeError (CPython may accept or raise — mamba's to enforce)."""

class _W:
    pass


from xml.etree.ElementPath import prepare_parent
try:
    prepare_parent(_W(), None)  # next: _Next <- wrong-typed
    print("no_typeerror:")  # CPython accepted the wrong-typed arg; mamba must raise
except TypeError as e:
    print("typeerror:", type(e).__name__)
except Exception as e:
    print("setup_or_other:", type(e).__name__)
"###);
    assert!(out == "STRICT_TYPE_REJECTED" || out.starts_with("RUNTIME_REJECTED"),
        "type wall did not hold: {out}");
}

/// Ported from `tests/cpython/type/std-libs/xml_etree_ElementPath/prepare_predicate__next_as__Next_wrong.py`.
#[test]
fn test_gen_type_std_libs_xml_etree_ElementPath_prepare_predicate__next_as__Next_wrong() {
    let out = run_type_wall_fixture(r###"# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "xml_etree_ElementPath"
# dimension = "type"
# case = "prepare_predicate__next_as__Next_wrong"
# subject = "xml.etree.ElementPath.prepare_predicate(next: _Next)"
# kind = "semantic"
# xfail = ""
# mem_carveout = ""
# source = "vendor/typeshed/stdlib/xml/etree/ElementPath.pyi"
# status = "filled"
# ///
# mamba-strict-type: TypeError
"""Type wall: xml.etree.ElementPath.prepare_predicate(next: _Next); call it with the wrong type.

typeshed contract: next is _Next. mamba is force-typed, so a wrong-typed
argument MUST raise TypeError (CPython may accept or raise — mamba's to enforce)."""

class _W:
    pass


from xml.etree.ElementPath import prepare_predicate
try:
    prepare_predicate(_W(), None)  # next: _Next <- wrong-typed
    print("no_typeerror:")  # CPython accepted the wrong-typed arg; mamba must raise
except TypeError as e:
    print("typeerror:", type(e).__name__)
except Exception as e:
    print("setup_or_other:", type(e).__name__)
"###);
    assert!(out == "STRICT_TYPE_REJECTED" || out.starts_with("RUNTIME_REJECTED"),
        "type wall did not hold: {out}");
}

/// Ported from `tests/cpython/type/std-libs/xml_etree_ElementPath/prepare_self__next_as__Next_wrong.py`.
#[test]
fn test_gen_type_std_libs_xml_etree_ElementPath_prepare_self__next_as__Next_wrong() {
    let out = run_type_wall_fixture(r###"# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "xml_etree_ElementPath"
# dimension = "type"
# case = "prepare_self__next_as__Next_wrong"
# subject = "xml.etree.ElementPath.prepare_self(next: _Next)"
# kind = "semantic"
# xfail = ""
# mem_carveout = ""
# source = "vendor/typeshed/stdlib/xml/etree/ElementPath.pyi"
# status = "filled"
# ///
# mamba-strict-type: TypeError
"""Type wall: xml.etree.ElementPath.prepare_self(next: _Next); call it with the wrong type.

typeshed contract: next is _Next. mamba is force-typed, so a wrong-typed
argument MUST raise TypeError (CPython may accept or raise — mamba's to enforce)."""

class _W:
    pass


from xml.etree.ElementPath import prepare_self
try:
    prepare_self(_W(), None)  # next: _Next <- wrong-typed
    print("no_typeerror:")  # CPython accepted the wrong-typed arg; mamba must raise
except TypeError as e:
    print("typeerror:", type(e).__name__)
except Exception as e:
    print("setup_or_other:", type(e).__name__)
"###);
    assert!(out == "STRICT_TYPE_REJECTED" || out.starts_with("RUNTIME_REJECTED"),
        "type wall did not hold: {out}");
}

/// Ported from `tests/cpython/type/std-libs/xml_etree_ElementPath/prepare_star__next_as__Next_wrong.py`.
#[test]
fn test_gen_type_std_libs_xml_etree_ElementPath_prepare_star__next_as__Next_wrong() {
    let out = run_type_wall_fixture(r###"# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "xml_etree_ElementPath"
# dimension = "type"
# case = "prepare_star__next_as__Next_wrong"
# subject = "xml.etree.ElementPath.prepare_star(next: _Next)"
# kind = "semantic"
# xfail = ""
# mem_carveout = ""
# source = "vendor/typeshed/stdlib/xml/etree/ElementPath.pyi"
# status = "filled"
# ///
# mamba-strict-type: TypeError
"""Type wall: xml.etree.ElementPath.prepare_star(next: _Next); call it with the wrong type.

typeshed contract: next is _Next. mamba is force-typed, so a wrong-typed
argument MUST raise TypeError (CPython may accept or raise — mamba's to enforce)."""

class _W:
    pass


from xml.etree.ElementPath import prepare_star
try:
    prepare_star(_W(), None)  # next: _Next <- wrong-typed
    print("no_typeerror:")  # CPython accepted the wrong-typed arg; mamba must raise
except TypeError as e:
    print("typeerror:", type(e).__name__)
except Exception as e:
    print("setup_or_other:", type(e).__name__)
"###);
    assert!(out == "STRICT_TYPE_REJECTED" || out.starts_with("RUNTIME_REJECTED"),
        "type wall did not hold: {out}");
}

/// Ported from `tests/cpython/type/std-libs/xml_etree_ElementPath/xpath_tokenizer__pattern_as_str_wrong.py`.
#[test]
fn test_gen_type_std_libs_xml_etree_ElementPath_xpath_tokenizer__pattern_as_str_wrong() {
    let out = run_type_wall_fixture(r###"# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "xml_etree_ElementPath"
# dimension = "type"
# case = "xpath_tokenizer__pattern_as_str_wrong"
# subject = "xml.etree.ElementPath.xpath_tokenizer(pattern: str)"
# kind = "semantic"
# xfail = ""
# mem_carveout = ""
# source = "vendor/typeshed/stdlib/xml/etree/ElementPath.pyi"
# status = "filled"
# ///
# mamba-strict-type: TypeError
"""Type wall: xml.etree.ElementPath.xpath_tokenizer(pattern: str); call it with the wrong type.

typeshed contract: pattern is str. mamba is force-typed, so a wrong-typed
argument MUST raise TypeError (CPython may accept or raise — mamba's to enforce)."""

from xml.etree.ElementPath import xpath_tokenizer
try:
    xpath_tokenizer(12345)  # pattern: str <- wrong-typed
    print("no_typeerror:")  # CPython accepted the wrong-typed arg; mamba must raise
except TypeError as e:
    print("typeerror:", type(e).__name__)
except Exception as e:
    print("setup_or_other:", type(e).__name__)
"###);
    assert!(out == "STRICT_TYPE_REJECTED" || out.starts_with("RUNTIME_REJECTED"),
        "type wall did not hold: {out}");
}
