use super::super::super::super::harness::*;

/// Ported from `tests/cpython/type/std-libs/xml_dom_NodeFilter/NodeFilter__acceptNode__node_as_Node_wrong.py`.
#[test]
fn test_gen_type_std_libs_xml_dom_NodeFilter_NodeFilter__acceptNode__node_as_Node_wrong() {
    let out = run_type_wall_fixture(r###"# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "xml_dom_NodeFilter"
# dimension = "type"
# case = "NodeFilter__acceptNode__node_as_Node_wrong"
# subject = "xml.dom.NodeFilter.NodeFilter.acceptNode(node: Node)"
# kind = "semantic"
# mem_carveout = ""
# source = "vendor/typeshed/stdlib/xml/dom/NodeFilter.pyi"
# status = "filled"
# ///
# mamba-strict-type: TypeError
"""Type wall: xml.dom.NodeFilter.NodeFilter.acceptNode(node: Node); call it with the wrong type.

typeshed contract: node is Node. mamba is force-typed, so a wrong-typed
argument MUST raise TypeError (CPython may accept or raise — mamba's to enforce)."""

class _W:
    pass


from xml.dom.NodeFilter import NodeFilter
obj = object.__new__(NodeFilter)
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
