use super::super::super::super::harness::*;

/// Ported from `tests/cpython/behavior/pep/variable_annotations/annotated_with_value_is_class_attr.py`.
#[test]
fn test_gen_behavior_pep_variable_annotations_annotated_with_value_is_class_attr() {
    let out = jit_capture(r###"# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "pep"
# lib = "variable_annotations"
# dimension = "behavior"
# case = "annotated_with_value_is_class_attr"
# subject = "__annotations__"
# kind = "semantic"
# xfail = "class __annotations__ / annotation-only attribute machinery diverges on mamba. See project_mamba_pep_silent_divergences_2026_05_27."
# mem_carveout = ""
# source = ""
# status = "filled"
# ///
"""__annotations__: an annotated class attribute WITH a value (`y: str = 'hi'`) becomes a real class attribute, while a bare annotation (`x: int`) does NOT"""


class A:
    x: int  # bare annotation: documented, not bound
    y: str = "hi"  # annotation + value: a real class attribute


assert not hasattr(A, "x"), "bare annotation must NOT create a class attribute"
assert hasattr(A, "y"), "annotation with value IS a class attribute"
assert A.y == "hi", A.y
print("annotated_with_value_is_class_attr OK")
"###);
    assert_output(&out, r###"annotated_with_value_is_class_attr OK
"###);
}

/// Ported from `tests/cpython/behavior/pep/variable_annotations/class_annotation_value_is_type.py`.
#[test]
fn test_gen_behavior_pep_variable_annotations_class_annotation_value_is_type() {
    let out = jit_capture(r###"# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "pep"
# lib = "variable_annotations"
# dimension = "behavior"
# case = "class_annotation_value_is_type"
# subject = "__annotations__"
# kind = "semantic"
# xfail = "class __annotations__ is an undefined name on mamba. See project_mamba_pep_silent_divergences_2026_05_27."
# mem_carveout = ""
# source = ""
# status = "filled"
# ///
"""__annotations__: the recorded class annotation value is the annotation object itself: C.__annotations__['host'] is str"""


class Config:
    host: str = "localhost"
    port: int = 8080


assert Config.__annotations__["host"] is str, Config.__annotations__["host"]
assert Config.__annotations__["port"] is int, Config.__annotations__["port"]
print("class_annotation_value_is_type OK")
"###);
    assert_output(&out, r###"class_annotation_value_is_type OK
"###);
}

/// Ported from `tests/cpython/behavior/pep/variable_annotations/class_annotations_record_keys.py`.
#[test]
fn test_gen_behavior_pep_variable_annotations_class_annotations_record_keys() {
    let out = jit_capture(r###"# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "pep"
# lib = "variable_annotations"
# dimension = "behavior"
# case = "class_annotations_record_keys"
# subject = "__annotations__"
# kind = "semantic"
# xfail = "class __annotations__ is an undefined name on mamba. See project_mamba_pep_silent_divergences_2026_05_27."
# mem_carveout = ""
# source = ""
# status = "filled"
# ///
"""__annotations__: class-body annotations `host: str; port: int; debug: bool` are recorded in the class __annotations__ mapping (keys {'host', 'port', 'debug'})"""


class Config:
    host: str = "localhost"
    port: int = 8080
    debug: bool = False


assert sorted(Config.__annotations__.keys()) == ["debug", "host", "port"], Config.__annotations__
print("class_annotations_record_keys OK")
"###);
    assert_output(&out, r###"class_annotations_record_keys OK
"###);
}

/// Ported from `tests/cpython/behavior/pep/variable_annotations/classvar_in_annotations.py`.
#[test]
fn test_gen_behavior_pep_variable_annotations_classvar_in_annotations() {
    let out = jit_capture(r###"# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "pep"
# lib = "variable_annotations"
# dimension = "behavior"
# case = "classvar_in_annotations"
# subject = "typing.ClassVar"
# kind = "semantic"
# xfail = "class __annotations__ is an undefined name on mamba; ClassVar declaration machinery diverges. See project_mamba_pep_silent_divergences_2026_05_27."
# mem_carveout = ""
# source = ""
# status = "filled"
# ///
"""typing.ClassVar: a `count: ClassVar[int] = 0` declaration is recorded in __annotations__ and the ClassVar attribute is shared (mutating it via the class is visible to all instances)"""
from typing import ClassVar


class Counter:
    count: ClassVar[int] = 0

    def __init__(self):
        Counter.count += 1


assert "count" in Counter.__annotations__, Counter.__annotations__
Counter.count = 0
c1 = Counter()
c2 = Counter()
# The ClassVar is class-shared state: both instances bumped the one counter.
assert Counter.count == 2, Counter.count
assert c1.count == 2 and c2.count == 2, (c1.count, c2.count)
print("classvar_in_annotations OK")
"###);
    assert_output(&out, r###"classvar_in_annotations OK
"###);
}

/// Ported from `tests/cpython/behavior/pep/variable_annotations/final_not_enforced_at_runtime.py`.
#[test]
fn test_gen_behavior_pep_variable_annotations_final_not_enforced_at_runtime() {
    let out = jit_capture(r###"# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "pep"
# lib = "variable_annotations"
# dimension = "behavior"
# case = "final_not_enforced_at_runtime"
# subject = "__annotations__"
# kind = "semantic"
# xfail = "mamba is force-typed; reassigning a Final-annotated name may be rejected, and the underlying annotation machinery diverges. See project_mamba_pep_silent_divergences_2026_05_27."
# mem_carveout = ""
# source = ""
# status = "filled"
# ///
"""__annotations__: a `Final` annotation is a type-checker hint only; at runtime `MAX: Final = 100` can be reassigned to 200 with no error"""
from typing import Final

# Final is enforced by type checkers, never at runtime.
MAX: Final = 100
assert MAX == 100, MAX
MAX = 200  # type: ignore[misc]
assert MAX == 200, "Final not enforced at runtime"
print("final_not_enforced_at_runtime OK")
"###);
    assert_output(&out, r###"final_not_enforced_at_runtime OK
"###);
}

/// Ported from `tests/cpython/behavior/pep/variable_annotations/get_type_hints_resolves_class.py`.
#[test]
fn test_gen_behavior_pep_variable_annotations_get_type_hints_resolves_class() {
    let out = jit_capture(r###"# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "pep"
# lib = "variable_annotations"
# dimension = "behavior"
# case = "get_type_hints_resolves_class"
# subject = "typing.get_type_hints"
# kind = "semantic"
# xfail = "get_type_hints reads __annotations__, which is an undefined name on mamba. See project_mamba_pep_silent_divergences_2026_05_27."
# mem_carveout = ""
# source = ""
# status = "filled"
# ///
"""typing.get_type_hints: get_type_hints(C) resolves a class's declared annotations to their runtime type objects: {'val': int}"""
from typing import get_type_hints


class C:
    val: int = 5


hints = get_type_hints(C)
assert "val" in hints, hints
assert hints["val"] is int, hints["val"]
print("get_type_hints_resolves_class OK")
"###);
    assert_output(&out, r###"get_type_hints_resolves_class OK
"###);
}

/// Ported from `tests/cpython/behavior/pep/variable_annotations/instance_annotation_in_init_binds.py`.
#[test]
fn test_gen_behavior_pep_variable_annotations_instance_annotation_in_init_binds() {
    let out = jit_capture(r###"# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "pep"
# lib = "variable_annotations"
# dimension = "behavior"
# case = "instance_annotation_in_init_binds"
# subject = "__annotations__"
# kind = "semantic"
# xfail = "class __annotations__ is an undefined name on mamba. See project_mamba_pep_silent_divergences_2026_05_27."
# mem_carveout = ""
# source = ""
# status = "filled"
# ///
"""__annotations__: class-body instance annotations (`x: float; y: float`) are recorded in the class __annotations__ while the values are bound per-instance in __init__"""


class Point:
    x: float
    y: float

    def __init__(self, x: float, y: float):
        self.x = x
        self.y = y


assert sorted(Point.__annotations__.keys()) == ["x", "y"], Point.__annotations__
# The annotations do not bind values; __init__ binds them per instance.
p = Point(1.0, 2.0)
assert p.x == 1.0 and p.y == 2.0, (p.x, p.y)
print("instance_annotation_in_init_binds OK")
"###);
    assert_output(&out, r###"instance_annotation_in_init_binds OK
"###);
}

/// Ported from `tests/cpython/behavior/pep/variable_annotations/module_annotation_records.py`.
#[test]
fn test_gen_behavior_pep_variable_annotations_module_annotation_records() {
    let out = jit_capture(r###"# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "pep"
# lib = "variable_annotations"
# dimension = "behavior"
# case = "module_annotation_records"
# subject = "__annotations__"
# kind = "semantic"
# xfail = "module __annotations__ is an undefined name on mamba. See project_mamba_pep_silent_divergences_2026_05_27."
# mem_carveout = ""
# source = ""
# status = "filled"
# ///
"""__annotations__: an annotated module-scope assignment `x: int = 42` records 'x' in the module __annotations__ mapping AND binds x to 42"""

x: int = 42
assert x == 42, x
assert "x" in __annotations__, __annotations__
print("module_annotation_records OK")
"###);
    assert_output(&out, r###"module_annotation_records OK
"###);
}

/// Ported from `tests/cpython/behavior/pep/variable_annotations/module_annotations_is_dict.py`.
#[test]
fn test_gen_behavior_pep_variable_annotations_module_annotations_is_dict() {
    let out = jit_capture(r###"# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "pep"
# lib = "variable_annotations"
# dimension = "behavior"
# case = "module_annotations_is_dict"
# subject = "__annotations__"
# kind = "semantic"
# xfail = "module __annotations__ is an undefined name on mamba. See project_mamba_pep_silent_divergences_2026_05_27."
# mem_carveout = ""
# source = ""
# status = "filled"
# ///
"""__annotations__: module-level __annotations__ is a plain dict after a `score: int = 0` annotation"""

score: int = 0
assert isinstance(__annotations__, dict), type(__annotations__)
assert "score" in __annotations__, __annotations__
print("module_annotations_is_dict OK")
"###);
    assert_output(&out, r###"module_annotations_is_dict OK
"###);
}
