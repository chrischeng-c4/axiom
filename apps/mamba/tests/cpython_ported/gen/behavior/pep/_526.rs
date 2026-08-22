use super::super::super::super::harness::*;

/// Ported from `tests/cpython/behavior/pep/526/annotated_value_binds.py`.
#[test]
fn test_gen_behavior_pep_526_annotated_value_binds() {
    let out = jit_capture(r###"# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "pep"
# lib = "526"
# dimension = "behavior"
# case = "annotated_value_binds"
# subject = "__annotations__"
# kind = "semantic"
# xfail = "module __annotations__ is an undefined name on mamba. See project_mamba_pep_silent_divergences_2026_05_27."
# mem_carveout = ""
# source = ""
# status = "filled"
# ///
"""__annotations__: an annotated assignment with a value `x: int = 42` binds the name to that value (42) like a normal assignment"""

x: int = 42
assert x == 42, x
assert "x" in __annotations__, __annotations__
print("annotated_value_binds OK")
"###);
    assert_output(&out, r###"annotated_value_binds OK
"###);
}

/// Ported from `tests/cpython/behavior/pep/526/annotation_only_does_not_bind.py`.
#[test]
fn test_gen_behavior_pep_526_annotation_only_does_not_bind() {
    let out = jit_capture(r###"# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "pep"
# lib = "526"
# dimension = "behavior"
# case = "annotation_only_does_not_bind"
# subject = "__annotations__"
# kind = "semantic"
# xfail = "module __annotations__ is an undefined name on mamba. See project_mamba_pep_silent_divergences_2026_05_27."
# mem_carveout = ""
# source = ""
# status = "filled"
# ///
"""__annotations__: a bare annotation `y: int` (no value) does NOT bind the name at module scope (NameError on read) but DOES record 'y' in __annotations__"""

# A bare annotation records the name but never binds it.
y: int  # type-only annotation, no value
bound = True
try:
    y  # noqa: B018  -- read the unbound name
except NameError:
    bound = False
assert bound is False, "bare annotation must not bind the name"
assert "y" in __annotations__, __annotations__
print("annotation_only_does_not_bind OK")
"###);
    assert_output(&out, r###"annotation_only_does_not_bind OK
"###);
}

/// Ported from `tests/cpython/behavior/pep/526/class_annotation_records.py`.
#[test]
fn test_gen_behavior_pep_526_class_annotation_records() {
    let out = jit_capture(r###"# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "pep"
# lib = "526"
# dimension = "behavior"
# case = "class_annotation_records"
# subject = "__annotations__"
# kind = "semantic"
# xfail = "class __annotations__ is an undefined name on mamba. See project_mamba_pep_silent_divergences_2026_05_27."
# mem_carveout = ""
# source = ""
# status = "filled"
# ///
"""__annotations__: class-body annotations `a: int = 1; b: str = 'hi'` are recorded in the class __annotations__ mapping (keys {'a', 'b'})"""


class C:
    a: int = 1
    b: str = "hi"


assert sorted(C.__annotations__.keys()) == ["a", "b"], C.__annotations__
print("class_annotation_records OK")
"###);
    assert_output(&out, r###"class_annotation_records OK
"###);
}

/// Ported from `tests/cpython/behavior/pep/526/forward_ref_string_kept.py`.
#[test]
fn test_gen_behavior_pep_526_forward_ref_string_kept() {
    let out = jit_capture(r###"# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "pep"
# lib = "526"
# dimension = "behavior"
# case = "forward_ref_string_kept"
# subject = "__annotations__"
# kind = "semantic"
# xfail = ""
# mem_carveout = ""
# source = ""
# status = "filled"
# ///
"""__annotations__: a string forward-reference annotation `x: 'Undefined'` is stored verbatim as the string 'Undefined' in __annotations__ (never evaluated)"""


# The annotation names a type that does not exist; given as a string it is
# stored as-is and never resolved, so defining the function does not raise.
def lazy(x: "Undefined") -> "Undefined":  # type: ignore[name-defined]  # noqa: F821
    return x


assert lazy.__annotations__.get("x") == "Undefined", lazy.__annotations__
assert lazy.__annotations__.get("return") == "Undefined", lazy.__annotations__
print("forward_ref_string_kept OK")
"###);
    assert_output(&out, r###"forward_ref_string_kept OK
"###);
}

/// Ported from `tests/cpython/behavior/pep/526/function_annotation_records.py`.
#[test]
fn test_gen_behavior_pep_526_function_annotation_records() {
    let out = jit_capture(r###"# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "pep"
# lib = "526"
# dimension = "behavior"
# case = "function_annotation_records"
# subject = "__annotations__"
# kind = "semantic"
# xfail = "function __annotations__ returns None on mamba. See project_mamba_pep_silent_divergences_2026_05_27."
# mem_carveout = ""
# source = ""
# status = "filled"
# ///
"""__annotations__: function parameter and return annotations populate fn.__annotations__ with keys {'a', 'b', 'return'}"""


def fn(a: int, b: str) -> bool:
    return True


assert sorted(fn.__annotations__.keys()) == ["a", "b", "return"], fn.__annotations__
print("function_annotation_records OK")
"###);
    assert_output(&out, r###"function_annotation_records OK
"###);
}

/// Ported from `tests/cpython/behavior/pep/526/function_annotations_dict.py`.
#[test]
fn test_gen_behavior_pep_526_function_annotations_dict() {
    let out = jit_capture(r###"# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "pep"
# lib = "526"
# dimension = "behavior"
# case = "function_annotations_dict"
# subject = "__annotations__"
# kind = "semantic"
# xfail = "function __annotations__ returns None on mamba; subscripting it raises TypeError. See project_mamba_pep_silent_divergences_2026_05_27."
# mem_carveout = ""
# source = ""
# status = "filled"
# ///
"""__annotations__: fn.__annotations__ is a dict mapping each annotated name to its annotation object: {'x': int, 'y': int, 'return': int}"""


def annotated(x: int, y: int) -> int:
    return x + y


ann = annotated.__annotations__
assert type(ann).__name__ == "dict", type(ann).__name__
assert sorted(ann.keys()) == ["return", "x", "y"], sorted(ann.keys())
assert ann["x"] is int and ann["y"] is int and ann["return"] is int, ann
print("function_annotations_dict OK")
"###);
    assert_output(&out, r###"function_annotations_dict OK
"###);
}

/// Ported from `tests/cpython/behavior/pep/526/module_annotation_records.py`.
#[test]
fn test_gen_behavior_pep_526_module_annotation_records() {
    let out = jit_capture(r###"# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "pep"
# lib = "526"
# dimension = "behavior"
# case = "module_annotation_records"
# subject = "__annotations__"
# kind = "semantic"
# xfail = "module __annotations__ is an undefined name on mamba. See project_mamba_pep_silent_divergences_2026_05_27."
# mem_carveout = ""
# source = ""
# status = "filled"
# ///
"""__annotations__: an annotated module-scope assignment `x: int = 1` records 'x' in the module __annotations__ mapping"""

x: int = 1
assert "x" in __annotations__, __annotations__
print("module_annotation_records OK")
"###);
    assert_output(&out, r###"module_annotation_records OK
"###);
}
