#![cfg(test)]

use crate::parser;
use crate::source::span::FileId;
use crate::types::TypeChecker;

fn check(src: &str) -> Vec<String> {
    let module = parser::parse(src, FileId(0)).expect("parse failed");
    let mut checker = TypeChecker::new();
    let errors = checker.check_module(&module);
    errors.into_iter().map(|e| e.to_string()).collect()
}

#[allow(dead_code)]
fn check_strict(src: &str) -> Vec<String> {
    let module = parser::parse(src, FileId(0)).expect("parse failed");
    let mut checker = TypeChecker::new();
    checker.strict = true;
    let errors = checker.check_module(&module);
    errors.into_iter().map(|e| e.to_string()).collect()
}

#[allow(dead_code)]
fn check_warnings(src: &str) -> Vec<String> {
    let module = parser::parse(src, FileId(0)).expect("parse failed");
    let mut checker = TypeChecker::new();
    let _ = checker.check_module(&module);
    checker
        .diagnostics
        .iter()
        .map(|d| d.message.clone())
        .collect()
}

#[test]
fn test_valid_fibonacci() {
    let errors = check(
        "def fibonacci(n: int) -> int:\n\
         \x20   a: int = 0\n\
         \x20   b: int = 1\n\
         \x20   i: int = 0\n\
         \x20   while i < n:\n\
         \x20       temp: int = b\n\
         \x20       b = a + b\n\
         \x20       a = temp\n\
         \x20       i = i + 1\n\
         \x20   return a\n",
    );
    assert!(errors.is_empty(), "expected no errors, got: {errors:?}");
}

#[test]
fn test_type_mismatch_var_decl() {
    let errors = check("x: str = 42\n");
    assert!(!errors.is_empty());
    assert!(errors[0].contains("type mismatch"));
}

#[test]
fn test_return_type_mismatch() {
    // Use a genuinely incompatible return type (str). `bool` is a subtype
    // of `int` per CPython semantics (#1680), so `return True` from an
    // `int`-annotated function is correctly accepted now.
    let errors = check(
        "def bad() -> int:\n\
         \x20   return \"hi\"\n",
    );
    assert!(!errors.is_empty());
    assert!(errors[0].contains("return type mismatch"));
}

#[test]
fn test_undefined_variable() {
    let errors = check("x: int = y\n");
    assert!(!errors.is_empty());
    assert!(errors[0].contains("undefined name"));
}

#[test]
fn test_valid_arithmetic() {
    let errors = check(
        "def calc() -> int:\n\
         \x20   a: int = 1\n\
         \x20   b: int = 2\n\
         \x20   c: int = a + b * a\n\
         \x20   return c\n",
    );
    assert!(errors.is_empty(), "got: {errors:?}");
}

#[test]
fn test_int_float_mismatch() {
    let errors = check("x: int = 3.14\n");
    assert!(!errors.is_empty());
    assert!(errors[0].contains("type mismatch"));
}

#[test]
fn test_bool_condition_accepts_int() {
    // Python truthiness: `if x:` accepts any type (the runtime calls
    // __bool__/__len__). The type checker mirrors this policy
    // (check_stmt.rs:96) for Py3.12 compat — int conditions are valid.
    let errors = check(
        "x: int = 1\n\
         if x:\n\
         \x20   pass\n",
    );
    assert!(
        errors.is_empty(),
        "int condition should be accepted, got: {errors:?}"
    );
}

#[test]
fn test_function_call_arg_count_underflow_skipped() {
    // The arity check intentionally skips when positional < params.len()
    // (check_expr.rs — `might_have_defaults`) because the type system does
    // not yet plumb default info per parameter. `add(1)` and `add()` would
    // TypeError at runtime, but the static checker abstains here. The zero
    // case is required for #1600 — see also `test_zero_arg_call_to_default_param_fn`.
    let errors = check(
        "def add(a: int, b: int) -> int:\n\
         \x20   return a + b\n\
         add(1)\n",
    );
    assert!(errors.is_empty(),
        "underflow arity is intentionally skipped to avoid false positives on default params, got: {errors:?}");
    let errors = check(
        "def add(a: int, b: int) -> int:\n\
         \x20   return a + b\n\
         add()\n",
    );
    assert!(
        errors.is_empty(),
        "zero-arg underflow is also skipped (#1600), got: {errors:?}"
    );
}

#[test]
fn test_valid_boolean_ops() {
    let errors = check(
        "x: bool = True\n\
         y: bool = False\n\
         z: bool = x and y\n",
    );
    assert!(errors.is_empty(), "got: {errors:?}");
}

#[test]
fn test_comparison_returns_bool() {
    let errors = check(
        "a: int = 1\n\
         b: int = 2\n\
         c: bool = a < b\n",
    );
    assert!(errors.is_empty(), "got: {errors:?}");
}

#[test]
fn test_string_variable() {
    let errors = check("name: str = \"hello\"\n");
    assert!(errors.is_empty(), "got: {errors:?}");
}

#[test]
fn test_multiple_functions() {
    let errors = check(
        "def square(n: int) -> int:\n\
         \x20   return n * n\n\
         def main() -> int:\n\
         \x20   result: int = square(5)\n\
         \x20   return result\n",
    );
    assert!(errors.is_empty(), "got: {errors:?}");
}

// --- #240: Any type and unannotated inference ---

#[test]
fn test_any_type_annotation() {
    // Explicit Any annotation should be compatible with anything
    let errors = check(
        "x: Any = 42\n\
         y: Any = \"hello\"\n\
         z: int = x\n",
    );
    assert!(errors.is_empty(), "Any should be compatible: {errors:?}");
}

#[test]
fn test_any_unannotated_return() {
    // Missing return annotation defaults to Any (#240)
    let errors = check(
        "def greet(name: str):\n\
         \x20   return name\n\
         x: int = greet(\"hi\")\n",
    );
    assert!(
        errors.is_empty(),
        "unannotated return should be Any: {errors:?}"
    );
}

#[test]
fn test_any_compatible_both_directions() {
    let errors = check(
        "a: Any = 42\n\
         b: int = a\n\
         c: str = a\n\
         d: Any = b\n",
    );
    assert!(
        errors.is_empty(),
        "Any should be compatible both ways: {errors:?}"
    );
}

#[test]
fn test_any_in_binop() {
    let errors = check(
        "a: Any = 42\n\
         b: int = 10\n\
         c: Any = a + b\n",
    );
    assert!(
        errors.is_empty(),
        "Any in binop should propagate: {errors:?}"
    );
}

// --- #241: Type alias support ---

#[test]
fn test_type_alias_simple() {
    let errors = check(
        "type Num = int\n\
         x: Num = 42\n",
    );
    assert!(errors.is_empty(), "type alias should resolve: {errors:?}");
}

#[test]
fn test_type_alias_tuple() {
    let errors = check(
        "type Point = tuple[int, int]\n\
         p: Point = (1, 2)\n",
    );
    assert!(errors.is_empty(), "tuple alias should resolve: {errors:?}");
}

// --- #245: Builtin function stubs ---

#[test]
fn test_builtin_len() {
    let errors = check("x: int = len(\"hello\")\n");
    assert!(errors.is_empty(), "len should return int: {errors:?}");
}

#[test]
fn test_builtin_isinstance() {
    // `int` is a keyword, use a class name instead
    let errors = check(
        "class MyType:\n\
         \x20   pass\n\
         x: bool = isinstance(42, MyType)\n",
    );
    assert!(
        errors.is_empty(),
        "isinstance should return bool: {errors:?}"
    );
}

#[test]
fn test_builtin_abs() {
    let errors = check("x: int = abs(-5)\n");
    assert!(
        errors.is_empty(),
        "abs should work with Any param: {errors:?}"
    );
}

#[test]
fn test_builtin_print_accepts_any() {
    let errors = check("print(42)\nprint(\"hello\")\nprint(True)\n");
    assert!(errors.is_empty(), "print should accept Any: {errors:?}");
}

// --- #246: Class field resolution ---

#[test]
fn test_class_field_access() {
    let errors = check(
        "class Point:\n\
         \x20   x: int = 0\n\
         \x20   y: int = 0\n",
    );
    assert!(errors.is_empty(), "class def should type-check: {errors:?}");
}

// --- #248: Index/subscript type checking ---

#[test]
fn test_list_index_type() {
    let errors = check(
        "def get_first(items: list[int]) -> int:\n\
         \x20   return items[0]\n",
    );
    assert!(errors.is_empty(), "list[int][0] should be int: {errors:?}");
}

#[test]
fn test_dict_index_type() {
    let errors = check(
        "def get_val(d: dict[str, int]) -> int:\n\
         \x20   return d[\"key\"]\n",
    );
    assert!(
        errors.is_empty(),
        "dict[str,int][key] should be int: {errors:?}"
    );
}

#[test]
fn test_str_index_type() {
    let errors = check(
        "def first_char(s: str) -> str:\n\
         \x20   return s[0]\n",
    );
    assert!(errors.is_empty(), "str[0] should be str: {errors:?}");
}

#[test]
fn test_user_subscript_assignment_does_not_require_receiver_type() {
    let errors = check(
        "class Recorder:\n\
         \x20   def __setitem__(self, key, value):\n\
         \x20       pass\n\
         r = Recorder()\n\
         r[1:2] = 42\n",
    );
    assert!(
        errors.is_empty(),
        "user __setitem__ slice assignment should be runtime-dispatched: {errors:?}"
    );
}

#[test]
fn test_subscript_assignment_list_index_still_checks_element_type() {
    let errors = check("xs: list[int] = [1]\nxs[0] = \"bad\"\n");
    assert!(
        errors
            .iter()
            .any(|e| e.contains("expected `int`, got `str`")),
        "list element assignment mismatch should still be rejected: {errors:?}"
    );
}

#[test]
fn test_subscript_assignment_list_slice_checks_list_value_type() {
    let errors = check("xs: list[int] = [1]\nxs[0:1] = [\"bad\"]\n");
    assert!(
        errors
            .iter()
            .any(|e| e.contains("expected `list[int]`, got `list[str]`")),
        "list slice assignment mismatch should still be rejected: {errors:?}"
    );
}

// --- #249: Exception hierarchy ---

#[test]
fn test_exception_classes_exist() {
    // Exception classes should be in scope and usable
    let errors = check(
        "try:\n\
         \x20   pass\n\
         except ValueError as e:\n\
         \x20   pass\n",
    );
    assert!(
        errors.is_empty(),
        "ValueError should be in scope: {errors:?}"
    );
}

// --- #314: Generics and Protocols ---

#[test]
fn test_generic_function_type_params() {
    // Generic function with type params should type-check
    let errors = check(
        "def first[T](items: list[T]) -> T:\n\
         \x20   return items[0]\n",
    );
    assert!(
        errors.is_empty(),
        "generic function should type-check: {errors:?}"
    );
}

#[test]
fn test_generic_function_call_inference() {
    // Calling a generic function should infer type args
    let errors = check(
        "def identity[T](x: T) -> T:\n\
         \x20   return x\n\
         result: int = identity(42)\n",
    );
    assert!(
        errors.is_empty(),
        "generic call should infer T=int: {errors:?}"
    );
}

#[test]
fn test_generic_class_definition() {
    // Generic class with type params should type-check
    let errors = check(
        "class Box[T]:\n\
         \x20   pass\n",
    );
    assert!(
        errors.is_empty(),
        "generic class should type-check: {errors:?}"
    );
}

#[test]
fn test_generic_class_as_type() {
    // User-defined generic class should be resolvable as a type
    let errors = check(
        "class Container[T]:\n\
         \x20   pass\n\
         def use_container(c: Container[int]) -> None:\n\
         \x20   pass\n",
    );
    assert!(
        errors.is_empty(),
        "Container[int] should resolve: {errors:?}"
    );
}

#[test]
fn test_protocol_registration() {
    // Protocol class should type-check without errors
    let errors = check(
        "class Drawable(Protocol):\n\
         \x20   def draw(self) -> None:\n\
         \x20       pass\n",
    );
    assert!(
        errors.is_empty(),
        "Protocol definition should work: {errors:?}"
    );
}

#[test]
fn test_protocol_structural_matching() {
    // A class that implements protocol methods should be usable where protocol is expected
    let errors = check(
        "class Drawable(Protocol):\n\
         \x20   def draw(self) -> None:\n\
         \x20       pass\n\
         class Circle:\n\
         \x20   def draw(self) -> None:\n\
         \x20       pass\n\
         def render(obj: Drawable) -> None:\n\
         \x20   pass\n\
         render(Circle())\n",
    );
    assert!(
        errors.is_empty(),
        "Circle should satisfy Drawable protocol: {errors:?}"
    );
}

#[test]
fn test_generic_type_param_scoping() {
    // Type param T should not leak outside its function scope
    let errors = check(
        "def first[T](items: list[T]) -> T:\n\
         \x20   return items[0]\n\
         def second(x: int) -> int:\n\
         \x20   return x\n",
    );
    assert!(errors.is_empty(), "T should not leak: {errors:?}");
}

#[test]
fn test_generic_inference_conflict() {
    // Calling identity[T](x: T, y: T) with int and str should report conflict
    let errors = check(
        "def same[T](x: T, y: T) -> T:\n\
         \x20   return x\n\
         same(1, \"hello\")\n",
    );
    assert!(!errors.is_empty(), "conflicting T should error");
}

#[test]
fn test_generic_class_constraint_via_function() {
    // Generic function enforces type consistency on Box[T]
    let errors = check(
        "class Box[T]:\n\
         \x20   pass\n\
         def unbox[T](b: Box[T], default: T) -> T:\n\
         \x20   return default\n\
         unbox(Box(), 42)\n",
    );
    assert!(
        errors.is_empty(),
        "generic function with Box[T] should work: {errors:?}"
    );
}

#[test]
fn test_generic_class_rejects_wrong_type_arg() {
    // Box[int].value should be int; returning it as str must fail
    let errors = check(
        "class Box[T]:\n\
         \x20   value: T = None\n\
         def get_value(b: Box[int]) -> str:\n\
         \x20   return b.value\n",
    );
    assert!(
        !errors.is_empty(),
        "Box[int].value is int, should reject str return"
    );
}

// --- #827: Match/case type narrowing ---

#[test]
fn test_match_class_pattern_narrows_type() {
    // Inside `case Point():`, the subject should be narrowed to Point type
    // so accessing Point fields should not produce type errors
    let errors = check(
        "class Point:\n\
         \x20   x: int = 0\n\
         \x20   y: int = 0\n\
         def process(p: Point) -> int:\n\
         \x20   match p:\n\
         \x20       case Point():\n\
         \x20           return p.x\n\
         \x20   return 0\n",
    );
    assert!(
        errors.is_empty(),
        "class pattern body should type-check cleanly: {errors:?}"
    );
}

#[test]
fn test_match_guard_type_checks() {
    // Guard expression in match arm should be type-checked
    let errors = check(
        "x: int = 5\n\
         match x:\n\
         \x20   case n if n > 0:\n\
         \x20       y: int = n\n\
         \x20   case _:\n\
         \x20       y: int = 0\n",
    );
    assert!(
        errors.is_empty(),
        "match guard should type-check cleanly: {errors:?}"
    );
}

#[test]
fn test_match_class_capture_types() {
    let errors = check(
        "class Point:\n    x: int = 0\n    y: int = 0\n\
         p = Point()\n\
         match p:\n\
         \x20   case Point(x=a):\n\
         \x20       z: int = a\n", // a should be typed as int
    );
    assert!(
        errors.is_empty(),
        "class pattern capture should be typed as field type: {errors:?}"
    );
}

#[test]
fn test_match_class_positional_follows_match_args() {
    // __match_args__ = ("y", "x") means positional slot 0 = y (str), slot 1 = x (int).
    // Uses bare assignment form (no type annotation) to avoid the `tuple` builtin ambiguity.
    let errors = check(
        "class Point:\n    x: int = 0\n    y: str = \"\"\n    __match_args__ = (\"y\", \"x\")\n\
         p = Point()\n\
         match p:\n\
         \x20   case Point(a, b):\n\
         \x20       s: str = a\n\
         \x20       i: int = b\n",
    );
    assert!(
        errors.is_empty(),
        "__match_args__ reordering should type-check: {errors:?}"
    );
}

#[test]
fn test_match_sequence_capture_element_type() {
    // case [x]: on list[int] should type x as int
    let errors = check(
        "def f(xs: list[int]) -> int:\n\
         \x20   match xs:\n\
         \x20       case [x]:\n\
         \x20           y: int = x\n\
         \x20           return y\n\
         \x20   return 0\n",
    );
    assert!(
        errors.is_empty(),
        "sequence element capture should be int: {errors:?}"
    );
}

#[test]
fn test_match_tuple_sequence_capture() {
    // case (n, _): on (int, str) should type n as int (per-slot, not Union)
    let errors = check(
        "def f() -> int:\n\
         \x20   match (1, 2):\n\
         \x20       case (n, _):\n\
         \x20           return n + 1\n\
         \x20   return 0\n",
    );
    assert!(
        errors.is_empty(),
        "tuple capture slot should be int for arithmetic: {errors:?}"
    );
}

#[test]
fn test_match_bool_class_pattern_narrows_to_bool() {
    // case bool(b): should narrow b to bool, not int (#827 R4)
    let errors = check(
        "def f(x: int) -> bool:\n\
         \x20   match x:\n\
         \x20       case bool(b):\n\
         \x20           return b\n\
         \x20   return False\n",
    );
    assert!(
        errors.is_empty(),
        "bool class pattern should narrow capture to bool: {errors:?}"
    );
}

#[test]
fn test_match_explicit_empty_match_args_no_positional() {
    // class C with explicit __match_args__ = () should disallow positional patterns (#827 R5).
    // The type checker must treat empty __match_args__ as authoritative (no positional slots).
    // Here `case C(v):` with C.__match_args__ = () has no positional fields, so v gets any().
    let errors = check(
        "class C:\n\
         \x20   __match_args__ = ()\n\
         def f(c: C) -> int:\n\
         \x20   match c:\n\
         \x20       case C(v):\n\
         \x20           return 0\n\
         \x20   return 0\n",
    );
    // Should not produce a crash; the type checker is consistent (no panic).
    let _ = errors;
}

// --- string-ops: Str + Str type checking ---

#[test]
fn test_str_add_str_no_type_error() {
    // str + str must not emit "arithmetic requires numeric types"
    let errors = check(
        "a: str = \"hello\"\n\
         b: str = \" world\"\n\
         c: str = a + b\n",
    );
    assert!(
        errors.is_empty(),
        "str + str should typecheck without errors: {errors:?}"
    );
}

#[test]
fn test_str_concat_return_type() {
    // str + str result is assignable to str; function return type is accepted
    let errors = check(
        "def greet(first: str, last: str) -> str:\n\
         \x20   return first + last\n",
    );
    assert!(
        errors.is_empty(),
        "str + str return should be accepted as str: {errors:?}"
    );
}

#[test]
fn test_str_add_int_is_type_error() {
    // str + int must still be rejected (operand type mismatch)
    let errors = check(
        "a: str = \"x\"\n\
         b: int = 1\n\
         c: str = a + b\n",
    );
    assert!(!errors.is_empty(), "str + int should produce a type error");
}

// ── R9: Type checker — multi-argument stdlib forms ──

// R9.1: next(iterator, default) 2-argument form must be accepted
#[test]
fn test_next_two_arg_form_accepted() {
    let errors = check(
        "it = iter([])\n\
         result = next(it, 42)\n",
    );
    assert!(
        errors.is_empty(),
        "next(it, default) 2-arg form should be accepted: {errors:?}"
    );
}

// R9.1: next(iterator) 1-argument form must still be accepted
#[test]
fn test_next_one_arg_form_accepted() {
    let errors = check(
        "it = iter([])\n\
         result = next(it)\n",
    );
    assert!(
        errors.is_empty(),
        "next(it) 1-arg form should be accepted: {errors:?}"
    );
}

// #1574: unary + and - must accept bool (Python: +True == 1, -True == -1).
#[test]
fn test_unary_plus_minus_on_bool() {
    let errors = check(
        "x: bool = True\n\
         a = +x\n\
         b = -x\n",
    );
    assert!(
        errors.is_empty(),
        "unary +/- on bool should type-check (bool is int subtype): {errors:?}"
    );
}

// #1562: for-loop over a homogeneous tuple of strings must yield a Str
// element type, not Union[Str,Str,Str], otherwise Str+Str inside the body
// hits "arithmetic requires numeric types".
#[test]
fn test_for_over_homogeneous_str_tuple_concats() {
    let errors = check(
        "for sign in \"\", \"+\", \"-\":\n\
         \x20   ss = sign + sign\n",
    );
    assert!(
        errors.is_empty(),
        "for over homogeneous str-tuple should allow Str+Str in body: {errors:?}"
    );
}

// #1578: dotted-path generics (`typing.Iterable[int]`) and freeform
// string-literal annotations (`'This is a new annotation'`) must resolve
// to Any rather than emitting `unknown (generic) type` errors.
#[test]
fn test_dotted_generic_and_freeform_string_annotation() {
    let errors = check(
        "import typing\n\
         def f(a: 'This is a new annotation') -> int:\n\
         \x20   return 1\n\
         def g(x: typing.Iterable[int]) -> typing.Union[int, str]:\n\
         \x20   return 0\n",
    );
    assert!(errors.is_empty(),
        "freeform string-literal + dotted-path generic annotations should resolve to Any: {errors:?}");
}

// #1576: dotted-path annotations like `collections.abc.Mapping` must parse
// and resolve to Any (external/forward reference), not error.
#[test]
fn test_dotted_path_annotation_resolves_to_any() {
    let errors = check(
        "def f(arg: collections.abc.Mapping) -> int:\n\
         \x20   return 1\n\
         def g(arg: int) -> collections.abc.Mapping:\n\
         \x20   return arg\n",
    );
    assert!(
        errors.is_empty(),
        "dotted-path annotations should type-check as Any: {errors:?}"
    );
}

// R9.1: iter() is variadic (accepts 1 or 2 args)
#[test]
fn test_iter_two_arg_form_accepted() {
    let errors = check(
        "def sentinel() -> int:\n\
         \x20   return -1\n\
         it = iter(sentinel, -1)\n",
    );
    assert!(
        errors.is_empty(),
        "iter(callable, sentinel) 2-arg form should be accepted: {errors:?}"
    );
}

#[test]
fn test_stdlib_iter_wrong_bare_object_rejected() {
    let errors = check("class _W:\n    pass\niter(_W())\n");
    assert!(
        errors
            .iter()
            .any(|e| e.contains("does not satisfy parameter `object`")),
        "iter(_W()) should reject a bare object operand, got: {errors:?}"
    );

    let errors = check("class _W:\n    pass\niter(_W(), None)\n");
    assert!(
        errors
            .iter()
            .any(|e| e.contains("does not satisfy parameter `object`")),
        "iter(_W(), None) should reject a bare callable operand, got: {errors:?}"
    );

    let errors = check(
        "def sentinel() -> int:\n    return -1\niter(sentinel, -1)\niter([])\niter(\"abc\")\n",
    );
    assert!(
        errors.is_empty(),
        "valid iter forms must stay accepted, got: {errors:?}"
    );
}

#[test]
fn test_stdlib_list_dunder_contracts_rejected() {
    let errors = check(
        "obj = []\nobj.__add__(12345)\nobj.__ge__(12345)\nobj.__gt__(12345)\nobj.__le__(12345)\nobj.__lt__(12345)\n",
    );
    let list_value_errors = errors
        .iter()
        .filter(|e| e.contains("expected `list`, got `int`"))
        .count();
    assert_eq!(
        list_value_errors, 5,
        "list value dunders should reject concrete non-list operands, got: {errors:?}"
    );

    let errors = check(
        "class _W:\n    pass\nobj = []\nobj.__getitem__(_W())\nobj.__delitem__(_W())\nobj.__setitem__(_W(), None)\n",
    );
    let key_errors = errors
        .iter()
        .filter(|e| e.contains("does not satisfy parameter `key`"))
        .count();
    assert_eq!(
        key_errors, 3,
        "list key dunders should reject bare key operands, got: {errors:?}"
    );

    let errors = check(
        "obj = []\nobj.__add__([])\nobj.__ge__([])\nobj.__gt__([])\nobj.__le__([])\nobj.__lt__([])\nobj.__getitem__(0)\nobj.__getitem__(slice(0, 1))\nobj.__delitem__(0)\nobj.__setitem__(0, None)\nobj.__setitem__(slice(0, 1), [])\n",
    );
    assert!(
        errors.is_empty(),
        "valid list dunder forms must stay accepted, got: {errors:?}"
    );
}

#[test]
fn test_stdlib_tuple_dunder_contracts_rejected() {
    let errors = check(
        "obj = ()\nobj.__add__(12345)\nobj.__ge__(12345)\nobj.__gt__(12345)\nobj.__le__(12345)\nobj.__lt__(12345)\n",
    );
    let tuple_value_errors = errors
        .iter()
        .filter(|e| e.contains("expected `tuple`, got `int`"))
        .count();
    assert_eq!(
        tuple_value_errors, 5,
        "tuple value dunders should reject concrete non-tuple operands, got: {errors:?}"
    );

    let errors = check("class _W:\n    pass\nobj = ()\nobj.__getitem__(_W())\n");
    let key_errors = errors
        .iter()
        .filter(|e| e.contains("does not satisfy parameter `key`"))
        .count();
    assert_eq!(
        key_errors, 1,
        "tuple key dunder should reject a bare key operand, got: {errors:?}"
    );

    let errors = check(
        "obj = ()\nobj.__add__(())\nobj.__ge__(())\nobj.__gt__(())\nobj.__le__(())\nobj.__lt__(())\nobj.__getitem__(0)\nobj.__getitem__(slice(0, 1))\n",
    );
    assert!(
        errors.is_empty(),
        "valid tuple dunder forms must stay accepted, got: {errors:?}"
    );
}

#[test]
fn test_stdlib_type_and_zip_contracts_rejected() {
    let errors = check(
        "from builtins import type\nobj = object.__new__(type)\nobj.__new__(12345, None, None)\n",
    );
    assert!(
        errors
            .iter()
            .any(|e| e.contains("expected `str`, got `int`")),
        "type.__new__ should reject a non-str name, got: {errors:?}"
    );

    let errors = check(
        "class _W:\n    pass\nfrom builtins import type\nobj = object.__new__(type)\nobj.__subclasscheck__(_W())\n",
    );
    assert!(
        errors
            .iter()
            .any(|e| e.contains("does not satisfy parameter `subclass`")),
        "type.__subclasscheck__ should reject a bare instance subclass, got: {errors:?}"
    );

    let errors =
        check("class _W:\n    pass\nfrom builtins import zip\nobj = object.__new__(zip)\nobj.__new__(_W())\n");
    assert!(
        errors
            .iter()
            .any(|e| e.contains("does not satisfy parameter `iter1`")),
        "zip.__new__ should reject a bare non-iterable probe, got: {errors:?}"
    );

    let errors = check(
        "from builtins import type, zip\nobj = object.__new__(type)\nobj.__new__('X', (), {})\nobj.__subclasscheck__(type)\nz = object.__new__(zip)\nz.__new__([])\n",
    );
    assert!(
        errors.is_empty(),
        "valid type/zip strict contract forms must stay accepted, got: {errors:?}"
    );
}

#[test]
fn test_list_generic_method_contracts_rejected() {
    let errors = check(
        "class _W:\n    def __eq__(self, other):\n        return True\nobj: list[int] = [1]\nobj.append(_W())\nobj.count(_W())\nobj.index(_W())\nobj.remove(_W())\n",
    );
    let element_errors = errors
        .iter()
        .filter(|e| e.contains("expected `int`, got `_W`"))
        .count();
    assert_eq!(
        element_errors, 4,
        "list[T] methods should reject values outside the element type, got: {errors:?}"
    );

    let errors = check(
        "obj: list[int] = [1]\nobj.append(2)\nobj.count(1)\nobj.index(1)\nobj.index(1, 0)\nobj.index(1, 0, 1)\nobj.remove(1)\n",
    );
    assert!(
        errors.is_empty(),
        "valid list[T] method forms must stay accepted, got: {errors:?}"
    );
}

#[test]
fn test_set_generic_method_contracts_rejected() {
    let errors = check(
        "class _W:\n    def __hash__(self):\n        return hash(1)\n    def __eq__(self, other):\n        return True\nobj: set[int] = {1}\nobj.add(_W())\nobj.remove(_W())\n",
    );
    let element_errors = errors
        .iter()
        .filter(|e| e.contains("expected `int`, got `_W`"))
        .count();
    assert_eq!(
        element_errors, 2,
        "set[T] methods should reject values outside the element type, got: {errors:?}"
    );

    let errors = check("obj: set[int] = {1}\nobj.add(2)\nobj.remove(1)\n");
    assert!(
        errors.is_empty(),
        "valid set[T] method forms must stay accepted, got: {errors:?}"
    );
}

#[test]
fn test_stdlib_slice_new_contracts_rejected() {
    let errors = check(
        "from builtins import slice\nclass _W:\n    pass\nslice.__new__(slice, _W())\nslice.__new__(slice, _W(), None)\nslice.__new__(slice, None, _W())\n",
    );
    let typed_errors = errors
        .iter()
        .filter(|e| e.contains("does not satisfy parameter"))
        .count();
    assert_eq!(
        typed_errors, 3,
        "slice.__new__ should reject bare start/stop instances, got: {errors:?}"
    );

    let errors = check(
        "from builtins import slice\nclass C:\n    pass\nslice.__new__(slice, 3)\nslice.__new__(slice, 1, 3)\nslice.__new__(slice, None, None)\nslice.__new__(slice, C)\n",
    );
    assert!(
        errors.is_empty(),
        "valid slice.__new__ forms must stay accepted, got: {errors:?}"
    );
}

#[test]
fn test_stdlib_str_contracts_rejected() {
    let errors = check(
        "from builtins import str\nclass _W:\n    pass\nobj = str.__new__(str)\nobj.__add__(_W())\nobj.__add__(123)\nobj.__getitem__(_W())\nobj.__mod__(_W())\nobj.__mul__(_W())\nobj.__rmul__(_W())\nobj.center(_W())\nobj.endswith(_W())\nobj.expandtabs(_W())\nstr.__new__(str, _W(), \"utf-8\")\n",
    );
    let typed_errors = errors
        .iter()
        .filter(|e| e.contains("does not satisfy parameter"))
        .count();
    assert_eq!(
        typed_errors, 9,
        "str protocol/typed walls should reject bare instances, got: {errors:?}"
    );
    assert!(
        errors
            .iter()
            .any(|e| e.contains("expected `str`, got `int`")),
        "str.__add__(int) should be rejected as a scalar mismatch, got: {errors:?}"
    );

    let errors = check(
        "from builtins import str\nobj = str.__new__(str)\nobj.__add__(\"x\")\nobj.__getitem__(0)\nobj.__getitem__(slice(0, 1))\nobj.__mul__(2)\nobj.__rmul__(2)\nobj.center(3)\nobj.endswith(\"x\")\nobj.endswith((\"x\", \"y\"))\nobj.expandtabs(4)\nstr.__new__(str)\nstr.__new__(str, 123)\n",
    );
    assert!(
        errors.is_empty(),
        "valid str method forms must stay accepted, got: {errors:?}"
    );
}

#[test]
fn test_stdlib_str_text_method_contracts_rejected() {
    let errors = check(
        "from builtins import str\nclass _W:\n    pass\nobj = str.__new__(str)\nobj.ljust(_W())\nobj.rjust(_W())\nobj.zfill(_W())\nobj.lstrip(_W())\nobj.rstrip(_W())\nobj.strip(_W())\nobj.split(_W())\nobj.rsplit(_W())\nobj.startswith(_W())\nstr.maketrans(_W())\nobj.partition(123)\nobj.rpartition(123)\nobj.removeprefix(123)\nobj.removesuffix(123)\nobj.replace(123, \"\")\nobj.splitlines(123)\nstr.maketrans(123, \"\")\n",
    );
    let typed_errors = errors
        .iter()
        .filter(|e| e.contains("does not satisfy parameter"))
        .count();
    assert_eq!(
        typed_errors, 10,
        "str text-method protocol walls should reject bare instances, got: {errors:?}"
    );
    let scalar_errors = errors.iter().filter(|e| e.contains("expected `")).count();
    assert_eq!(
        scalar_errors, 7,
        "str text-method scalar walls should reject wrong scalars, got: {errors:?}"
    );

    let errors = check(
        "from builtins import str\nobj = str.__new__(str)\nobj.ljust(3)\nobj.rjust(3)\nobj.zfill(3)\nobj.lstrip(None)\nobj.lstrip(\"x\")\nobj.rstrip(None)\nobj.strip(\"x\")\nobj.partition(\"x\")\nobj.rpartition(\"x\")\nobj.removeprefix(\"x\")\nobj.removesuffix(\"x\")\nobj.replace(\"x\", \"y\")\nobj.split(None)\nobj.split(\"x\", 1)\nobj.rsplit(None)\nobj.splitlines(True)\nobj.startswith(\"x\")\nobj.startswith((\"x\", \"y\"))\nstr.maketrans(\"a\", \"b\")\nstr.maketrans({\"a\": \"b\"})\n",
    );
    assert!(
        errors.is_empty(),
        "valid str text method forms must stay accepted, got: {errors:?}"
    );
}

#[test]
fn test_stdlib_map_new_callable_rejected() {
    let errors =
        check("from builtins import map\nclass _W:\n    pass\nmap.__new__(map, _W(), None)\n");
    assert!(
        errors
            .iter()
            .any(|e| e.contains("does not satisfy parameter `func`")),
        "map.__new__(map, _W(), None) should reject a bare non-Callable func, got: {errors:?}"
    );

    let errors = check(
        "from builtins import map\ndef identity(x):\n    return x\nmap.__new__(map, identity, [1])\n",
    );
    assert!(
        errors.is_empty(),
        "valid map.__new__ callable form must stay accepted, got: {errors:?}"
    );
}

#[test]
fn test_stdlib_memoryview_method_contracts_rejected() {
    let errors = check(
        "from builtins import memoryview\nclass _W:\n    pass\nobj = memoryview(bytearray(b\"abc\"))\nobj.__exit__(_W(), None, None)\nobj.__getitem__(_W())\nobj.__setitem__(_W(), None)\nobj.tobytes(_W())\nobj.__release_buffer__(12345)\n",
    );
    let typed_errors = errors
        .iter()
        .filter(|e| e.contains("does not satisfy parameter"))
        .count();
    assert_eq!(
        typed_errors, 4,
        "memoryview typed params should reject bare user instances, got: {errors:?}"
    );
    assert!(
        errors
            .iter()
            .any(|e| e.contains("expected `memoryview`, got `int`")),
        "memoryview.__release_buffer__(int) should reject scalar buffers, got: {errors:?}"
    );

    let errors = check(
        "from builtins import memoryview\nobj = memoryview(bytearray(b\"abc\"))\nobj.__exit__(None, None, None)\nobj.__getitem__(0)\nobj.__getitem__(slice(0, 1))\nobj.__setitem__(0, 65)\nobj.tobytes()\nobj.tobytes(\"C\")\nbuf: Any = 12345\nobj.__release_buffer__(buf)\n",
    );
    assert!(
        errors.is_empty(),
        "valid and dynamic memoryview method forms must stay accepted, got: {errors:?}"
    );
}

#[test]
fn test_stdlib_range_method_contracts_rejected() {
    let errors = check(
        "from builtins import range\nclass _W:\n    pass\nobj = range(3)\nobj.__getitem__(_W())\nrange.__new__(range, _W())\nrange.__new__(range, _W(), 3)\nrange.__new__(range, 0, _W())\n",
    );
    let typed_errors = errors
        .iter()
        .filter(|e| e.contains("does not satisfy parameter"))
        .count();
    assert_eq!(
        typed_errors, 4,
        "range protocol params should reject bare user instances, got: {errors:?}"
    );

    let errors = check(
        "from builtins import range\nclass _Index:\n    def __index__(self) -> int:\n        return 1\nobj = range(3)\nobj.__getitem__(0)\nobj.__getitem__(slice(0, 1))\nobj.__getitem__(_Index())\nrange.__new__(range, 3)\nrange.__new__(range, 0, 3)\nrange.__new__(range, 0, 3, 1)\nvalue: Any = _Index()\nobj.__getitem__(value)\nrange.__new__(range, value)\n",
    );
    assert!(
        errors.is_empty(),
        "valid and dynamic range protocol forms must stay accepted, got: {errors:?}"
    );
}

#[test]
fn test_stdlib_property_descriptor_contracts_rejected() {
    let errors = check(
        "from builtins import property\nclass _W:\n    pass\ndef f(self=None):\n    return None\nobj = property(f)\nobj.__get__(_W(), None)\nobj.__get__(None, _W())\nobj.getter(_W())\nobj.setter(_W())\nobj.deleter(_W())\nproperty(_W())\n",
    );
    let typed_errors = errors
        .iter()
        .filter(|e| e.contains("does not satisfy parameter"))
        .count();
    assert_eq!(
        typed_errors, 6,
        "property descriptor protocol slots should reject bare user instances, got: {errors:?}"
    );

    let errors = check(
        "from builtins import property\nclass _Owner:\n    def marker(self):\n        return None\ndef f(self=None):\n    return None\ndef s(self, value):\n    pass\ndef d(self):\n    pass\nobj = property(f)\nobj.__get__(None, None)\nobj.__get__(None, _Owner)\nobj.getter(f)\nobj.setter(s)\nobj.deleter(d)\nproperty(f, s, d, \"doc\")\nvalue: Any = f\nobj.getter(value)\nproperty(value)\n",
    );
    assert!(
        errors.is_empty(),
        "valid and dynamic property descriptor forms must stay accepted, got: {errors:?}"
    );
}

#[test]
fn test_stdlib_object_subclasshook_rejects_instance_not_type() {
    let errors =
        check("from builtins import object\nclass _W:\n    pass\nobject.__subclasshook__(_W())\n");
    assert!(
        errors
            .iter()
            .any(|e| e.contains("does not satisfy parameter `subclass`")),
        "object.__subclasshook__(_W()) should reject a bare instance, got: {errors:?}"
    );

    let errors =
        check("from builtins import object\nclass _W:\n    pass\nobject.__subclasshook__(_W)\n");
    assert!(
        errors.is_empty(),
        "object.__subclasshook__(_W) must accept class objects, got: {errors:?}"
    );

    let errors = check("def f():\n    pass\nclass C:\n    pass\nf.__get__(None, C)\n");
    assert!(
        errors.is_empty(),
        "descriptor owner params must accept class objects, got: {errors:?}"
    );
}

#[test]
fn test_stdlib_reversed_new_protocol_sequence_rejected() {
    let errors = check(
        "from builtins import reversed\nclass _W:\n    pass\nreversed.__new__(reversed, _W())\n",
    );
    assert!(
        errors
            .iter()
            .any(|e| e.contains("does not satisfy parameter `sequence`")),
        "reversed.__new__(reversed, _W()) should reject a bare non-sequence instance, got: {errors:?}"
    );

    let errors = check(
        "from builtins import reversed\nclass SeqLike:\n    def __len__(self):\n        return 0\n    def __getitem__(self, index):\n        return index\nreversed.__new__(reversed, [1, 2])\nreversed.__new__(reversed, SeqLike())\n",
    );
    assert!(
        errors.is_empty(),
        "reversed.__new__ protocol wall must stay skip-safe for list and sequence-like operands, got: {errors:?}"
    );
}

// R9.3: getattr() with default (3-arg form) must be accepted
#[test]
fn test_getattr_three_arg_form_accepted() {
    let errors = check(
        "class Foo:\n\
         \x20   x: int = 1\n\
         obj = Foo()\n\
         val = getattr(obj, \"x\", 0)\n",
    );
    assert!(
        errors.is_empty(),
        "getattr(obj, name, default) 3-arg form should be accepted: {errors:?}"
    );
}

// R9: open() with mode and additional kwargs — variadic builtin
#[test]
fn test_open_variadic_form_accepted() {
    let errors = check("f = open(\"path.txt\", \"r\")\n");
    assert!(
        errors.is_empty(),
        "open(path, mode) 2-arg form should be accepted: {errors:?}"
    );
}

// #1586: heterogeneous-callable Union — for-target binding to a tuple of
// type constructors / fns must be callable across the loop body.
#[test]
fn test_union_of_callables_is_callable() {
    let errors = check(
        "for C in set, frozenset, str, list, tuple:\n\
         \x20   x = C('a')\n\
         \x20   y = C('b')\n",
    );
    assert!(
        errors.is_empty(),
        "Union of Fn/Class types must be callable, got: {errors:?}"
    );
}

// #1588: free names inside function bodies should defer to runtime (Any)
// rather than erroring at type-check time. Matches Python's lazy global
// lookup semantics. Module-level free names stay hard errors.
#[test]
fn test_free_name_in_fn_body_is_lazy() {
    let errors = check(
        "class C:\n\
         \x20   def m(self):\n\
         \x20       return undefined_name\n",
    );
    assert!(
        errors.is_empty(),
        "free name in method body should be deferred to runtime, got: {errors:?}"
    );
}

#[test]
fn test_free_name_at_module_level_still_errors() {
    let errors = check("print(undefined_name)\n");
    assert!(
        errors.iter().any(|e| e.contains("undefined name")),
        "module-level free name should still error, got: {errors:?}"
    );
}

#[test]
fn test_zero_arg_call_to_default_param_fn() {
    // `def f(x=1): return x; f()` — the existing heuristic skips arity for
    // partial-fill (1..N-1 args), and #1600 extends it to the zero-arg case.
    // Defaults aren't surfaced through `Ty::Fn`, so this is the only way to
    // accept all-defaults calls without breaking down the type structure.
    let errors = check("def f(x=1):\n    return x\n\np = f()\n");
    assert!(
        errors.is_empty(),
        "zero-arg call to default-param fn should type-check, got: {errors:?}"
    );
    let errors = check("def g(a=1, b=2, c=3):\n    return a + b + c\n\nq = g()\n");
    assert!(
        errors.is_empty(),
        "zero-arg call to all-default fn should type-check, got: {errors:?}"
    );
}

#[test]
fn test_property_zero_arg_is_callable() {
    // CPython: property(fget=None, fset=None, fdel=None, doc=None) — all
    // params optional. Mamba's stub must accept 0..=4 args, not require fget.
    let errors = check("p = property()\n");
    assert!(
        errors.is_empty(),
        "property() with no args should type-check (variadic stub), got: {errors:?}"
    );
    let errors = check("p = property(lambda self: 1)\n");
    assert!(
        errors.is_empty(),
        "property(fget) should still type-check, got: {errors:?}"
    );
}

// ── ① Type-wall PoC: stdlib argument enforcement ─────────────────────────────

#[test]
fn test_stdlib_module_fn_wrong_scalar_rejected() {
    // os.strerror(code: int) called with a str literal — must be rejected.
    let errors = check("from os import strerror\nstrerror(\"x\")\n");
    assert!(
        errors.iter().any(|e| e.contains("argument type mismatch")),
        "strerror(str) should be rejected, got: {errors:?}"
    );
    // os.getenv(key: str) called with an int literal — must be rejected.
    let errors = check("from os import getenv\ngetenv(123)\n");
    assert!(
        errors.iter().any(|e| e.contains("argument type mismatch")),
        "getenv(int) should be rejected, got: {errors:?}"
    );
    // multiprocessing.reduction.duplicate(handle: int) with str — rejected.
    let errors = check("from multiprocessing.reduction import duplicate\nduplicate(\"x\")\n");
    assert!(
        errors.iter().any(|e| e.contains("argument type mismatch")),
        "duplicate(str) should be rejected, got: {errors:?}"
    );
}

#[test]
fn test_stdlib_module_fn_correct_scalar_clean() {
    // Correct calls must NOT be rejected (the ② behavior oracle).
    let errors = check("from os import strerror\nstrerror(2)\n");
    assert!(
        errors.is_empty(),
        "strerror(int) must be clean, got: {errors:?}"
    );
    let errors = check("from os import getenv\ngetenv(\"PATH\")\n");
    assert!(
        errors.is_empty(),
        "getenv(str) must be clean, got: {errors:?}"
    );
    // Bool->int and int->float coercions must be allowed.
    let errors = check("from os import strerror\nstrerror(True)\n");
    assert!(
        errors.is_empty(),
        "strerror(bool) must be clean, got: {errors:?}"
    );
}

#[test]
fn test_stdlib_module_fn_via_module_attr() {
    // `import os; os.strerror("x")` — attr path through module binding.
    let errors = check("import os\nos.strerror(\"x\")\n");
    assert!(
        errors.iter().any(|e| e.contains("argument type mismatch")),
        "os.strerror(str) attr-path should be rejected, got: {errors:?}"
    );
    let errors = check("import os\nos.strerror(2)\n");
    assert!(
        errors.is_empty(),
        "os.strerror(int) attr-path must be clean, got: {errors:?}"
    );
}

#[test]
fn test_stdlib_method_wrong_scalar_rejected() {
    // HTMLParser.handle_entityref(name: str) via object.__new__ instance.
    let errors = check(
        "from html.parser import HTMLParser\n\
         obj = object.__new__(HTMLParser)\n\
         obj.handle_entityref(12345)\n",
    );
    assert!(
        errors.iter().any(|e| e.contains("argument type mismatch")),
        "handle_entityref(int) should be rejected, got: {errors:?}"
    );
    // Correct call is clean.
    let errors = check(
        "from html.parser import HTMLParser\n\
         obj = object.__new__(HTMLParser)\n\
         obj.handle_entityref(\"amp\")\n",
    );
    assert!(
        errors.is_empty(),
        "handle_entityref(str) must be clean, got: {errors:?}"
    );
}

#[test]
fn test_stdlib_constructor_wrong_scalar_rejected() {
    let errors = check("from builtins import SyntaxError\nSyntaxError(12345, None)\n");
    assert!(
        errors.iter().any(|e| e.contains("argument type mismatch")),
        "SyntaxError(non_str_msg, details) should be rejected by the strict type wall, got: {errors:?}"
    );
    let errors = check("from builtins import SyntaxError\nSyntaxError(\"bad\", None)\n");
    assert!(
        errors.is_empty(),
        "SyntaxError(str_msg, details) must be clean at type-check time, got: {errors:?}"
    );
}

#[test]
fn test_stdlib_exception_group_typed_method_rejects_bare_instance() {
    let errors = check(
        "from builtins import ExceptionGroup\nclass _W:\n    pass\nobj = ExceptionGroup(\"msg\", [ValueError(\"x\")])\nobj.split(_W())\n",
    );
    assert!(
        errors
            .iter()
            .any(|e| e.contains("does not satisfy parameter `matcher_value`")),
        "ExceptionGroup.split(_W()) should reject a bare instance, got: {errors:?}"
    );

    let errors = check(
        "from builtins import BaseExceptionGroup\nclass _W:\n    pass\nobj = BaseExceptionGroup(\"msg\", [ValueError(\"x\")])\nobj.derive(_W())\n",
    );
    assert!(
        errors
            .iter()
            .any(|e| e.contains("does not satisfy parameter `excs`")),
        "BaseExceptionGroup.derive(_W()) should reject a bare instance, got: {errors:?}"
    );

    let errors = check(
        "from builtins import ExceptionGroup\nobj = ExceptionGroup(\"msg\", [ValueError(\"x\")])\ndef matcher(exc):\n    return True\nobj.split(matcher)\n",
    );
    assert!(
        errors.is_empty(),
        "ExceptionGroup.split(callable) must remain clean, got: {errors:?}"
    );
}

#[test]
fn test_direct_builtin_typed_argument_rejected_unless_shadowed() {
    let errors = check("class _W:\n    pass\naiter(_W())\n");
    assert!(
        errors
            .iter()
            .any(|e| e.contains("does not satisfy parameter `async_iterable`")),
        "direct builtin aiter(_W()) should reject a bare instance, got: {errors:?}"
    );

    let errors = check("class _W:\n    pass\nanext(_W(), None)\n");
    assert!(
        errors
            .iter()
            .any(|e| e.contains("does not satisfy parameter `i`")),
        "direct builtin anext(_W(), None) should reject a bare instance, got: {errors:?}"
    );

    let errors = check("class _W:\n    pass\ndef aiter(value):\n    return value\naiter(_W())\n");
    assert!(
        errors.is_empty(),
        "user-shadowed aiter must not use the stdlib signature, got: {errors:?}"
    );
}

#[test]
fn test_stdlib_bool_bitwise_wrong_scalar_rejected() {
    let errors = check("from builtins import bool\nobj = bool()\nobj.__and__(\"bad\")\n");
    assert!(
        errors
            .iter()
            .any(|e| e.contains("expected `int`, got `str`")),
        "bool.__and__(str) should be rejected, got: {errors:?}"
    );

    let errors =
        check("from builtins import bool\nobj = bool()\nobj.__and__(True)\nobj.__or__(1)\n");
    assert!(
        errors.is_empty(),
        "bool bitwise dunders must allow bool/int operands, got: {errors:?}"
    );
}

#[test]
fn test_stdlib_set_operator_bare_instance_rejected() {
    let errors =
        check("from builtins import set\nclass _W:\n    pass\nobj = set()\nobj.__and__(_W())\nobj.__ge__(_W())\nobj.__gt__(_W())\nobj.__iand__(_W())\nobj.__ior__(_W())\nobj.__isub__(_W())\nobj.__ixor__(_W())\nobj.__le__(_W())\nobj.__lt__(_W())\nobj.__or__(_W())\nobj.__sub__(_W())\nobj.__xor__(_W())\n");
    assert!(
        errors
            .iter()
            .filter(|e| e.contains("does not satisfy parameter `value`"))
            .count()
            >= 12,
        "set operators should reject bare non-AbstractSet operands, got: {errors:?}"
    );

    let errors = check(
        "from builtins import set\nclass SetLike:\n    def __contains__(self, item):\n        return False\nobj = set()\nobj.__and__(set())\nobj.__ior__(SetLike())\n",
    );
    assert!(
        errors.is_empty(),
        "set operator wall must stay skip-safe for modeled/dynamic set-like operands, got: {errors:?}"
    );
}

#[test]
fn test_stdlib_frozenset_operator_bare_instance_rejected() {
    let errors =
        check("from builtins import frozenset\nclass _W:\n    pass\nobj = frozenset()\nobj.__and__(_W())\nobj.__ge__(_W())\nobj.__gt__(_W())\nobj.__le__(_W())\nobj.__lt__(_W())\nobj.__or__(_W())\nobj.__sub__(_W())\nobj.__xor__(_W())\n");
    assert!(
        errors
            .iter()
            .filter(|e| e.contains("does not satisfy parameter `value`"))
            .count()
            >= 8,
        "frozenset operators should reject bare non-AbstractSet operands, got: {errors:?}"
    );

    let errors = check(
        "from builtins import frozenset\nclass SetLike:\n    def __contains__(self, item):\n        return False\nobj = frozenset()\nobj.__and__(frozenset())\nobj.__ge__(SetLike())\n",
    );
    assert!(
        errors.is_empty(),
        "frozenset operator wall must stay skip-safe for modeled/dynamic set-like operands, got: {errors:?}"
    );
}

#[test]
fn test_stdlib_frozenset_new_iterable_rejected() {
    let errors = check(
        "from builtins import frozenset\nclass _W:\n    pass\nfrozenset.__new__(frozenset, _W())\n",
    );
    assert!(
        errors
            .iter()
            .any(|e| e.contains("does not satisfy parameter `iterable`")),
        "frozenset.__new__(frozenset, _W()) should reject a bare non-Iterable operand, got: {errors:?}"
    );
}

#[test]
fn test_stdlib_bytes_bytearray_wall_rejects_impossible_scalars() {
    let errors = check("from builtins import bytes\nobj = bytes()\nobj.__gt__(123)\n");
    assert!(
        errors
            .iter()
            .any(|e| e.contains("expected `bytes`, got `int`")),
        "bytes.__gt__(int) should be rejected, got: {errors:?}"
    );

    let errors = check("from builtins import bytes\nobj = bytes()\nobj.__gt__(b'ok')\n");
    assert!(
        errors.is_empty(),
        "bytes literal arguments infer to Any today and must stay skip-safe, got: {errors:?}"
    );

    let errors =
        check("from builtins import bytearray\nobj = bytearray()\nobj.splitlines(\"not_bool\")\n");
    assert!(
        errors
            .iter()
            .any(|e| e.contains("expected `bool`, got `str`")),
        "bytearray.splitlines(str) should be rejected, got: {errors:?}"
    );
}

#[test]
fn test_stdlib_bytes_bytearray_constructor_overload_walls() {
    let errors = check("from builtins import bytes\nclass _W:\n    pass\nbytes(_W())\n");
    assert!(
        errors
            .iter()
            .any(|e| e.contains("does not satisfy parameter `source`")),
        "bytes(_W()) should reject a bare source instance, got: {errors:?}"
    );

    let errors = check("from builtins import bytes\nbytes(12345, \"\")\n");
    assert!(
        errors
            .iter()
            .any(|e| e.contains("expected `str` source when `encoding` is provided")),
        "bytes(int, encoding) should reject the dependent overload mismatch, got: {errors:?}"
    );

    let errors = check("from builtins import bytearray\nbytearray(12345, \"\")\n");
    assert!(
        errors
            .iter()
            .any(|e| e.contains("expected `str` source when `encoding` is provided")),
        "bytearray(int, encoding) should reject the dependent overload mismatch, got: {errors:?}"
    );

    let errors = check(
        "from builtins import bytes, bytearray\nbytes(\"ok\", \"utf-8\")\nbytearray(\"ok\", \"utf-8\")\nbytes(3)\nbytearray(3)\n",
    );
    assert!(
        errors.is_empty(),
        "valid string+encoding and size constructors must stay clean, got: {errors:?}"
    );
}

#[test]
fn test_stdlib_bytearray_release_buffer_rejects_scalar() {
    let errors =
        check("from builtins import bytearray\nobj = bytearray()\nobj.__release_buffer__(12345)\n");
    assert!(
        errors
            .iter()
            .any(|e| e.contains("expected `memoryview`, got `int`")),
        "bytearray.__release_buffer__(int) should be rejected, got: {errors:?}"
    );

    let errors =
        check("from builtins import bytearray\nobj = bytearray()\nbuf: Any = 12345\nobj.__release_buffer__(buf)\n");
    assert!(
        errors.is_empty(),
        "dynamic memoryview-like values must stay skip-safe, got: {errors:?}"
    );
}

#[test]
fn test_stdlib_complex_constructor_and_dunder_walls() {
    let errors = check("from builtins import complex\nclass _W:\n    pass\ncomplex(_W())\n");
    assert!(
        errors
            .iter()
            .any(|e| e.contains("does not satisfy parameter `real`")),
        "complex(_W()) should reject a bare real argument, got: {errors:?}"
    );

    let errors = check("from builtins import complex\nobj = complex()\nobj.__add__(\"bad\")\n");
    assert!(
        errors
            .iter()
            .any(|e| e.contains("expected `complex`, got `str`")),
        "complex.__add__(str) should be rejected, got: {errors:?}"
    );

    let errors = check("from builtins import complex\nobj = complex()\nobj.__pow__(\"bad\")\n");
    assert!(
        errors
            .iter()
            .any(|e| e.contains("expected `complex`, got `str`")),
        "complex.__pow__(str) should be rejected, got: {errors:?}"
    );

    let errors = check(
        "from builtins import complex\nobj = complex()\nobj.__add__(1)\nobj.__mul__(1.5)\nobj.__truediv__(True)\ncomplex(1)\ncomplex(1.5)\ncomplex(\"1\")\n",
    );
    assert!(
        errors.is_empty(),
        "complex numeric/string constructor and numeric dunder uses must stay clean, got: {errors:?}"
    );

    let errors = check(
        "from builtins import complex\nobj = complex()\nvalue: Any = \"bad\"\nobj.__add__(value)\n",
    );
    assert!(
        errors.is_empty(),
        "dynamic complex-like values must stay skip-safe, got: {errors:?}"
    );
}

#[test]
fn test_stdlib_float_pow_round_walls() {
    let errors = check("from builtins import float\nobj = float()\nobj.__pow__(\"bad\")\n");
    assert!(
        errors
            .iter()
            .any(|e| e.contains("expected `float`, got `str`")),
        "float.__pow__(str) should be rejected, got: {errors:?}"
    );

    let errors = check("from builtins import float\nobj = float()\nobj.__rpow__(\"bad\")\n");
    assert!(
        errors
            .iter()
            .any(|e| e.contains("expected `float`, got `str`")),
        "float.__rpow__(str) should be rejected, got: {errors:?}"
    );

    let errors = check(
        "from builtins import float\nclass _W:\n    pass\nobj = float()\nobj.__round__(_W())\n",
    );
    assert!(
        errors
            .iter()
            .any(|e| e.contains("does not satisfy parameter `ndigits`")),
        "float.__round__(_W()) should reject a bare SupportsIndex miss, got: {errors:?}"
    );

    let errors = check(
        "from builtins import float\nobj = float()\nobj.__pow__(1)\nobj.__pow__(1.5)\nobj.__pow__(True)\nobj.__rpow__(1)\nobj.__rpow__(1.5)\nobj.__round__(1)\nobj.__round__(True)\n",
    );
    assert!(
        errors.is_empty(),
        "float numeric dunder uses must stay clean, got: {errors:?}"
    );

    let errors = check(
        "from builtins import float\nobj = float()\nvalue: Any = \"bad\"\nobj.__pow__(value)\nobj.__round__(value)\n",
    );
    assert!(
        errors.is_empty(),
        "dynamic float dunder values must stay skip-safe, got: {errors:?}"
    );
}

#[test]
fn test_dict_receiver_generic_key_methods() {
    let errors = check("class _W:\n    pass\nobj: dict[str, int] = {}\nobj.__getitem__(_W())\n");
    assert!(
        errors
            .iter()
            .any(|e| e.contains("expected `str`, got `_W`")),
        "dict[str, int].__getitem__(_W()) should reject the key type, got: {errors:?}"
    );

    let errors = check("class _W:\n    pass\nobj: dict[str, int] = {}\nobj.__setitem__(_W(), 1)\n");
    assert!(
        errors
            .iter()
            .any(|e| e.contains("expected `str`, got `_W`")),
        "dict[str, int].__setitem__(_W(), 1) should reject the key type, got: {errors:?}"
    );

    let errors = check(
        "class _W:\n    pass\nobj: dict[str, int] = {}\nobj.__delitem__(_W())\nobj.get(_W())\nobj.pop(_W(), None)\n",
    );
    assert!(
        errors
            .iter()
            .filter(|e| e.contains("expected `str`, got `_W`"))
            .count()
            >= 3,
        "dict key methods should reject wrong typed keys, got: {errors:?}"
    );

    let errors =
        check("class _K:\n    pass\nobj = {_K(): 1}\nobj.__getitem__(_K())\nobj.get(_K())\n");
    assert!(
        errors.is_empty(),
        "dicts keyed by a user class must stay valid when the receiver key type matches, got: {errors:?}"
    );
}

#[test]
fn test_dict_operator_negative_mapping_walls() {
    let errors = check("obj: dict[str, int] = {}\nobj.__or__(12345)\nobj.__ror__(\"bad\")\n");
    assert!(
        errors
            .iter()
            .filter(|e| e.contains("expected `mapping`"))
            .count()
            >= 2,
        "dict union operators should reject concrete scalar operands, got: {errors:?}"
    );

    let errors = check("class _W:\n    pass\nobj: dict[str, int] = {}\nobj.__ior__(_W())\n");
    assert!(
        errors
            .iter()
            .any(|e| e.contains("expected `mapping`, got `_W`")),
        "dict.__ior__(_W()) should reject a bare non-mapping operand, got: {errors:?}"
    );

    let errors = check(
        "class MappingLike:\n    def keys(self):\n        return []\n    def __getitem__(self, key):\n        return 1\nobj: dict[str, int] = {}\nobj.__or__({\"a\": 1})\nobj.__or__(MappingLike())\nobj.__ior__([(\"b\", 2)])\nvalue: Any = 1\nobj.__ror__(value)\n",
    );
    assert!(
        errors.is_empty(),
        "dict operator wall must stay skip-safe for mapping-like, iterable-pair, and dynamic operands, got: {errors:?}"
    );
}

#[test]
fn test_stdlib_classmethod_wrong_bare_instance_rejected() {
    let errors = check(
        "from builtins import classmethod\nclass _W:\n    pass\nobj = classmethod(lambda cls: None)\nobj.__get__(_W())\n",
    );
    assert!(
        errors
            .iter()
            .any(|e| e.contains("does not satisfy parameter `instance`")),
        "classmethod.__get__(_W()) should be rejected, got: {errors:?}"
    );

    let errors =
        check("from builtins import classmethod\nclass _W:\n    pass\nclassmethod(_W())\n");
    assert!(
        errors
            .iter()
            .any(|e| e.contains("does not satisfy parameter `f`")),
        "classmethod(_W()) should be rejected, got: {errors:?}"
    );

    let errors = check(
        "from builtins import classmethod\nobj = classmethod(lambda cls: None)\nobj.__get__(None)\n",
    );
    assert!(
        errors.is_empty(),
        "classmethod callable/None descriptor use must stay skip-safe, got: {errors:?}"
    );
}

#[test]
fn test_stdlib_staticmethod_wrong_bare_instance_rejected() {
    let errors = check(
        "from builtins import staticmethod\nclass _W:\n    pass\nobj = staticmethod(lambda: None)\nobj.__get__(_W())\n",
    );
    assert!(
        errors
            .iter()
            .any(|e| e.contains("does not satisfy parameter `instance`")),
        "staticmethod.__get__(_W()) should be rejected, got: {errors:?}"
    );

    let errors =
        check("from builtins import staticmethod\nclass _W:\n    pass\nstaticmethod(_W())\n");
    assert!(
        errors
            .iter()
            .any(|e| e.contains("does not satisfy parameter `f`")),
        "staticmethod(_W()) should be rejected, got: {errors:?}"
    );

    let errors = check(
        "from builtins import staticmethod\nobj = staticmethod(lambda: None)\nobj.__get__(None)\n",
    );
    assert!(
        errors.is_empty(),
        "staticmethod callable/None descriptor use must stay skip-safe, got: {errors:?}"
    );
}

#[test]
fn test_stdlib_function_get_owner_rejected() {
    let errors = check("class _W:\n    pass\ndef f():\n    pass\nf.__get__(None, _W())\n");
    assert!(
        errors
            .iter()
            .any(|e| e.contains("does not satisfy parameter `owner`")),
        "function.__get__(None, _W()) should reject a bare owner operand, got: {errors:?}"
    );

    let errors = check("def f():\n    pass\nf.__get__(None, None)\n");
    assert!(
        errors.is_empty(),
        "function.__get__(None, None) must stay skip-safe for runtime validation, got: {errors:?}"
    );
}

#[test]
fn test_stdlib_int_new_x_rejected() {
    let errors = check("from builtins import int\nclass _W:\n    pass\nint.__new__(int, _W())\n");
    assert!(
        errors
            .iter()
            .any(|e| e.contains("does not satisfy parameter `x`")),
        "int.__new__(int, _W()) should reject a bare x operand, got: {errors:?}"
    );

    let errors =
        check("from builtins import int\nclass _W:\n    pass\nint.__new__(int, _W(), None)\n");
    assert!(
        errors
            .iter()
            .any(|e| e.contains("does not satisfy parameter `x`")),
        "int.__new__(int, _W(), None) should reject a bare x operand before runtime base validation, got: {errors:?}"
    );

    let errors = check("from builtins import int\nint.__new__(int)\nint.__new__(int, \"123\")\n");
    assert!(
        errors.is_empty(),
        "valid int.__new__ class-call forms must stay skip-safe, got: {errors:?}"
    );
}

#[test]
fn test_stdlib_int_pow_value_rejected() {
    let errors = check("from builtins import int\nobj = int()\nobj.__pow__(\"bad\")\n");
    assert!(
        errors
            .iter()
            .any(|e| e.contains("expected `int`, got `str`")),
        "int.__pow__(str) should reject a non-int value operand, got: {errors:?}"
    );

    let errors = check(
        "from builtins import int\nclass _W:\n    pass\nobj = int()\nobj.__pow__(_W(), None)\n",
    );
    assert!(
        errors
            .iter()
            .any(|e| e.contains("does not satisfy parameter `value`")),
        "int.__pow__(_W(), None) should reject a bare value operand, got: {errors:?}"
    );

    let errors =
        check("from builtins import int\nobj = int()\nobj.__pow__(2)\nobj.__pow__(2, None)\n");
    assert!(
        errors.is_empty(),
        "valid int.__pow__ forms must stay skip-safe, got: {errors:?}"
    );
}

#[test]
fn test_stdlib_filter_wrong_bare_function_rejected() {
    let errors = check("from builtins import filter\nclass _W:\n    pass\nfilter(_W(), [])\n");
    assert!(
        errors
            .iter()
            .any(|e| e.contains("does not satisfy parameter `function`")),
        "filter(_W(), []) should reject a bare non-callable instance, got: {errors:?}"
    );

    let errors =
        check("from builtins import filter\ndef pred(value):\n    return True\nfilter(pred, [])\n");
    assert!(
        errors.is_empty(),
        "filter(callable, iterable) must stay clean, got: {errors:?}"
    );

    let errors = check("from builtins import filter\nfilter(None, [])\n");
    assert!(
        errors.is_empty(),
        "filter(None, iterable) must stay clean, got: {errors:?}"
    );
}

#[test]
fn test_stdlib_isinstance_classinfo_rejected() {
    let errors = check("class _W:\n    pass\nisinstance(None, _W())\n");
    assert!(
        errors
            .iter()
            .any(|e| e.contains("does not satisfy parameter `class_or_tuple`")),
        "isinstance(None, _W()) should reject a bare classinfo operand, got: {errors:?}"
    );

    let errors = check("class _W:\n    pass\nisinstance(None, (int, _W()))\n");
    assert!(
        errors
            .iter()
            .any(|e| e.contains("does not satisfy parameter `class_or_tuple`")),
        "isinstance(None, (int, _W())) should reject a bare classinfo tuple element, got: {errors:?}"
    );

    let errors = check(
        "class MyType:\n    pass\nisinstance(None, MyType)\nisinstance(None, (MyType, int))\n",
    );
    assert!(
        errors.is_empty(),
        "valid class and tuple classinfo operands must stay accepted, got: {errors:?}"
    );
}

#[test]
fn test_stdlib_unenforceable_never_rejected() {
    // base64.b64encode(s: ReadableBuffer) -> Unknown: NOT enforceable. Even a
    // blatantly wrong int must NOT be rejected.
    let errors = check("from base64 import b64encode\nb64encode(123)\n");
    assert!(
        errors.is_empty(),
        "b64encode(int) must NOT be rejected (ReadableBuffer->Unknown), got: {errors:?}"
    );
    // math.factorial(x: SupportsIndex) -> Unknown: NOT enforceable.
    let errors = check("from math import factorial\nfactorial(3.0)\n");
    assert!(
        errors.is_empty(),
        "factorial(float) must NOT be rejected (SupportsIndex->Unknown), got: {errors:?}"
    );
}

#[test]
fn test_stdlib_skip_when_arg_not_concrete_scalar() {
    // Argument is a variable of unknown type -> skip (Any actual).
    let errors = check("from os import strerror\ndef f(v):\n    return strerror(v)\n");
    assert!(
        errors.is_empty(),
        "strerror(unknown-var) must be skipped, got: {errors:?}"
    );
    // Star-arg present -> stop enforcement.
    let errors = check("from os import strerror\nargs = [\"x\"]\nstrerror(*args)\n");
    assert!(
        errors.is_empty(),
        "strerror(*args) must be skipped, got: {errors:?}"
    );
}

#[test]
fn test_stdlib_non_stdlib_call_untouched() {
    // A user fn that happens to share a stdlib name is not in import_origins,
    // so the hook never touches it.
    let errors = check("def strerror(x):\n    return x\nstrerror(\"x\")\n");
    assert!(
        errors.is_empty(),
        "user strerror must be untouched, got: {errors:?}"
    );
}
