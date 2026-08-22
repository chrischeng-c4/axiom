use super::super::super::super::harness::*;

/// Ported from `tests/cpython/type/std-libs/xml_etree_ElementInclude/default_loader__href_as_FileDescriptorOrPath_wrong.py`.
#[test]
fn test_gen_type_std_libs_xml_etree_ElementInclude_default_loader__href_as_FileDescriptorOrPath_wrong() {
    let out = run_type_wall_fixture(r###"# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "xml_etree_ElementInclude"
# dimension = "type"
# case = "default_loader__href_as_FileDescriptorOrPath_wrong"
# subject = "xml.etree.ElementInclude.default_loader(href: FileDescriptorOrPath)"
# kind = "semantic"
# mem_carveout = ""
# source = "vendor/typeshed/stdlib/xml/etree/ElementInclude.pyi"
# status = "filled"
# ///
# mamba-strict-type: TypeError
"""Type wall: xml.etree.ElementInclude.default_loader(href: FileDescriptorOrPath); call it with the wrong type.

typeshed contract: href is FileDescriptorOrPath. mamba is force-typed, so a wrong-typed
argument MUST raise TypeError (CPython may accept or raise — mamba's to enforce)."""

class _W:
    pass


from xml.etree.ElementInclude import default_loader
try:
    default_loader(_W(), None)  # href: FileDescriptorOrPath <- wrong-typed
    print("no_typeerror:")  # CPython accepted the wrong-typed arg; mamba must raise
except TypeError as e:
    print("typeerror:", type(e).__name__)
except Exception as e:
    print("setup_or_other:", type(e).__name__)
"###);
    assert!(out == "STRICT_TYPE_REJECTED" || out.starts_with("RUNTIME_REJECTED"),
        "type wall did not hold: {out}");
}

/// Ported from `tests/cpython/type/std-libs/xml_etree_ElementInclude/include__elem_as_Element_wrong.py`.
#[test]
fn test_gen_type_std_libs_xml_etree_ElementInclude_include__elem_as_Element_wrong() {
    let out = run_type_wall_fixture(r###"# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "xml_etree_ElementInclude"
# dimension = "type"
# case = "include__elem_as_Element_wrong"
# subject = "xml.etree.ElementInclude.include(elem: Element)"
# kind = "semantic"
# xfail = ""
# mem_carveout = ""
# source = "vendor/typeshed/stdlib/xml/etree/ElementInclude.pyi"
# status = "filled"
# ///
# mamba-strict-type: TypeError
"""Type wall: xml.etree.ElementInclude.include(elem: Element); call it with the wrong type.

typeshed contract: elem is Element. mamba is force-typed, so a wrong-typed
argument MUST raise TypeError (CPython may accept or raise — mamba's to enforce)."""

class _W:
    pass


from xml.etree.ElementInclude import include
try:
    include(_W())  # elem: Element <- wrong-typed
    print("no_typeerror:")  # CPython accepted the wrong-typed arg; mamba must raise
except TypeError as e:
    print("typeerror:", type(e).__name__)
except Exception as e:
    print("setup_or_other:", type(e).__name__)
"###);
    assert!(out == "STRICT_TYPE_REJECTED" || out.starts_with("RUNTIME_REJECTED"),
        "type wall did not hold: {out}");
}
