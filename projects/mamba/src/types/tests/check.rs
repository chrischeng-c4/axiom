#![cfg(test)]

use crate::parser;
use crate::source::span::FileId;
use crate::types::ty::{ClassRole, Ty, TypeParamDefault, TypeVarKind};
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

fn check_runtime(src: &str) -> Vec<String> {
    let module = parser::parse(src, FileId(0)).expect("parse failed");
    let mut checker = TypeChecker::new();
    checker.allow_runtime_unresolved_names = true;
    let errors = checker.check_module(&module);
    errors.into_iter().map(|e| e.to_string()).collect()
}

fn check_desugared(src: &str) -> Vec<String> {
    let mut module = parser::parse(src, FileId(0)).expect("parse failed");
    crate::lower::pep695::desugar_module(&mut module);
    let mut checker = TypeChecker::new();
    let errors = checker.check_module(&module);
    errors.into_iter().map(|e| e.to_string()).collect()
}

fn has_parameter_error(errors: &[String], parameter: &str) -> bool {
    let marker = format!("parameter `{parameter}`");
    errors.iter().any(|error| error.contains(&marker))
}

fn parameter_error_count(errors: &[String], parameter: &str) -> usize {
    let marker = format!("parameter `{parameter}`");
    errors
        .iter()
        .filter(|error| error.contains(&marker))
        .count()
}

fn bare_instance_parameter_error_count(errors: &[String]) -> usize {
    errors
        .iter()
        .filter(|error| {
            error.contains("got `_W`") || error.contains("does not satisfy parameter")
        })
        .count()
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
fn test_function_str_arg_rejects_bytes_literal() {
    let errors = check("def upper(s: str) -> str:\n    return s\nupper(b\"hi\")\n");
    assert!(
        errors
            .iter()
            .any(|e| e.contains("expected `str`, got `bytes`")),
        "str-annotated positional arg should reject bytes literals, got: {errors:?}"
    );

    let errors = check(
        "from typing import Any\nx: Any = b\"hi\"\ndef upper(s: str) -> str:\n    return s\nupper(\"hi\")\nupper(x)\n",
    );
    assert!(
        errors.is_empty(),
        "valid str literal and dynamic Any calls should remain accepted, got: {errors:?}"
    );
}

#[test]
fn external_builtin_annotations_preserve_nominal_identity() {
    let errors = check(
        "data: bytes = b\"ok\"\n\
         mutable: bytearray = bytearray(b\"ok\")\n\
         number: complex = 1j\n\
         view: memoryview = memoryview(data)\n\
         span: range = range(3)\n\
         cut: slice = slice(1)\n\
         frozen: frozenset[int] = frozenset()\n",
    );
    assert!(
        errors.is_empty(),
        "external builtin annotations must match their literals and constructors: {errors:?}"
    );

    let errors = check(
        "data: bytes = bytearray()\n\
         mutable: bytearray = b\"bad\"\n\
         number: complex = \"bad\"\n\
         view: memoryview = b\"bad\"\n",
    );
    assert!(
        errors.iter().any(|error| error.contains("expected `bytes`, got `bytearray`")),
        "bytes and bytearray must stay nominally distinct: {errors:?}"
    );
    assert!(
        errors.iter().any(|error| error.contains("expected `bytearray`, got `bytes`")),
        "bytearray and bytes must stay nominally distinct: {errors:?}"
    );
    assert!(
        errors.iter().any(|error| error.contains("expected `complex`, got `str`")),
        "complex annotations must reject strings: {errors:?}"
    );
    assert!(
        errors.iter().any(|error| error.contains("expected `memoryview`, got `bytes`")),
        "memoryview annotations must reject raw bytes: {errors:?}"
    );
}

#[test]
fn type_object_annotations_preserve_class_identity_and_subclasses() {
    let errors = check(
        "from datetime import date\nclass Base:\n    pass\nclass Child(Base):\n    pass\ndef take(cls: type[Base]) -> None:\n    pass\ndef choose() -> type[Base]:\n    return Child\ndef take_date(cls: type[date]) -> None:\n    pass\ntake(Base)\ntake(Child)\ntake_date(date)\nalias: type[Base] = Child\nvalue: Base = choose()()\ndef take_int(cls: type[int]) -> None:\n    pass\ntake_int(int)\n",
    );
    assert!(
        errors.is_empty(),
        "type[T] must accept proven class objects and preserve their instance type: {errors:?}"
    );

    let errors = check(
        "from datetime import date\nclass Base:\n    pass\nclass Other:\n    pass\ndef take(cls: type[Base]) -> None:\n    pass\ndef take_date(cls: type[date]) -> None:\n    pass\ntake(Base())\ntake(Other)\ntake(1)\ntake_date(str)\ndef take_int(cls: type[int]) -> None:\n    pass\ntake_int(str)\n",
    );
    assert_eq!(
        errors
            .iter()
            .filter(|error| error.contains("argument type mismatch"))
            .count(),
        5,
        "type[T] must reject instances, unrelated classes, and non-class values: {errors:?}"
    );
}

#[test]
fn typeshed_type_object_generic_round_trips_through_calls() {
    let errors = check(
        "from functools import total_ordering\nclass C:\n    def __lt__(self, other):\n        return True\nDecorated = total_ordering(C)\nvalue: C = Decorated()\n",
    );
    assert!(
        errors.is_empty(),
        "typeshed type[T] returns must preserve the inferred class object: {errors:?}"
    );

    let errors = check(
        "from functools import total_ordering\nclass C:\n    def __lt__(self, other):\n        return True\ntotal_ordering(C())\n",
    );
    assert!(
        has_parameter_error(&errors, "cls"),
        "typeshed type[T] parameters must reject instances: {errors:?}"
    );
}

#[test]
fn test_function_extended_arg_annotations_rejected() {
    let errors = check("def requires_count(*, count: int) -> int:\n    return count\nrequires_count(count=\"3\")\n");
    assert!(
        errors
            .iter()
            .any(|e| e.contains("expected `int`, got `str`")),
        "keyword-only int arg should reject str, got: {errors:?}"
    );

    let errors =
        check("def sum_items(*items: int) -> int:\n    return len(items)\nsum_items(\"3\")\n");
    assert!(
        errors
            .iter()
            .any(|e| e.contains("expected `int`, got `str`")),
        "*args int annotation should reject str elements, got: {errors:?}"
    );

    let errors = check(
        "def count_items(**items: int) -> int:\n    return len(items)\ncount_items(count=\"3\")\n",
    );
    assert!(
        errors
            .iter()
            .any(|e| e.contains("expected `int`, got `str`")),
        "**kwargs int annotation should reject str values, got: {errors:?}"
    );

    let errors = check(
        "def ok(*, count: int) -> int:\n    return count\ndef var(*items: int) -> int:\n    return len(items)\ndef kw(**items: int) -> int:\n    return len(items)\nok(count=3)\nvar(1, 2)\nkw(count=3)\n",
    );
    assert!(
        errors.is_empty(),
        "valid keyword-only, *args, and **kwargs calls should remain accepted, got: {errors:?}"
    );
}

#[test]
fn test_unbound_method_receiver_contract_rejected() {
    let errors = check("class Box:\n    def get(self, which: int) -> int:\n        return which\nBox.get(\"not_a_box\", 3)\n");
    assert!(
        errors
            .iter()
            .any(|e| e.contains("expected `Box`, got `str`")),
        "unbound method should reject wrong receiver, got: {errors:?}"
    );

    let errors = check(
        "class Box:\n    def get(self, which: int) -> int:\n        return which\nBox.get(Box(), 3)\n",
    );
    assert!(
        errors.is_empty(),
        "unbound method should accept class instance receiver, got: {errors:?}"
    );

    let errors = check(
        "class Box:\n    def get(self, which: int) -> int:\n        return which\nbox: Box = Box()\nbox.get(3)\n",
    );
    assert!(
        errors.is_empty(),
        "instance method call should not require an explicit receiver, got: {errors:?}"
    );
}

#[test]
fn unbound_method_alias_preserves_declared_receiver_metadata() {
    let valid = check(
        "class Box:\n\
         \x20   def get(receiver, value: int = 1) -> int:\n\
         \x20       return value\n\
         method = Box.get\n\
         method(receiver=Box())\n",
    );
    assert!(
        valid.is_empty(),
        "an aliased unbound method must retain the declared receiver name and defaults: {valid:?}"
    );

    let invalid = check(
        "class Box:\n\
         \x20   def get(receiver, value: int = 1) -> int:\n\
         \x20       return value\n\
         method = Box.get\n\
         method(receiver=\"bad\")\n",
    );
    assert!(
        invalid
            .iter()
            .any(|error| error.contains("expected `Box`, got `str`")),
        "receiver metadata must remain intrinsic through method aliasing: {invalid:?}"
    );
}

#[test]
fn test_keyword_iskeyword_wall_is_universal() {
    let errors = check("from keyword import iskeyword\niskeyword(12345)\n");
    assert!(
        errors
            .iter()
            .any(|e| e.contains("expected `str`, got `int`")),
        "force typing must reject a non-str iskeyword arg, got: {errors:?}"
    );
    let errors = check("from keyword import issoftkeyword\nissoftkeyword(12345)\n");
    assert!(
        errors
            .iter()
            .any(|e| e.contains("expected `str`, got `int`")),
        "force typing must reject a non-str issoftkeyword arg, got: {errors:?}"
    );

    let errors = check("from keyword import iskeyword\niskeyword(\"match\")\n");
    assert!(
        errors.is_empty(),
        "keyword.iskeyword should accept a correctly-typed str arg, got: {errors:?}"
    );

    let errors = check("from keyword import iskeyword\nresult: str = iskeyword(\"if\")\n");
    assert!(
        errors
            .iter()
            .any(|error| error.contains("expected `str`, got `bool`")),
        "typeshed return types must participate in ordinary inference: {errors:?}"
    );
}

#[test]
fn textwrap_contract_is_universal() {
    let errors = check("from textwrap import indent\nindent(1, \"  \")\n");
    assert!(
        errors
            .iter()
            .any(|error| error.contains("expected `str`, got `int`")),
        "textwrap.indent must reject a non-str text argument: {errors:?}"
    );

    let errors = check(
        "from textwrap import indent\nresult: int = indent(\"value\", \"  \")\n",
    );
    assert!(
        errors
            .iter()
            .any(|error| error.contains("expected `int`, got `str`")),
        "textwrap.indent must propagate its str return type: {errors:?}"
    );
}

#[test]
fn adjacent_raise_does_not_disable_force_type_enforcement() {
    let errors = check(
        "from os import strerror\n\
         def probe():\n\
         \x20   try:\n\
         \x20       strerror(\"bad\")\n\
         \x20       raise AssertionError(\"expected TypeError\")\n\
         \x20   except TypeError:\n\
         \x20       pass\n",
    );
    assert!(
        errors
            .iter()
            .any(|error| error.contains("expected `int`, got `str`")),
        "an adjacent raise must not disable the stdlib type contract: {errors:?}"
    );
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
fn test_runtime_mode_allows_unknown_annotation_names() {
    let errors = check("def foo(a: THIS_DOES_NOT_EXIST) -> int:\n    return 0\n");
    assert!(
        errors
            .iter()
            .any(|e| e.contains("unknown type: `THIS_DOES_NOT_EXIST`")),
        "non-runtime checking should keep rejecting unknown annotations, got: {errors:?}"
    );

    let errors = check_runtime("def foo(a: THIS_DOES_NOT_EXIST) -> int:\n    return 0\n");
    assert!(
        errors.is_empty(),
        "runtime mode should defer unknown annotation names instead of hard-failing, got: {errors:?}"
    );
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

#[test]
fn pep695_generic_type_alias_substitutes_and_enforces_arity() {
    let errors = check(
        "type Pair[T] = tuple[T, T]\n\
         good: Pair[int] = (1, 2)\n\
         bad: Pair[int] = (1, \"two\")\n\
         too_many: Pair[int, str] = (1, 2)\n",
    );
    assert!(
        errors.iter().any(|error| error.contains("type mismatch")),
        "a specialized alias must substitute its type parameters: {errors:?}"
    );
    assert!(
        errors
            .iter()
            .any(|error| error.contains("expected 1 type arguments, got 2")),
        "a generic alias must enforce its declared arity: {errors:?}"
    );

    let errors = check("type Plain = int\nvalue: Plain[str] = 1\n");
    assert!(
        errors
            .iter()
            .any(|error| error.contains("type 'Plain' is not generic")),
        "a non-generic alias must reject specialization: {errors:?}"
    );

    let errors = check("type Duplicate = int\ntype Duplicate = str\n");
    assert_eq!(
        errors
            .iter()
            .filter(|error| error.contains("already defined in this scope"))
            .count(),
        1,
        "same-scope alias redefinition must be explicit, not silently keep the first body: {errors:?}"
    );
}

#[test]
fn pep695_generic_type_alias_applies_defaults_and_constraints() {
    let errors = check(
        "type Entry[K: (int, str), V = str] = tuple[K, V]\n\
         good: Entry[int] = (1, \"ok\")\n\
         bad_default: Entry[int] = (1, 2)\n\
         bad_constraint: Entry[float] = (1.5, \"no\")\n",
    );
    assert!(
        errors.iter().any(|error| error.contains("type mismatch")),
        "an omitted alias argument must use its declared default: {errors:?}"
    );
    assert!(
        errors
            .iter()
            .any(|error| error.contains("constrained types")),
        "an explicit alias argument must satisfy its constraints: {errors:?}"
    );

    let errors = check(
        "type Numbers[T: int] = list[T]\n\
         invalid: Numbers[str] = [\"no\"]\n",
    );
    assert!(
        errors.iter().any(|error| error.contains("bound violation")),
        "an explicit alias argument must satisfy its bound: {errors:?}"
    );

    let errors = check(
        "type Dependent[T, U = list[T]] = tuple[T, U]\n\
         valid: Dependent[int] = (1, [2])\n\
         invalid: Dependent[int] = (1, [\"bad\"])\n",
    );
    assert_eq!(
        errors
            .iter()
            .filter(|error| error.contains("type mismatch"))
            .count(),
        1,
        "a dependent alias default must substitute earlier parameters: {errors:?}"
    );
}

#[test]
fn pep695_productive_direct_recursive_alias_checks_every_level() {
    let errors = check(
        "type Node = tuple[int, Node] | None\n\
         empty: Node = None\n\
         one: Node = (1, None)\n\
         deep: Node = (1, (2, None))\n\
         bad: Node = (1, (\"bad\", None))\n",
    );
    assert_eq!(
        errors
            .iter()
            .filter(|error| error.contains("type mismatch"))
            .count(),
        1,
        "recursive back-edges must retain their leaf types: {errors:?}"
    );
    assert_eq!(errors.len(), 1, "unexpected diagnostics: {errors:?}");
    assert!(
        errors
            .iter()
            .all(|error| !error.contains("recursive type alias")),
        "a tuple-guarded recursive alias is productive: {errors:?}"
    );
}

#[test]
fn pep695_productive_mutual_recursive_alias_resolves_scc() {
    let errors = check(
        "type Left = Right\n\
         type Right = tuple[int, Left] | None\n\
         good: Left = (1, (2, None))\n\
         bad: Left = (\"bad\", None)\n",
    );
    assert_eq!(
        errors
            .iter()
            .filter(|error| error.contains("type mismatch"))
            .count(),
        1,
        "a productive mutual SCC must remain structurally checked: {errors:?}"
    );
    assert_eq!(errors.len(), 1, "unexpected diagnostics: {errors:?}");
}

#[test]
fn pep695_mutual_generic_and_periodic_recursive_aliases_are_regular() {
    let errors = check(
        "type Left[T] = tuple[T, Right[T]] | None\n\
         type Right[T] = tuple[T, Left[T]] | None\n\
         good: Left[int] = (1, (2, None))\n\
         bad: Left[int] = (1, (\"bad\", None))\n\
         type Flip[T, U] = tuple[T, Flip[U, T]] | None\n\
         flip: Flip[int, str] = (1, (\"two\", (3, None)))\n\
         bad_flip: Flip[int, str] = (1, (2, None))\n",
    );
    assert_eq!(
        errors
            .iter()
            .filter(|error| error.contains("type mismatch"))
            .count(),
        2,
        "mutual and argument-periodic recursion must close to finite graphs: {errors:?}"
    );
    assert_eq!(errors.len(), 2, "unexpected diagnostics: {errors:?}");
    assert!(
        errors
            .iter()
            .all(|error| !error.contains("unproductive recursive type alias")),
        "guarded periodic cycles are productive: {errors:?}"
    );
}

#[test]
fn pep695_union_forwarding_requires_a_later_structural_guard() {
    let errors = check(
        "type Guarded = int | GuardedList\n\
         type GuardedList = list[Guarded]\n\
         good: Guarded = [1, [2]]\n\
         bad: Guarded = [\"bad\"]\n",
    );
    assert_eq!(
        errors
            .iter()
            .filter(|error| error.contains("type mismatch"))
            .count(),
        1,
        "a structural constructor reached through a union must guard recursion: {errors:?}"
    );
    assert_eq!(errors.len(), 1, "unexpected diagnostics: {errors:?}");
    assert!(
        errors
            .iter()
            .all(|error| !error.contains("unproductive recursive type alias")),
        "the list edge makes this mutual cycle productive: {errors:?}"
    );
}

#[test]
fn pep695_generic_recursive_alias_specializations_are_independent() {
    let errors = check(
        "type Chain[T] = tuple[T, Chain[T]] | None\n\
         ints: Chain[int] = (1, (2, None))\n\
         texts: Chain[str] = (\"a\", (\"b\", None))\n\
         bad_nested: Chain[int] = (1, (\"bad\", None))\n\
         bad_cross: Chain[int] = texts\n",
    );
    assert_eq!(
        errors
            .iter()
            .filter(|error| error.contains("type mismatch"))
            .count(),
        2,
        "recursive generic instances must substitute and cache by arguments: {errors:?}"
    );
    assert_eq!(errors.len(), 2, "unexpected diagnostics: {errors:?}");
}

#[test]
fn pep695_productive_argument_changing_recursive_alias_materializes_lazily() {
    let errors = check(
        "type Growing[T] = tuple[T, Growing[list[T]]] | None\n\
         good: Growing[int] = (1, ([2], ([[3]], None)))\n\
         bad: Growing[int] = (1, ([2], (\"bad\", None)))\n",
    );
    assert_eq!(
        errors
            .iter()
            .filter(|error| error.contains("type mismatch"))
            .count(),
        1,
        "argument-changing productive recursion must expand only as deeply as consumed: {errors:?}"
    );
    assert_eq!(errors.len(), 1, "unexpected diagnostics: {errors:?}");
}

#[test]
fn pep695_generic_inference_reads_recursive_alias_arguments() {
    let errors = check(
        "type Chain[T] = list[Chain[T]]\n\
         def inferred[T](value: Chain[T]) -> T:\n\
         \x20   raise RuntimeError()\n\
         ints: Chain[int] = []\n\
         good: int = inferred(ints)\n\
         bad: str = inferred(ints)\n",
    );
    assert_eq!(
        errors
            .iter()
            .filter(|error| error.contains("type mismatch"))
            .count(),
        1,
        "generic inference must unify recursive alias instance arguments: {errors:?}"
    );
    assert_eq!(errors.len(), 1, "unexpected diagnostics: {errors:?}");
}

#[test]
fn pep695_nullable_recursive_alias_participates_in_generic_inference() {
    let errors = check(
        "type Chain[T] = tuple[T, Chain[T]] | None\n\
         def inferred[T](value: Chain[T]) -> T:\n\
         \x20   raise RuntimeError()\n\
         ints: Chain[int] = None\n\
         good: int = inferred(ints)\n\
         bad: str = inferred(ints)\n",
    );
    assert_eq!(
        errors
            .iter()
            .filter(|error| error.contains("type mismatch"))
            .count(),
        1,
        "union-shaped recursive aliases must still infer their parameters: {errors:?}"
    );
    assert_eq!(errors.len(), 1, "unexpected diagnostics: {errors:?}");
}

#[test]
fn pep695_non_regular_alias_comparison_is_depth_bounded() {
    let errors = check(
        "type Growing[T] = tuple[T, Growing[list[T]]] | None\n\
         ints: Growing[int] = None\n\
         bools: Growing[bool] = None\n\
         bounded: Growing[int] = bools\n\
         type Other[T] = tuple[T, Other[list[T]]] | None\n\
         other: Other[int] = ints\n",
    );
    assert_eq!(
        errors
            .iter()
            .filter(|error| error.contains("type mismatch"))
            .count(),
        2,
        "non-regular coinduction must terminate conservatively: {errors:?}"
    );
    assert_eq!(errors.len(), 2, "unexpected diagnostics: {errors:?}");
}

#[test]
fn pep695_recursive_alias_captures_enclosing_class_type_params() {
    let src = "class Holder[T]:\n\
         \x20   type Chain = tuple[T, Chain] | None\n\
         \x20   def accept(self, value: Chain) -> None:\n\
         \x20       pass\n\
         Holder[int]().accept((1, (2, None)))\n\
         Holder[int]().accept((1, (\"bad\", None)))\n";
    for (stage, errors) in [("source", check(src)), ("desugared", check_desugared(src))] {
        assert_eq!(
            errors
                .iter()
                .filter(|error| {
                    error.contains("type mismatch") || error.contains("expected `int`")
                })
                .count(),
            1,
            "captured class TypeVars must specialize through every recursive edge ({stage}): {errors:?}"
        );
        assert_eq!(
            errors.len(),
            1,
            "unexpected diagnostics after {stage} checking: {errors:?}"
        );
    }
}

#[test]
fn pep695_save_probe_shape_does_not_hide_user_undefined_names() {
    let errors = check(
        "try:\n\
         \x20   __mb_pep695_saved_0_0 = missing\n\
         \x20   __mb_pep695_had_0_0 = True\n\
         except NameError:\n\
         \x20   __mb_pep695_had_0_0 = False\n",
    );
    assert_eq!(errors.len(), 1, "unexpected diagnostics: {errors:?}");
    assert!(
        errors[0].contains("undefined name: `missing`"),
        "user-authored lookalikes must not gain synthetic-name privileges: {errors:?}"
    );
}

#[test]
fn pep695_quoted_recursive_forward_ref_is_not_any() {
    let errors = check(
        "type Quoted = tuple[int, \"Quoted\"] | None\n\
         good: Quoted = (1, (2, None))\n\
         bad: Quoted = (1, (\"bad\", None))\n",
    );
    assert_eq!(
        errors
            .iter()
            .filter(|error| error.contains("type mismatch"))
            .count(),
        1,
        "alias-local quoted forward refs must preserve recursion: {errors:?}"
    );
    assert_eq!(errors.len(), 1, "unexpected diagnostics: {errors:?}");
}

#[test]
fn pep695_quoted_recursive_forward_ref_parses_full_type_expressions() {
    let errors = check(
        "type Chain[T] = tuple[T, \"Chain[T]\"] | None\n\
         good: Chain[int] = (1, (2, None))\n\
         bad: Chain[int] = (1, (\"bad\", None))\n\
         type Left = \"Right | None\"\n\
         type Right = list[Left]\n\
         left: Left = [None]\n",
    );
    assert_eq!(
        errors.len(),
        1,
        "compound quoted forward refs must be parsed rather than erased: {errors:?}"
    );
    assert!(errors[0].contains("type mismatch"), "{errors:?}");
}

#[test]
fn pep695_recursive_alias_compatibility_is_coinductive_and_discriminating() {
    let errors = check(
        "type IntA = tuple[int, IntA] | None\n\
         type IntB = tuple[int, IntB] | None\n\
         type StrPath = tuple[str, StrPath] | None\n\
         a: IntA = None\n\
         b: IntB = None\n\
         s: StrPath = None\n\
         ok_ab: IntA = b\n\
         ok_ba: IntB = a\n\
         bad: IntA = s\n",
    );
    assert_eq!(
        errors
            .iter()
            .filter(|error| error.contains("type mismatch"))
            .count(),
        1,
        "equivalent cycles must terminate while different leaves reject: {errors:?}"
    );
    assert_eq!(errors.len(), 1, "unexpected diagnostics: {errors:?}");
}

#[test]
fn pep695_recursive_alias_back_edges_remain_semantic_in_consumers() {
    let errors = check(
        "type Nest = list[Nest]\n\
         def bad_index(value: Nest) -> int:\n\
         \x20   return value[0][0]\n\
         def bad_iter(value: Nest) -> str:\n\
         \x20   for item in value[0]:\n\
         \x20       return item\n\
         \x20   return \"\"\n\
         def bad_method(value: Nest) -> None:\n\
         \x20   value[0].append(1)\n",
    );
    assert_eq!(
        errors
            .iter()
            .filter(|error| {
                error.contains("return type mismatch")
                    || error.contains("argument type mismatch")
                    || error.contains("expected `Nest`")
            })
            .count(),
        3,
        "shape consumers must unfold recursive back-edges without erasing them: {errors:?}"
    );
    assert_eq!(errors.len(), 3, "unexpected diagnostics: {errors:?}");
}

#[test]
fn pep695_unproductive_recursive_alias_cycles_are_rejected_once() {
    for (source, alias) in [
        ("type Loop = Loop\nx: Loop = 1\ny: Loop = \"x\"\n", "Loop"),
        ("type A = B\ntype B = A\nx: A = 1\ny: B = 2\n", "A"),
        ("type Generic[T] = Generic[T]\n", "Generic"),
        ("type Changing[T] = Changing[list[T]]\n", "Changing"),
        ("type UnionLoop = int | UnionLoop\n", "UnionLoop"),
        (
            "type UnionA = int | UnionB\ntype UnionB = str | UnionA\n",
            "UnionA",
        ),
    ] {
        let errors = check(source);
        assert_eq!(errors.len(), 1, "unexpected diagnostics: {errors:?}");
        assert_eq!(
            errors
                .iter()
                .filter(|error| error.contains("unproductive recursive type alias"))
                .count(),
            1,
            "a head-only alias cycle must produce one stable diagnostic: {errors:?}"
        );
        assert!(errors[0].contains(alias), "missing alias name: {errors:?}");
        assert!(
            errors.iter().all(|error| !error.contains("unknown type")),
            "cycle detection must not degrade into name lookup errors: {errors:?}"
        );
    }
}

#[test]
fn pep695_type_aliases_are_forward_resolved_and_lexically_scoped() {
    let errors = check(
        "type Forward = Later\n\
         type Later = int\n\
         top_bad: Forward = \"bad\"\n\
         def local() -> None:\n\
         \x20   type Later = str\n\
         \x20   inner_bad: Later = 1\n\
         outside_bad: Later = \"bad\"\n",
    );
    assert_eq!(
        errors
            .iter()
            .filter(|error| error.contains("type mismatch"))
            .count(),
        3,
        "forward aliases and same-named nested aliases must keep lexical identity: {errors:?}"
    );

    let errors = check(
        "def local() -> None:\n\
         \x20   type Hidden = int\n\
         \x20   value: Hidden = 1\n\
         leaked: Hidden = 1\n",
    );
    assert!(
        errors
            .iter()
            .any(|error| error.contains("unknown type: `Hidden`")),
        "a function-local alias must not leak into its parent scope: {errors:?}"
    );

    let errors = check(
        "type T = str\n\
         type Box[T] = list[T]\n\
         box_bad: Box[int] = [\"bad\"]\n\
         outer_bad: T = 1\n",
    );
    assert_eq!(
        errors
            .iter()
            .filter(|error| error.contains("type mismatch"))
            .count(),
        2,
        "an alias parameter must shadow and then restore an outer alias name: {errors:?}"
    );

    let errors = check(
        "from typing import TypeVar\n\
         T = TypeVar(\"T\")\n\
         def local() -> None:\n\
         \x20   type T = str\n\
         \x20   bad: T = 1\n",
    );
    assert_eq!(
        errors
            .iter()
            .filter(|error| error.contains("type mismatch"))
            .count(),
        1,
        "a lexical PEP 695 alias must shadow a legacy TypeVar alias: {errors:?}"
    );

    let errors = check(
        "type Before[T] = After[T]\n\
         type After[U] = list[U]\n\
         good: Before[int] = [1]\n\
         bad: Before[int] = [\"bad\"]\n",
    );
    assert_eq!(
        errors
            .iter()
            .filter(|error| error.contains("type mismatch"))
            .count(),
        1,
        "a forward generic alias must preserve and substitute its application: {errors:?}"
    );

    let errors = check(
        "type Model = Later\n\
         class Later:\n\
         \x20   pass\n\
         value: Model = Later()\n",
    );
    assert!(
        errors.is_empty(),
        "a type alias body must remain lazy until later declarations are registered: {errors:?}"
    );

    let errors = check(
        "type Model = Later\n\
         def bad(value: Model) -> str:\n\
         \x20   return value\n\
         class Later:\n\
         \x20   pass\n",
    );
    assert_eq!(
        errors
            .iter()
            .filter(|error| error.contains("return type mismatch"))
            .count(),
        1,
        "an intervening signature must not freeze a forward alias as Any or Error: {errors:?}"
    );

    let errors = check(
        "def bad(value: Defaulted) -> str:\n\
         \x20   return value[0]\n\
         type Defaulted[T = Later] = list[T]\n\
         class Later:\n\
         \x20   pass\n",
    );
    assert_eq!(
        errors
            .iter()
            .filter(|error| error.contains("return type mismatch"))
            .count(),
        1,
        "signature refresh must run after later alias defaults are finalized: {errors:?}"
    );
}

#[test]
fn pep695_class_local_aliases_feed_field_and_method_metadata() {
    let errors = check(
        "class Container:\n\
         \x20   type Item = int\n\
         \x20   value: Item = 0\n\
         \x20   def keep(self, value: Item) -> Item:\n\
         \x20       return value\n\
         def bad_field(value: Container) -> str:\n\
         \x20   return value.value\n\
         Container().keep(\"bad\")\n",
    );
    assert_eq!(
        errors
            .iter()
            .filter(|error| error.contains("type mismatch") || error.contains("expected `int`"))
            .count(),
        2,
        "class-local aliases must resolve before field and method metadata is collected: {errors:?}"
    );

    let errors = check(
        "class Holder:\n\
         \x20   type Item = Later\n\
         \x20   type Items[T = Later] = list[T]\n\
         \x20   value: Item = None\n\
         \x20   values: Items = []\n\
         class Later:\n\
         \x20   pass\n\
         def bad_item(holder: Holder) -> str:\n\
         \x20   return holder.value\n\
         def bad_items(holder: Holder) -> list[str]:\n\
         \x20   return holder.values\n",
    );
    assert_eq!(
        errors
            .iter()
            .filter(|error| error.contains("return type mismatch"))
            .count(),
        2,
        "class metadata must be rebuilt after forward alias bodies and defaults finalize: {errors:?}"
    );

    let errors = check(
        "class BeforeAlias:\n\
         \x20   values: LaterItems = []\n\
         type LaterItems[T = LaterValue] = list[T]\n\
         class LaterValue:\n\
         \x20   pass\n\
         def bad(holder: BeforeAlias) -> list[str]:\n\
         \x20   return holder.values\n",
    );
    assert_eq!(
        errors
            .iter()
            .filter(|error| error.contains("return type mismatch"))
            .count(),
        1,
        "all module aliases must finalize before an earlier class is rebuilt: {errors:?}"
    );
}

#[test]
fn pep695_alias_headers_cover_async_and_match_bodies() {
    let errors = check(
        "async def async_scope(source: Any, manager: Any) -> None:\n\
         \x20   async for item in source:\n\
         \x20       type FromFor = int\n\
         \x20       class Later:\n\
         \x20           pass\n\
         \x20   async with manager:\n\
         \x20       type FromWith = str\n\
         \x20   type Forward = Later\n\
         \x20   bad_for: FromFor = \"bad\"\n\
         \x20   bad_with: FromWith = 1\n\
         \x20   bad_forward: Forward = \"bad\"\n\
         def match_scope(value: int) -> None:\n\
         \x20   match value:\n\
         \x20       case _:\n\
         \x20           type FromMatch = bool\n\
         \x20   bad_match: FromMatch = \"bad\"\n",
    );
    assert_eq!(
        errors
            .iter()
            .filter(|error| error.contains("type mismatch"))
            .count(),
        3,
        "same-scope async and match bodies must participate in alias preregistration: {errors:?}"
    );
    assert!(
        errors
            .iter()
            .all(|error| !error.contains("unknown type: `Later`")),
        "async-body declaration targets must be registered before aliases finalize: {errors:?}"
    );
}

#[test]
fn pep695_desugared_generic_alias_type_reaches_hir() {
    let mut module = parser::parse(
        "type Pair[T] = tuple[T, T]\n\
         def keep(value: Pair[int]) -> Pair[int]:\n\
         \x20   return value\n",
        FileId(0),
    )
    .expect("parse failed");
    crate::lower::pep695::desugar_module(&mut module);

    let mut checker = TypeChecker::new();
    let errors = checker.check_module(&module);
    assert!(errors.is_empty(), "type check failed: {errors:?}");
    let keep = checker.symbols.lookup("keep").expect("keep registered");
    let hir = crate::lower::ast_to_hir::lower_module(&module, &checker).expect("lower failed");
    let function = hir
        .functions
        .iter()
        .find(|function| function.name == keep)
        .expect("keep lowered");

    for ty in [function.params[0].1, function.return_ty] {
        let crate::types::Ty::Tuple(items) = checker.tcx.get(ty) else {
            panic!(
                "desugared Pair[int] must remain a tuple type, got {:?}",
                checker.tcx.get(ty)
            );
        };
        assert_eq!(items, &vec![checker.tcx.int(), checker.tcx.int()]);
    }
}

#[test]
fn pep695_recursive_alias_back_edge_reaches_hir_and_mir() {
    let mut module = parser::parse(
        "type Nest = list[Nest]\n\
         def peel(value: Nest) -> Nest:\n\
         \x20   return value[0][0]\n",
        FileId(0),
    )
    .expect("parse failed");
    crate::lower::pep695::desugar_module(&mut module);

    let mut checker = TypeChecker::new();
    let errors = checker.check_module(&module);
    assert!(errors.is_empty(), "type check failed: {errors:?}");
    let peel = checker.symbols.lookup("peel").expect("peel registered");
    let hir = crate::lower::ast_to_hir::lower_module(&module, &checker).expect("lower failed");
    let function = hir
        .functions
        .iter()
        .find(|function| function.name == peel)
        .expect("peel lowered");
    let crate::hir::HirStmt::Return {
        value: Some(value), ..
    } = &function.body[0]
    else {
        panic!("peel return was not lowered");
    };
    let crate::types::Ty::AliasRef(instance) = checker.tcx.get(value.ty()) else {
        panic!("nested indexing erased the recursive alias back-edge");
    };
    let head = checker
        .tcx
        .semantic_head_id(value.ty())
        .expect("recursive alias head remained unresolved");
    let crate::types::Ty::List(nested) = checker.tcx.get(head) else {
        panic!("recursive alias lost its productive list head");
    };
    assert_eq!(
        checker.tcx.get(*nested),
        &crate::types::Ty::AliasRef(*instance)
    );

    let _mir = crate::lower::lower_hir_to_mir(&hir, &checker.tcx);
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
fn pep695_bound_constraint_metadata() {
    let module = parser::parse(
        "def bounded[T: float](x: T) -> T:\n\
         \x20   return x\n\
         def constrained[U: (int, str)](x: U) -> U:\n\
         \x20   return x\n",
        FileId(0),
    )
    .expect("parse failed");
    let mut checker = TypeChecker::new();
    let errors = checker.check_module(&module);
    assert!(
        errors.is_empty(),
        "metadata declarations should check: {errors:?}"
    );

    let bounded_symbol = checker.symbols.lookup("bounded").unwrap();
    let bounded = &checker.generic_defs[&bounded_symbol].params[0];
    assert_eq!(bounded.bound, Some(checker.tcx.float()));
    assert!(bounded.constraints.is_empty());
    let bounded_info = checker.tcx.get_type_var(bounded.id);
    assert_eq!(bounded_info.bound, bounded.bound);
    assert_eq!(bounded_info.constraints, bounded.constraints);

    let constrained_symbol = checker.symbols.lookup("constrained").unwrap();
    let constrained = &checker.generic_defs[&constrained_symbol].params[0];
    assert_eq!(
        constrained.constraints,
        vec![checker.tcx.int(), checker.tcx.str()]
    );
    let constrained_info = checker.tcx.get_type_var(constrained.id);
    assert_eq!(constrained_info.bound, constrained.bound);
    assert_eq!(constrained_info.constraints, constrained.constraints);
}

#[test]
fn pep695_function_bound_constraint_enforcement() {
    let errors = check(
        "def widen[T: float](x: T) -> T:\n\
         \x20   return x\n\
         widen(1)\n",
    );
    assert!(
        errors.is_empty(),
        "int should satisfy a float bound: {errors:?}"
    );

    let errors = check(
        "def widen[T: float](x: T) -> T:\n\
         \x20   return x\n\
         widen(\"bad\")\n",
    );
    assert!(
        errors.iter().any(|error| error.contains("bound violation")),
        "str should violate a float bound: {errors:?}"
    );

    let errors = check(
        "def choose[T: (int, str)](x: T) -> T:\n\
         \x20   return x\n\
         choose(1)\n\
         choose(\"ok\")\n",
    );
    assert!(
        errors.is_empty(),
        "declared constrained types should be accepted: {errors:?}"
    );

    let errors = check(
        "def choose[T: (int, str)](x: T) -> T:\n\
         \x20   return x\n\
         choose(1.5)\n",
    );
    assert!(
        errors
            .iter()
            .any(|error| error.contains("constrained types")),
        "float should violate int/str constraints: {errors:?}"
    );
}

#[test]
fn pep695_class_bound_constraint_enforcement() {
    let errors = check(
        "class NumericBox[T: float]:\n\
         \x20   def __init__(self, value: T) -> None:\n\
         \x20       pass\n\
         NumericBox(1)\n\
         def accepts(box: NumericBox[int]) -> None:\n\
         \x20   pass\n",
    );
    assert!(
        errors.is_empty(),
        "class construction and specialization should accept a valid bound: {errors:?}"
    );

    let errors = check(
        "class NumericBox[T: float]:\n\
         \x20   def __init__(self, value: T) -> None:\n\
         \x20       pass\n\
         NumericBox(\"bad\")\n\
         def rejects(box: NumericBox[str]) -> None:\n\
         \x20   pass\n",
    );
    assert!(
        errors.iter().any(|error| error.contains("bound violation")),
        "class calls and explicit specialization should reject invalid bounds: {errors:?}"
    );

    let errors = check(
        "class NumericBox[T: float]:\n\
         \x20   pass\n\
         def rejects(box: NumericBox[str]) -> None:\n\
         \x20   pass\n",
    );
    assert!(
        errors.iter().any(|error| error.contains("bound violation")),
        "explicit specialization should independently reject an invalid bound: {errors:?}"
    );

    let errors = check(
        "class Choice[T: (int, str)]:\n\
         \x20   def __init__(self, value: T) -> None:\n\
         \x20       pass\n\
         Choice(1.5)\n",
    );
    assert!(
        errors
            .iter()
            .any(|error| error.contains("constrained types")),
        "class construction should reject a type outside its constraints: {errors:?}"
    );
}

#[test]
fn pep695_forward_bound_is_skip_safe() {
    let module = parser::parse(
        "def keep[T: Later](value: T) -> T:\n\
         \x20   return value\n\
         class Later:\n\
         \x20   pass\n",
        FileId(0),
    )
    .expect("parse failed");
    let mut checker = TypeChecker::new();
    let errors: Vec<_> = checker
        .check_module(&module)
        .into_iter()
        .map(|error| error.to_string())
        .collect();
    assert!(
        errors.is_empty(),
        "a lazy forward bound must not emit an early unknown-type error: {errors:?}"
    );

    let later_sym = checker.symbols.lookup("Later").expect("Later registered");
    let later_ty = checker.get_sym_type(later_sym.0);
    let later_instance = checker.with_class_role(later_ty, ClassRole::Instance);
    let keep_symbol = checker.symbols.lookup("keep").unwrap();
    let keep_param = &checker.generic_defs[&keep_symbol].params[0];
    assert_eq!(keep_param.bound, Some(later_instance));
    assert_eq!(
        checker.tcx.get_type_var(keep_param.id).bound,
        Some(later_instance)
    );

    let errors = check(
        "def keep[T: Later](value: T) -> T:\n\
         \x20   return value\n\
         class Later:\n\
         \x20   pass\n\
         class Other:\n\
         \x20   pass\n\
         keep(Later())\n\
         keep(Other())\n",
    );
    assert!(
        errors.iter().any(|error| error.contains("bound violation")),
        "a finalized forward bound must reject an unrelated class: {errors:?}"
    );
}

#[test]
fn pep695_bound_must_be_concrete() {
    let errors = check(
        "def invalid[U, T: U](left: U, right: T) -> T:\n\
         \x20   return right\n",
    );
    assert!(
        errors
            .iter()
            .any(|error| error.contains("bound must be concrete")),
        "PEP 695 rejects bounds parameterized by another TypeVar: {errors:?}"
    );

    let errors = check(
        "def invalid[U, T: list[U]](value: T) -> T:\n\
         \x20   return value\n",
    );
    assert!(
        errors
            .iter()
            .any(|error| error.contains("bound must be concrete")),
        "nested TypeVars also make a bound non-concrete: {errors:?}"
    );
}

#[test]
fn pep695_keyword_arguments_participate_in_inference() {
    let errors = check(
        "def choose[T: (int, str)](value: T) -> T:\n\
         \x20   return value\n\
         choose(value=1)\n\
         choose(value=\"ok\")\n",
    );
    assert!(
        errors.is_empty(),
        "valid keyword arguments should infer constrained types: {errors:?}"
    );

    let errors = check(
        "def choose[T: (int, str)](value: T) -> T:\n\
         \x20   return value\n\
         choose(value=1.5)\n",
    );
    assert!(
        errors
            .iter()
            .any(|error| error.contains("constrained types")),
        "invalid keyword arguments must not bypass generic constraints: {errors:?}"
    );

    let errors = check(
        "class Choice[T: (int, str)]:\n\
         \x20   def __init__(self, value: T) -> None:\n\
         \x20       pass\n\
         Choice(value=1)\n\
         Choice(value=\"ok\")\n",
    );
    assert!(
        errors.is_empty(),
        "valid constructor keywords should infer constrained types: {errors:?}"
    );

    let errors = check(
        "class Choice[T: (int, str)]:\n\
         \x20   def __init__(self, value: T) -> None:\n\
         \x20       pass\n\
         Choice(value=1.5)\n",
    );
    assert!(
        errors
            .iter()
            .any(|error| error.contains("constrained types")),
        "invalid constructor keywords must not bypass generic constraints: {errors:?}"
    );
}

#[test]
fn pep695_argument_binder_covers_parameter_kinds() {
    let errors = check(
        "def choose[T: (int, str)](*, value: T) -> T:\n\
         \x20   return value\n\
         choose(value=1.5)\n",
    );
    assert!(
        errors
            .iter()
            .any(|error| error.contains("constrained types")),
        "keyword-only parameters must contribute inference evidence: {errors:?}"
    );

    let errors = check(
        "def collect[T: (int, str)](*values: T) -> None:\n\
         \x20   pass\n\
         collect(1.5)\n",
    );
    assert!(
        errors
            .iter()
            .any(|error| error.contains("constrained types")),
        "declared *args must consume explicit positional arguments: {errors:?}"
    );

    let errors = check(
        "def collect[T: (int, str)](**values: T) -> None:\n\
         \x20   pass\n\
         collect(bad=1.5)\n",
    );
    assert!(
        errors
            .iter()
            .any(|error| error.contains("constrained types")),
        "declared **kwargs must consume unmatched explicit keywords: {errors:?}"
    );

    let errors = check(
        "def collect[T: (int, str)](value: int, /, **rest: T) -> None:\n\
         \x20   pass\n\
         collect(1, value=1.5)\n",
    );
    assert!(
        errors
            .iter()
            .any(|error| error.contains("constrained types")),
        "a positional-only name passed by keyword must bind through **kwargs: {errors:?}"
    );

    let errors = check(
        "def choose[T: int](value: T) -> T:\n\
         \x20   return value\n\
         values: list[str] = [\"dynamic\"]\n\
         choose(*values)\n",
    );
    assert!(
        errors.is_empty(),
        "dynamic call-site spreads must remain skip-safe for inference: {errors:?}"
    );
}

#[test]
fn pep695_constructor_arguments_are_checked_once() {
    let errors = check(
        "class Box[T]:\n\
         \x20   def __init__(self, value: T) -> None:\n\
         \x20       pass\n\
         Box(missing)\n",
    );
    let undefined_count = errors
        .iter()
        .filter(|error| error.contains("undefined name: `missing`"))
        .count();
    assert_eq!(
        undefined_count, 1,
        "constructor argument expressions must be checked once: {errors:?}"
    );
}

#[test]
fn pep695_nested_function_uses_symbol_identity() {
    let errors = check(
        "def pick[T: int](value: T) -> T:\n\
         \x20   return value\n\
         def outer() -> None:\n\
         \x20   def pick[T: str](value: T) -> T:\n\
         \x20       return value\n\
         \x20   pick(1)\n\
         outer()\n\
         pick(\"bad\")\n",
    );
    let violations = errors
        .iter()
        .filter(|error| error.contains("bound violation"))
        .count();
    assert_eq!(
        violations, 2,
        "nested and top-level definitions must keep distinct generic metadata: {errors:?}"
    );
}

#[test]
fn pep695_nested_forward_bound_is_finalized_in_lexical_scope() {
    let errors = check(
        "def outer() -> None:\n\
         \x20   def keep[T: Later](value: T) -> T:\n\
         \x20       return value\n\
         \x20   class Later:\n\
         \x20       pass\n\
         \x20   class Other:\n\
         \x20       pass\n\
         \x20   keep(Other())\n\
         outer()\n",
    );
    assert!(
        errors.iter().any(|error| error.contains("bound violation")),
        "nested forward bounds must resolve before checking calls: {errors:?}"
    );
}

#[test]
fn pep695_nested_class_constructor_uses_symbol_identity() {
    let errors = check(
        "class Box[T: int]:\n\
         \x20   def __init__(self, value: T) -> None:\n\
         \x20       pass\n\
         def outer() -> None:\n\
         \x20   class Box[T: str]:\n\
         \x20       def __init__(self, value: T) -> None:\n\
         \x20           pass\n\
         \x20   Box(1)\n\
         outer()\n\
         Box(\"bad\")\n",
    );
    let violations = errors
        .iter()
        .filter(|error| error.contains("bound violation"))
        .count();
    assert_eq!(
        violations, 2,
        "nested and top-level classes must keep distinct constructor metadata: {errors:?}"
    );
}

#[test]
fn pep695_generic_method_enforces_bounds_and_infers_return() {
    let errors = check(
        "class Picker:\n\
         \x20   def pick[T: int](self, value: T) -> T:\n\
         \x20       return value\n\
         picker = Picker()\n\
         good: int = picker.pick(1)\n\
         picker.pick(\"bad\")\n",
    );
    let violations = errors
        .iter()
        .filter(|error| error.contains("bound violation"))
        .count();
    assert_eq!(
        violations, 1,
        "generic methods must enforce their local bounds: {errors:?}"
    );

    let errors = check(
        "class Identity:\n\
         \x20   def apply[T](self, value: T) -> T:\n\
         \x20       return value\n\
         identity = Identity()\n\
         text: str = identity.apply(\"ok\")\n",
    );
    assert!(
        errors.is_empty(),
        "generic method return types should use call inference: {errors:?}"
    );
}

#[test]
fn pep695_generic_method_uses_owner_identity_and_keyword_binding() {
    let errors = check(
        "class IntPicker:\n\
         \x20   def pick[T: int](self, value: T) -> T:\n\
         \x20       return value\n\
         class StrPicker:\n\
         \x20   def pick[T: str](self, value: T) -> T:\n\
         \x20       return value\n\
         IntPicker().pick(value=\"bad\")\n\
         StrPicker().pick(value=1)\n",
    );
    let violations = errors
        .iter()
        .filter(|error| error.contains("bound violation"))
        .count();
    assert_eq!(
        violations, 2,
        "same-named methods must use their owning class metadata: {errors:?}"
    );
}

#[test]
fn pep695_generic_method_forward_bound_is_finalized() {
    let errors = check(
        "class Picker:\n\
         \x20   def pick[T: Later](self, value: T) -> T:\n\
         \x20       return value\n\
         class Later:\n\
         \x20   pass\n\
         class Other:\n\
         \x20   pass\n\
         Picker().pick(Other())\n",
    );
    assert!(
        errors.iter().any(|error| error.contains("bound violation")),
        "method-local forward bounds must be finalized after preregistration: {errors:?}"
    );
}

#[test]
fn pep695_constrained_inference_promotes_to_declared_constraint() {
    let errors = check(
        "def same[T: (int, str)](left: T, right: T) -> T:\n\
         \x20   return left\n\
         value: int = same(True, 1)\n",
    );
    assert!(
        errors.is_empty(),
        "bool and int should both solve a constrained TypeVar to int: {errors:?}"
    );

    let errors = check(
        "def choose[T: (int, str)](value: T) -> T:\n\
         \x20   return value\n\
         result: bool = choose(True)\n",
    );
    assert!(
        errors.iter().any(|error| error.contains("type mismatch")),
        "a bool argument must promote the constrained return type to int: {errors:?}"
    );

    let errors = check(
        "def choose[T: (float, int)](value: T) -> T:\n\
         \x20   return value\n\
         result: int = choose(1)\n",
    );
    assert!(
        errors.is_empty(),
        "an exact constraint match must beat an earlier wider constraint: {errors:?}"
    );
}

#[test]
fn pep695_generic_defaults_are_checked_against_bounds() {
    let errors = check(
        "def valid[T: (int, str)](value: T = 1) -> T:\n\
         \x20   return value\n\
         def invalid[T: (int, str)](value: T = 1.5) -> T:\n\
         \x20   return value\n",
    );
    let violations = errors
        .iter()
        .filter(|error| error.contains("constrained types"))
        .count();
    assert_eq!(
        violations, 1,
        "generic parameter defaults must use constraint checking: {errors:?}"
    );

    let errors = check(
        "def valid[T: float](value: T = 1) -> T:\n\
         \x20   return value\n\
         def invalid[T: float](value: T = \"bad\") -> T:\n\
         \x20   return value\n",
    );
    let violations = errors
        .iter()
        .filter(|error| error.contains("bound violation"))
        .count();
    assert_eq!(
        violations, 1,
        "generic parameter defaults must use upper-bound checking: {errors:?}"
    );

    let errors = check(
        "class Invalid[T: (int, str) = bool]:\n\
         \x20   pass\n",
    );
    assert!(
        errors
            .iter()
            .any(|error| error.contains("violates its constraints")),
        "a constrained default must exactly match a declared constraint: {errors:?}"
    );
}

#[test]
fn pep695_type_parameter_kind_and_default_state_are_preserved() {
    let module = parser::parse(
        "class Defaults[T = int]:\n\
         \x20   pass\n\
         class Variadic[*Ts, **P]:\n\
         \x20   pass\n\
         class Lazy[T = Missing]:\n\
         \x20   pass\n",
        FileId(0),
    )
    .expect("parse failed");
    let mut checker = TypeChecker::new();
    let errors = checker.check_module(&module);
    assert!(errors.is_empty(), "metadata should remain lazy: {errors:?}");

    let defaults_symbol = checker.symbols.lookup("Defaults").unwrap();
    let default_param = &checker.generic_defs[&defaults_symbol].params[0];
    assert_eq!(default_param.kind, TypeVarKind::TypeVar);
    assert_eq!(
        default_param.default,
        TypeParamDefault::Resolved(checker.tcx.int())
    );

    let variadic_symbol = checker.symbols.lookup("Variadic").unwrap();
    let variadic = &checker.generic_defs[&variadic_symbol].params;
    assert_eq!(variadic[0].kind, TypeVarKind::TypeVarTuple);
    assert_eq!(variadic[1].kind, TypeVarKind::ParamSpec);

    let lazy_symbol = checker.symbols.lookup("Lazy").unwrap();
    assert_eq!(
        checker.generic_defs[&lazy_symbol].params[0].default,
        TypeParamDefault::Unresolved
    );
}

#[test]
fn pep695_class_specialization_enforces_arity() {
    let errors = check(
        "class Pair[T, U]:\n\
         \x20   pass\n\
         def too_few(value: Pair[int]) -> None:\n\
         \x20   pass\n\
         def too_many(value: Pair[int, str, bool]) -> None:\n\
         \x20   pass\n",
    );
    let arity_errors = errors
        .iter()
        .filter(|error| error.contains("expected 2 type arguments"))
        .count();
    assert_eq!(
        arity_errors, 2,
        "fixed generic classes must reject missing and excess type arguments: {errors:?}"
    );

    let errors = check(
        "class Plain:\n\
         \x20   pass\n\
         def invalid(value: Plain[int]) -> None:\n\
         \x20   pass\n",
    );
    assert!(
        errors
            .iter()
            .any(|error| error.contains("type 'Plain' is not generic")),
        "non-generic classes must reject specialization: {errors:?}"
    );

    let errors = check(
        "def invalid(items: list[int, str], mapping: dict[str]) -> None:\n\
         \x20   pass\n",
    );
    assert_eq!(
        errors
            .iter()
            .filter(|error| error.contains("type argument"))
            .count(),
        2,
        "fixed-arity builtin generics must reject malformed subscriptions: {errors:?}"
    );
}

#[test]
fn pep695_class_specialization_applies_defaults() {
    let errors = check(
        "class Pair[T, U = list[T]]:\n\
         \x20   value: U = None\n\
         def valid(value: Pair[int]) -> list[int]:\n\
         \x20   return value.value\n\
         def invalid(value: Pair[int]) -> list[str]:\n\
         \x20   return value.value\n",
    );
    let mismatches = errors
        .iter()
        .filter(|error| error.contains("type mismatch"))
        .count();
    assert_eq!(
        mismatches, 1,
        "trailing defaults must be substituted through earlier arguments: {errors:?}"
    );

    let errors = check(
        "class DefaultBox[T = int]:\n\
         \x20   value: T = None\n\
         def invalid(value: DefaultBox) -> str:\n\
         \x20   return value.value\n",
    );
    assert!(
        errors.iter().any(|error| error.contains("type mismatch")),
        "bare generic annotations must apply declared defaults: {errors:?}"
    );

    let errors = check(
        "class Choice[T: (int, str)]:\n\
         \x20   value: T = None\n\
         def invalid(value: Choice[bool]) -> bool:\n\
         \x20   return value.value\n",
    );
    assert!(
        errors.iter().any(|error| error.contains("type mismatch")),
        "explicit constrained subtypes must promote to their declared constraint: {errors:?}"
    );
}

#[test]
fn pep695_default_references_must_point_backward() {
    let errors = check(
        "class Invalid[T = U, U = int]:\n\
         \x20   pass\n",
    );
    assert!(
        errors
            .iter()
            .any(|error| error.contains("may only reference earlier parameters of the same kind")),
        "forward type-parameter defaults must be rejected: {errors:?}"
    );
}

#[test]
fn pep695_constructor_propagates_inference_and_defaults_to_instances() {
    let errors = check(
        "class Box[T]:\n\
         \x20   value: T = None\n\
         \x20   def __init__(self, value: T):\n\
         \x20       self.value = value\n\
         \x20   def get(self) -> T:\n\
         \x20       return self.value\n\
         \x20   def put(self, value: T) -> None:\n\
         \x20       pass\n\
         box = Box(\"text\")\n\
         bad_field: int = box.value\n\
         bad_keyword: int = Box(value=\"text\").value\n\
         bad_method: int = Box(\"text\").get()\n\
         Box(\"text\").put(value=1)\n",
    );
    assert_eq!(
        errors
            .iter()
            .filter(|error| error.contains("type mismatch"))
            .count(),
        4,
        "constructor inference must reach fields, methods, and keyword calls: {errors:?}"
    );

    let errors = check(
        "class Pair[T, U = list[T]]:\n\
         \x20   second: U = None\n\
         \x20   def __init__(self, first: T):\n\
         \x20       pass\n\
         bad: list[str] = Pair(1).second\n\
         class DefaultBox[T = int]:\n\
         \x20   value: T = None\n\
         bad_default: str = DefaultBox().value\n",
    );
    assert_eq!(
        errors
            .iter()
            .filter(|error| error.contains("type mismatch"))
            .count(),
        2,
        "unsolved constructor parameters must consume defaults in declaration order: {errors:?}"
    );

    let errors = check(
        "class Loose[T]:\n\
         \x20   value: T = None\n\
         \x20   def __init__(self):\n\
         \x20       pass\n\
         gradual: int = Loose().value\n",
    );
    assert!(
        errors.is_empty(),
        "an unsolved parameter without a default must remain gradual Any: {errors:?}"
    );
}

#[test]
fn pep695_explicit_constructor_specialization_is_preserved() {
    let errors = check(
        "class Box[T]:\n\
         \x20   value: T = None\n\
         \x20   def __init__(self, value: T):\n\
         \x20       self.value = value\n\
         \x20   def get(self) -> T:\n\
         \x20       return self.value\n\
         \x20   def put(self, value: T) -> None:\n\
         \x20       pass\n\
         bad_field: str = Box[int](1).value\n\
         bad_method: str = Box[int](1).get()\n\
         Box[int](1).put(value=\"bad\")\n\
         Box[int](\"bad\")\n",
    );
    assert_eq!(
        errors
            .iter()
            .filter(|error| error.contains("type mismatch"))
            .count(),
        4,
        "explicit type arguments must survive construction and constrain arguments: {errors:?}"
    );
}

#[test]
fn pep695_generic_function_return_substitutes_user_class() {
    let errors = check(
        "class Box[T]:\n\
         \x20   value: T = None\n\
         \x20   def __init__(self, value: T):\n\
         \x20       self.value = value\n\
         def make[T](value: T) -> Box[T]:\n\
         \x20   return Box(value)\n\
         bad: str = make(1).value\n",
    );
    assert!(
        errors.iter().any(|error| error.contains("type mismatch")),
        "generic function return substitution must recurse through user classes: {errors:?}"
    );
}

#[test]
fn pep695_constructor_checks_fixed_parameters_after_inference() {
    let errors = check(
        "class Box[T]:\n\
         \x20   def __init__(self, value: T, label: str):\n\
         \x20       pass\n\
         Box(1, 2)\n\
         Box(value=1, label=2)\n",
    );
    assert_eq!(
        errors
            .iter()
            .filter(|error| error.contains("expected `str`, got `int`"))
            .count(),
        2,
        "constructor inference must still check concrete parameters: {errors:?}"
    );
}

#[test]
fn pep695_inference_recurses_through_user_class_arguments() {
    let errors = check(
        "class Box[T]:\n\
         \x20   value: T = None\n\
         \x20   def __init__(self, value: T):\n\
         \x20       self.value = value\n\
         def unwrap[T](box: Box[T]) -> T:\n\
         \x20   return box.value\n\
         bad: str = unwrap(Box(1))\n",
    );
    assert!(
        errors.iter().any(|error| error.contains("type mismatch")),
        "inference must flow from Box[int] into a Box[T] parameter: {errors:?}"
    );
}

#[test]
fn pep695_expression_specialization_rejects_non_generic_and_deduplicates_bounds() {
    let errors = check(
        "class Plain:\n\
         \x20   pass\n\
         Plain[int]()\n",
    );
    assert_eq!(
        errors
            .iter()
            .filter(|error| error.contains("type 'Plain' is not generic"))
            .count(),
        1,
        "expression-level specialization must match annotation diagnostics: {errors:?}"
    );

    let errors = check(
        "class NumericBox[T: float]:\n\
         \x20   def __init__(self, value: T):\n\
         \x20       pass\n\
         NumericBox[str](\"bad\")\n",
    );
    assert_eq!(
        errors
            .iter()
            .filter(|error| error.contains("bound violation"))
            .count(),
        1,
        "explicit constructor bounds must be diagnosed once: {errors:?}"
    );
}

#[test]
fn pep695_parameterized_class_method_access_is_always_typed() {
    let errors = check(
        "class Box[T]:\n\
         \x20   def get(self, fallback: T) -> T:\n\
         \x20       return fallback\n\
         dynamic: str = Box[int].get()\n",
    );
    assert!(
        !errors.is_empty(),
        "an unbound parameterized method must require its receiver and arguments: {errors:?}"
    );

    let errors = check(
        "class Box[T]:\n\
         \x20   def get(self, fallback: T) -> T:\n\
         \x20       return fallback\n\
         Box[int].get(Box(1), fallback=\"bad\")\n",
    );
    assert!(
        errors
            .iter()
            .any(|error| error.contains("expected `int`, got `str`")),
        "unbound calls must retain the receiver and specialize later parameters: {errors:?}"
    );

    let errors = check(
        "class PlainBox:\n\
         \x20   def get(self, fallback: int) -> int:\n\
         \x20       return fallback\n\
         PlainBox.get(PlainBox(), fallback=\"bad\")\n",
    );
    assert!(
        errors
            .iter()
            .any(|error| error.contains("expected `int`, got `str`")),
        "bare unbound keyword calls must align metadata after the receiver: {errors:?}"
    );

    let errors = check(
        "class Box:\n\
         \x20   def get(self, fallback: int) -> int:\n\
         \x20       return fallback\n\
         def outer() -> None:\n\
         \x20   class Box:\n\
         \x20       def get(self, fallback: str) -> str:\n\
         \x20           return fallback\n\
         \x20   Box.get(Box(), fallback=\"ok\")\n\
         outer()\n\
         Box.get(Box(), fallback=\"bad\")\n",
    );
    assert!(
        errors
            .iter()
            .any(|error| error.contains("expected `int`, got `str`")),
        "same-named nested classes must not overwrite unbound method metadata: {errors:?}"
    );
}

#[test]
fn user_class_objects_are_distinct_from_instances() {
    let errors = check(
        "class Box[T]:\n\
         \x20   value: T = None\n\
         \x20   def __init__(self, value: T):\n\
         \x20       self.value = value\n\
         good: Box[int] = Box[int](1)\n\
         bad: Box[int] = Box[int]\n",
    );
    assert_eq!(
        errors
            .iter()
            .filter(|error| error.contains("type mismatch"))
            .count(),
        1,
        "a class object must not satisfy its instance annotation: {errors:?}"
    );

    let errors = check(
        "class Plain:\n\
         \x20   def __init__(self, value: str):\n\
         \x20       pass\n\
         Plain(1)\n\
         Plain()()\n\
         class Callable:\n\
         \x20   def __call__(self, value: int) -> int:\n\
         \x20       return value\n\
         Callable()(1)\n\
         class Child(Callable):\n\
         \x20   pass\n\
         Child()(1)\n\
         Alias = Callable\n\
         class AliasChild(Alias):\n\
         \x20   pass\n\
         AliasChild()(1)\n\
         class GenericCallable[T]:\n\
         \x20   def __call__(self, value: T) -> T:\n\
         \x20       return value\n\
         class GenericChild(GenericCallable[int]):\n\
         \x20   pass\n\
         GenericChild()(1)\n\
         class SameName:\n\
         \x20   pass\n\
         def install() -> None:\n\
         \x20   class SameName:\n\
         \x20       def __call__(self, value: int) -> int:\n\
         \x20           return value\n\
         \x20   SameName()(1)\n\
         install()\n\
         SameName()()\n",
    );
    assert_eq!(
        errors
            .iter()
            .filter(|error| error.contains("called value is not a function"))
            .count(),
        2,
        "only nominal classes with own or inherited __call__ remain callable: {errors:?}"
    );
    assert_eq!(
        errors
            .iter()
            .filter(|error| error.contains("expected `str`, got `int`"))
            .count(),
        1,
        "non-generic constructors must enforce __init__ annotations: {errors:?}"
    );
}

#[test]
fn builtin_and_native_class_objects_are_distinct_from_instances() {
    let errors = check(
        "good: ValueError = ValueError(\"boom\")\n\
         bad: ValueError = ValueError\n\
         err = ValueError(\"boom\")\n\
         err()\n",
    );
    assert_eq!(
        errors
            .iter()
            .filter(|error| error.contains("type mismatch"))
            .count(),
        1,
        "a builtin class object must not satisfy its instance annotation: {errors:?}"
    );
    assert_eq!(
        errors
            .iter()
            .filter(|error| error.contains("called value is not a function"))
            .count(),
        1,
        "a constructed builtin exception must not remain a constructor: {errors:?}"
    );
    assert_eq!(errors.len(), 2, "unexpected diagnostics: {errors:?}");

    let errors = check(
        "ErrorAlias = ValueError\n\
         def take(error: ErrorAlias) -> None:\n\
         \x20   pass\n\
         take(ErrorAlias(\"ok\"))\n\
         take(ErrorAlias)\n",
    );
    assert_eq!(
        errors
            .iter()
            .filter(|error| error.contains("type mismatch"))
            .count(),
        1,
        "builtin class aliases must remain constructors but not instances: {errors:?}"
    );
    assert_eq!(errors.len(), 1, "unexpected diagnostics: {errors:?}");

    let errors = check(
        "import queue\n\
         q = queue.Queue()\n\
         q.qsize()\n\
         q()\n\
         QueueAlias = queue.Queue\n\
         QueueAlias().qsize()\n\
         QueueAlias()()\n",
    );
    assert_eq!(
        errors
            .iter()
            .filter(|error| error.contains("called value is not a function"))
            .count(),
        2,
        "native constructor results must be instances while aliases remain callable: {errors:?}"
    );
    assert_eq!(errors.len(), 2, "unexpected diagnostics: {errors:?}");
}

#[test]
fn user_class_object_aliases_preserve_specialization_and_construction() {
    let errors = check(
        "class Box[T]:\n\
         \x20   value: T = None\n\
         Open = Box\n\
         def keep(value: Open[int]) -> int:\n\
         \x20   return value.value\n",
    );
    assert!(
        errors.is_empty(),
        "class aliases must be available while later signatures are preregistered: {errors:?}"
    );

    let errors = check(
        "class Box[T]:\n\
         \x20   value: T = None\n\
         \x20   def __init__(self, value: T):\n\
         \x20       self.value = value\n\
         \x20   def get(self, fallback: T) -> T:\n\
         \x20       return fallback\n\
         Fixed = Box[int]\n\
         Fixed(\"bad\")\n\
         bad_fixed: str = Fixed(1).value\n\
         def fixed_annotation(value: Fixed) -> str:\n\
         \x20   return value.value\n\
         dynamic: str = Fixed.get()\n\
         Open = Box\n\
         Second = Open\n\
         bad_open: str = Second[int](1).value\n\
         def from_alias(value: Open[int]) -> str:\n\
         \x20   return value.value\n",
    );
    assert_eq!(
        errors
            .iter()
            .filter(|error| error.contains("type mismatch"))
            .count(),
        6,
        "fixed, open, and chained class aliases must preserve object role and type args: {errors:?}"
    );
    assert_eq!(
        errors.len(),
        6,
        "alias preregistration must not leave extra unknown-type diagnostics: {errors:?}"
    );

    let errors = check(
        "class Box[T]:\n\
         \x20   pass\n\
         Fixed = Box[int]\n\
         def invalid(value: Fixed[str]) -> None:\n\
         \x20   pass\n",
    );
    assert_eq!(
        errors
            .iter()
            .filter(|error| error.contains("type 'Fixed' is already specialized"))
            .count(),
        1,
        "a fixed class alias cannot be specialized again: {errors:?}"
    );

    let errors = check(
        "class Box[T]:\n\
         \x20   value: T = None\n\
         \x20   def __init__(self, value: T):\n\
         \x20       self.value = value\n\
         def local() -> str:\n\
         \x20   Alias = Box[int]\n\
         \x20   return Alias(1).value\n",
    );
    assert!(
        errors
            .iter()
            .any(|error| error.contains("return type mismatch")),
        "function-local class aliases must refine lexical Any placeholders: {errors:?}"
    );

    let errors = check(
        "class Box[T]:\n\
         \x20   def __init__(self, value: T):\n\
         \x20       pass\n\
         \x20   def get(self, fallback: T) -> T:\n\
         \x20       return fallback\n\
         Alias = Box[int]\n\
         Alias.get(Alias(1), fallback=\"bad\")\n",
    );
    assert!(
        errors
            .iter()
            .any(|error| error.contains("expected `int`, got `str`")),
        "strict unbound calls through aliases must retain declaration metadata: {errors:?}"
    );
}

#[test]
fn user_class_annotation_role_reaches_hir() {
    let module = parser::parse(
        "class Box[T]:\n\
         \x20   pass\n\
         def keep(value: Box[int]) -> Box[int]:\n\
         \x20   return value\n",
        FileId(0),
    )
    .expect("parse failed");
    let mut checker = TypeChecker::new();
    let errors = checker.check_module(&module);
    assert!(errors.is_empty(), "type check failed: {errors:?}");
    let keep = checker.symbols.lookup("keep").expect("keep registered");
    let hir = crate::lower::ast_to_hir::lower_module(&module, &checker).expect("lower failed");
    let function = hir
        .functions
        .iter()
        .find(|function| function.name == keep)
        .expect("keep lowered");

    for ty in [function.params[0].1, function.return_ty] {
        let crate::types::Ty::Class {
            role,
            user: Some(user),
            ..
        } = checker.tcx.get(ty)
        else {
            panic!(
                "expected lowered user class type, got {:?}",
                checker.tcx.get(ty)
            );
        };
        assert_eq!(*role, ClassRole::Instance);
        assert_eq!(user.args, vec![checker.tcx.int()]);
    }
}

#[test]
fn user_class_alias_preregistration_tracks_rebinding() {
    let module = parser::parse(
        "class A:\n\
         \x20   value: int = 0\n\
         class B:\n\
         \x20   value: str = \"b\"\n\
         Alias = A\n\
         Alias = B\n\
         bad: int = Alias().value\n\
         def take(value: Alias) -> None:\n\
         \x20   pass\n",
        FileId(0),
    )
    .expect("parse failed");
    let mut checker = TypeChecker::new();
    let errors: Vec<_> = checker
        .check_module(&module)
        .into_iter()
        .map(|error| error.to_string())
        .collect();
    assert_eq!(
        errors
            .iter()
            .filter(|error| error.contains("type mismatch"))
            .count(),
        1,
        "runtime alias calls must use the final sequential binding: {errors:?}"
    );
    let take = checker.symbols.lookup("take").expect("take registered");
    let b = checker.symbols.lookup("B").expect("B registered");
    let crate::types::Ty::Fn { params, .. } = checker.tcx.get(checker.get_sym_type(take.0)) else {
        panic!("take must be a function");
    };
    let crate::types::Ty::Class {
        role,
        user: Some(user),
        ..
    } = checker.tcx.get(params[0])
    else {
        panic!("rebound Alias must resolve to B instance");
    };
    assert_eq!(user.symbol, b);
    assert_eq!(*role, ClassRole::Instance);

    for source in [
        "class A:\n    pass\nAlias = A\nAlias = 42\ndef take(value: Alias) -> None:\n    pass\n",
        "class A:\n    pass\nclass B:\n    pass\nif True:\n    Alias = A\nelse:\n    Alias = B\ndef take(value: Alias) -> None:\n    pass\n",
    ] {
        let module = parser::parse(source, FileId(0)).expect("parse failed");
        let mut checker = TypeChecker::new();
        let errors = checker.check_module(&module);
        assert!(errors.is_empty(), "ambiguous alias failed: {errors:?}");
        let take = checker.symbols.lookup("take").expect("take registered");
        let alias = checker.symbols.lookup("Alias").expect("Alias registered");
        assert_eq!(checker.get_sym_type(alias.0), checker.tcx.any());
        let crate::types::Ty::Fn { params, .. } = checker.tcx.get(checker.get_sym_type(take.0))
        else {
            panic!("take must be a function");
        };
        assert_eq!(params, &vec![checker.tcx.any()]);
    }
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
fn test_match_class_pattern_rejects_instance_target() {
    let errors = check(
        "class C:\n\
         \x20   pass\n\
         pattern = C()\n\
         subject = C()\n\
         match subject:\n\
         \x20   case pattern() as captured:\n\
         \x20       pass\n",
    );
    assert_eq!(
        errors
            .iter()
            .filter(|error| error.contains("class pattern target must be a class"))
            .count(),
        1,
        "a known instance must not be accepted as a class-pattern head: {errors:?}"
    );
    assert_eq!(errors.len(), 1, "unexpected diagnostics: {errors:?}");

    let errors = check(
        "class C:\n\
         \x20   pass\n\
         Alias = C\n\
         match C():\n\
         \x20   case Alias():\n\
         \x20       pass\n",
    );
    assert!(
        errors.is_empty(),
        "a class-object alias must remain a valid pattern head: {errors:?}"
    );

    let errors = check(
        "from external import C\n\
         match 1:\n\
         \x20   case C():\n\
         \x20       pass\n",
    );
    assert!(
        errors.is_empty(),
        "an unresolved imported pattern head must stay skip-when-unsure: {errors:?}"
    );

    let errors = check(
        "class C:\n\
         \x20   pass\n\
         int = C()\n\
         match C():\n\
         \x20   case int():\n\
         \x20       pass\n",
    );
    assert_eq!(
        errors
            .iter()
            .filter(|error| error.contains("class pattern target must be a class"))
            .count(),
        1,
        "a shadowed builtin instance must not remain a valid pattern head: {errors:?}"
    );

    let errors = check(
        "class C:\n\
         \x20   pass\n\
         ValueError = C()\n\
         match C():\n\
         \x20   case ValueError():\n\
         \x20       pass\n",
    );
    assert_eq!(
        errors
            .iter()
            .filter(|error| error.contains("class pattern target must be a class"))
            .count(),
        1,
        "a shadowed builtin exception must not remain a valid pattern head: {errors:?}"
    );

    let errors = check(
        "match ValueError(\"boom\"):\n\
         \x20   case ValueError():\n\
         \x20       pass\n",
    );
    assert!(
        errors.is_empty(),
        "an unshadowed builtin exception must remain a valid pattern head: {errors:?}"
    );

    let errors = check(
        "class C:\n\
         \x20   pass\n\
         def local() -> None:\n\
         \x20   pattern = C()\n\
         \x20   int = C()\n\
         \x20   ValueError = C()\n\
         \x20   match C():\n\
         \x20       case pattern():\n\
         \x20           pass\n\
         \x20   match C():\n\
         \x20       case int():\n\
         \x20           pass\n\
         \x20   match C():\n\
         \x20       case ValueError():\n\
         \x20           pass\n",
    );
    assert_eq!(
        errors
            .iter()
            .filter(|error| error.contains("class pattern target must be a class"))
            .count(),
        3,
        "first assignment must refine function-local Any placeholders: {errors:?}"
    );

    let errors = check(
        "def local() -> None:\n\
         \x20   Alias = int\n\
         \x20   match 1:\n\
         \x20       case Alias(value):\n\
         \x20           keep: int = value\n",
    );
    assert!(
        errors.is_empty(),
        "function-local primitive class aliases must retain provenance: {errors:?}"
    );

    let errors = check(
        "class C:\n\
         \x20   pass\n\
         def outer() -> None:\n\
         \x20   def inner() -> None:\n\
         \x20       match C():\n\
         \x20           case captured():\n\
         \x20               pass\n\
         \x20   captured = C()\n\
         \x20   inner()\n",
    );
    assert_eq!(
        errors
            .iter()
            .filter(|error| error.contains("class pattern target must be a class"))
            .count(),
        1,
        "late-bound closures must see a single known local instance assignment: {errors:?}"
    );

    let errors = check(
        "class C:\n\
         \x20   pass\n\
         def outer() -> None:\n\
         \x20   def inner() -> None:\n\
         \x20       match C():\n\
         \x20           case Alias():\n\
         \x20               pass\n\
         \x20   Alias = C\n\
         \x20   inner()\n",
    );
    assert!(
        errors.is_empty(),
        "late-bound class-object aliases must remain valid pattern heads: {errors:?}"
    );

    let errors = check(
        "import external\n\
         C = 1\n\
         match 1:\n\
         \x20   case external.C():\n\
         \x20       pass\n",
    );
    assert!(
        errors.is_empty(),
        "a dotted unknown pattern head must not consult an unrelated local tail: {errors:?}"
    );

    let errors = check(
        "Alias = int\n\
         match 1:\n\
         \x20   case Alias(value):\n\
         \x20       keep: int = value\n\
         match set():\n\
         \x20   case set(value):\n\
         \x20       pass\n\
         match bytes():\n\
         \x20   case bytes(value):\n\
         \x20       pass\n",
    );
    assert!(
        errors.is_empty(),
        "primitive builtin classes and their aliases must remain valid pattern heads: {errors:?}"
    );

    let errors = check(
        "def ordinary() -> None:\n\
         \x20   pass\n\
         match 1:\n\
         \x20   case ordinary():\n\
         \x20       pass\n",
    );
    assert_eq!(
        errors
            .iter()
            .filter(|error| error.contains("class pattern target must be a class"))
            .count(),
        1,
        "a known ordinary function must not become a class-pattern head: {errors:?}"
    );
}

#[test]
fn test_match_class_pattern_closure_seed_disqualifies_other_rebindings() {
    for source in [
        "class C:\n\
         \x20   pass\n\
         def outer() -> None:\n\
         \x20   def inner() -> None:\n\
         \x20       match C():\n\
         \x20           case captured():\n\
         \x20               pass\n\
         \x20   captured = C()\n\
         \x20   for captured in [C]:\n\
         \x20       pass\n\
         \x20   inner()\n",
        "class C:\n\
         \x20   pass\n\
         def outer() -> None:\n\
         \x20   def inner() -> None:\n\
         \x20       match C():\n\
         \x20           case captured():\n\
         \x20               pass\n\
         \x20   captured = C()\n\
         \x20   (captured,) = (C,)\n\
         \x20   inner()\n",
        "class C:\n\
         \x20   pass\n\
         def outer() -> None:\n\
         \x20   def inner() -> None:\n\
         \x20       match C():\n\
         \x20           case captured():\n\
         \x20               pass\n\
         \x20   captured = C()\n\
         \x20   match 1:\n\
         \x20       case _ if (captured := C):\n\
         \x20           pass\n\
         \x20   inner()\n",
    ] {
        let errors = check(source);
        assert_eq!(
            errors
                .iter()
                .filter(|error| error.contains("class pattern target must be a class"))
                .count(),
            0,
            "closure seeding must remain conservative across rebinding forms: {errors:?}"
        );
    }
}

#[test]
fn test_match_class_pattern_closure_respects_local_shadowing_forms() {
    for statement in [
        "    del captured\n",
        "    captured: C\n",
        "    match C:\n        case captured:\n            pass\n",
    ] {
        let source = format!(
            "class C:\n\
             \x20   pass\n\
             captured = C()\n\
             def outer() -> None:\n\
             \x20   def inner() -> None:\n\
             \x20       match C():\n\
             \x20           case captured():\n\
             \x20               pass\n\
             {statement}\
             \x20   inner()\n"
        );
        let errors = check(&source);
        assert_eq!(
            errors
                .iter()
                .filter(|error| error.contains("class pattern target must be a class"))
                .count(),
            0,
            "function-local binding forms must shadow outer instances: {errors:?}"
        );
    }
}

#[test]
fn test_match_class_pattern_closure_alias_seed_requires_one_binding_event() {
    for (alias, rebinding) in [
        ("C", "for Alias in [D()]:\n        pass"),
        ("C", "match D():\n        case Alias:\n            pass"),
        ("int", "for Alias in [D()]:\n        pass"),
    ] {
        let source = format!(
            "class C:\n\
             \x20   x: int = 1\n\
             class D:\n\
             \x20   pass\n\
             def outer() -> None:\n\
             \x20   def inner() -> None:\n\
             \x20       match C():\n\
             \x20           case Alias(x=value):\n\
             \x20               keep: str = value\n\
             \x20   Alias = {alias}\n\
             \x20   {rebinding}\n\
             \x20   inner()\n"
        );
        let errors = check(&source);
        assert!(
            errors.is_empty(),
            "competing bindings must discard preregistered class metadata: {errors:?}"
        );
    }
}

#[test]
fn test_preregister_binding_counts_preserve_sequential_inference() {
    for source in [
        "x = 1\nx = 2\nkeep: str = x\n",
        "def local() -> None:\n    x = 1\n    x = 2\n    keep: str = x\n",
    ] {
        let errors = check(source);
        assert_eq!(
            errors
                .iter()
                .filter(|error| error.contains("type mismatch: expected `str`, got `int`"))
                .count(),
            1,
            "binding-count guards must not predeclare ordinary variables as Any: {errors:?}"
        );
    }
}

#[test]
fn test_class_alias_preregistration_does_not_overwrite_parameters() {
    let errors = check(
        "class C:\n\
         \x20   pass\n\
         class D:\n\
         \x20   pass\n\
         def outer(Alias: D) -> None:\n\
         \x20   def inner() -> None:\n\
         \x20       match C():\n\
         \x20           case Alias():\n\
         \x20               pass\n\
         \x20   inner()\n\
         \x20   Alias = C\n",
    );
    assert_eq!(
        errors
            .iter()
            .filter(|error| error.contains("class pattern target must be a class"))
            .count(),
        1,
        "a later class alias must not overwrite the parameter's initial instance type: {errors:?}"
    );
}

#[test]
fn test_class_alias_preregistration_respects_global_and_nonlocal() {
    for source in [
        "class C:\n\
         \x20   pass\n\
         class D:\n\
         \x20   pass\n\
         Alias = D()\n\
         def outer() -> None:\n\
         \x20   global Alias\n\
         \x20   def inner() -> None:\n\
         \x20       match C():\n\
         \x20           case Alias():\n\
         \x20               pass\n\
         \x20   inner()\n\
         \x20   Alias = C\n",
        "class C:\n\
         \x20   pass\n\
         class D:\n\
         \x20   pass\n\
         def enclosing() -> None:\n\
         \x20   Alias = D()\n\
         \x20   def outer() -> None:\n\
         \x20       nonlocal Alias\n\
         \x20       def inner() -> None:\n\
         \x20           match C():\n\
         \x20               case Alias():\n\
         \x20                   pass\n\
         \x20       inner()\n\
         \x20       Alias = C\n\
         \x20   outer()\n",
    ] {
        let errors = check(source);
        assert_eq!(
            errors
                .iter()
                .filter(|error| error.contains("class pattern target must be a class"))
                .count(),
            1,
            "global/nonlocal declarations must preserve the outer instance type: {errors:?}"
        );
    }
}

#[test]
fn global_and_nonlocal_resolve_only_legal_enclosing_scopes() {
    let errors = check(
        "class C:\n\
         \x20   pass\n\
         Alias = C()\n\
         class Holder:\n\
         \x20   Alias = C\n\
         \x20   def method(self) -> None:\n\
         \x20       global Alias\n\
         \x20       match C():\n\
         \x20           case Alias():\n\
         \x20               pass\n",
    );
    assert_eq!(
        errors
            .iter()
            .filter(|error| error.contains("class pattern target must be a class"))
            .count(),
        1,
        "global must resolve the module binding, not a class binding: {errors:?}"
    );

    let errors = check(
        "class C:\n\
         \x20   pass\n\
         def outer() -> None:\n\
         \x20   Alias = C()\n\
         \x20   class Holder:\n\
         \x20       Alias = C\n\
         \x20       def method(self) -> None:\n\
         \x20           nonlocal Alias\n\
         \x20           match C():\n\
         \x20               case Alias():\n\
         \x20                   pass\n",
    );
    assert_eq!(
        errors
            .iter()
            .filter(|error| error.contains("class pattern target must be a class"))
            .count(),
        1,
        "nonlocal must skip class scopes and use the enclosing function: {errors:?}"
    );

    let errors = check("Alias = 1\ndef invalid() -> None:\n    nonlocal Alias\n");
    assert!(
        errors
            .iter()
            .any(|error| error.contains("no binding for nonlocal `Alias` found")),
        "module bindings cannot satisfy nonlocal: {errors:?}"
    );
}

#[test]
fn function_body_global_analysis_does_not_mutate_module_state() {
    let errors = check(
        "class C:\n\
         \x20   pass\n\
         Alias = C()\n\
         def mutate() -> None:\n\
         \x20   global Alias\n\
         \x20   Alias = C\n\
         match C():\n\
         \x20   case Alias():\n\
         \x20       pass\n",
    );
    assert_eq!(
        errors
            .iter()
            .filter(|error| error.contains("class pattern target must be a class"))
            .count(),
        1,
        "checking an uncalled function must not change the module binding: {errors:?}"
    );

    let errors = check(
        "from os import strerror\n\
         def mutate() -> None:\n\
         \x20   global strerror\n\
         \x20   strerror = lambda value: value\n\
         \x20   strerror(\"inside\")\n\
         strerror(\"outside\")\n",
    );
    assert_eq!(
        errors
            .iter()
            .filter(|error| error.contains("argument type mismatch"))
            .count(),
        1,
        "global writes must affect body flow without erasing module provenance: {errors:?}"
    );
}

#[test]
fn method_free_names_skip_the_enclosing_class_namespace() {
    let errors = check(
        "from os import strerror\n\
         class Holder:\n\
         \x20   strerror = lambda value: value\n\
         \x20   def method(self) -> None:\n\
         \x20       strerror(\"bad\")\n",
    );
    assert_eq!(
        errors
            .iter()
            .filter(|error| error.contains("argument type mismatch"))
            .count(),
        1,
        "a method free name must resolve at module scope, not class scope: {errors:?}"
    );

    let errors = check(
        "class C:\n\
         \x20   pass\n\
         def outer() -> None:\n\
         \x20   Alias = C()\n\
         \x20   class Holder:\n\
         \x20       Alias = C\n\
         \x20       def method(self) -> None:\n\
         \x20           match C():\n\
         \x20               case Alias():\n\
         \x20                   pass\n",
    );
    assert_eq!(
        errors
            .iter()
            .filter(|error| error.contains("class pattern target must be a class"))
            .count(),
        1,
        "a method in a nested class must close over the outer function: {errors:?}"
    );
}

#[test]
fn recursive_calls_see_the_reasserted_function_declaration() {
    let errors = check(
        "class C:\n\
         \x20   pass\n\
         recursive = C\n\
         def recursive(value: int) -> int:\n\
         \x20   return recursive(\"bad\")\n",
    );
    assert!(
        errors.iter().any(|error| error.contains("argument type mismatch")),
        "recursive calls must see the stable def signature: {errors:?}"
    );
}

#[test]
fn repeated_function_declarations_keep_occurrence_signatures() {
    let errors = check(
        "def convert(value: int) -> int:\n\
         \x20   return value\n\
         convert(\"bad\")\n\
         def convert(value: str) -> str:\n\
         \x20   return value\n\
         convert(1)\n",
    );
    assert_eq!(
        errors
            .iter()
            .filter(|error| error.contains("argument type mismatch"))
            .count(),
        2,
        "each call must use the declaration active at that source position: {errors:?}"
    );
}

#[test]
fn repeated_function_bodies_keep_occurrence_return_types() {
    let errors = check(
        "def convert(value: int) -> int:\n\
         \x20   return \"bad\"\n\
         def convert(value: str) -> str:\n\
         \x20   return 1\n",
    );
    assert_eq!(
        errors
            .iter()
            .filter(|error| error.contains("return type mismatch"))
            .count(),
        2,
        "each body must use its own declaration signature: {errors:?}"
    );
}

#[test]
fn repeated_classes_keep_nominal_identity_and_clear_active_methods() {
    let errors = check(
        "class C:\n\
         \x20   def __neg__(self) -> int:\n\
         \x20       return 1\n\
         Old = C\n\
         class C:\n\
         \x20   pass\n\
         current: C = Old()\n\
         old_neg: int = -Old()\n\
         -C()\n",
    );
    assert_eq!(
        errors
            .iter()
            .filter(|error| error.contains("type mismatch"))
            .count(),
        1,
        "an alias of the first class must remain nominally distinct: {errors:?}"
    );
    assert_eq!(
        errors
            .iter()
            .filter(|error| error.contains("unary `-` requires numeric type"))
            .count(),
        1,
        "the second bare class must not inherit the first class's methods: {errors:?}"
    );
}

#[test]
fn repeated_classes_preserve_numeric_and_index_traits_by_symbol() {
    let errors = check(
        "class Numeric(int):\n\
         \x20   pass\n\
         OldNumeric = Numeric\n\
         class Numeric:\n\
         \x20   pass\n\
         -OldNumeric(1)\n\
         -Numeric()\n\
         class Indexed:\n\
         \x20   def __index__(this) -> int:\n\
         \x20       return 1\n\
         OldIndexed = Indexed\n\
         class Indexed:\n\
         \x20   def marker(self) -> None:\n\
         \x20       pass\n\
         hex(OldIndexed())\n\
         hex(Indexed())\n",
    );
    assert_eq!(
        errors
            .iter()
            .filter(|error| error.contains("unary `-` requires numeric type"))
            .count(),
        1,
        "only the active bare Numeric class should fail: {errors:?}"
    );
    assert_eq!(
        errors
            .iter()
            .filter(|error| error.contains("argument type mismatch"))
            .count(),
        1,
        "only the active bare Indexed class should lack __index__: {errors:?}"
    );
}

#[test]
fn repeated_classes_preserve_typed_dict_and_protocol_traits_by_symbol() {
    let errors = check(
        "from typing import Protocol, TypedDict\n\
         class Row(TypedDict):\n\
         \x20   value: int\n\
         OldRow = Row\n\
         class Row:\n\
         \x20   pass\n\
         old_row: OldRow = {\"value\": 1}\n\
         current_row: Row = {\"value\": 1}\n\
         class Runnable(Protocol):\n\
         \x20   def run(self) -> int:\n\
         \x20       pass\n\
         OldRunnable = Runnable\n\
         class Runnable:\n\
         \x20   pass\n\
         class Impl:\n\
         \x20   def run(self) -> int:\n\
         \x20       return 1\n\
         old_protocol: OldRunnable = Impl()\n\
         current_protocol: Runnable = Impl()\n",
    );
    assert_eq!(
        errors
            .iter()
            .filter(|error| error.contains("type mismatch"))
            .count(),
        2,
        "only the active non-TypedDict/non-Protocol classes should reject structural values: {errors:?}"
    );
}

#[test]
fn same_named_class_base_resolves_before_the_new_binding() {
    let errors = check(
        "class C:\n\
         \x20   def __neg__(self) -> int:\n\
         \x20       return 1\n\
         class C(C):\n\
         \x20   pass\n\
         result: int = -C()\n",
    );
    assert!(
        errors.is_empty(),
        "class C(C) must inherit from the prior C declaration, not itself: {errors:?}"
    );
}

#[test]
fn test_class_alias_preregistration_preserves_builtin_until_assignment() {
    let errors = check(
        "class C:\n\
         \x20   def __init__(self: C, value: int) -> None:\n\
         \x20       pass\n\
         keep: int = int(\"1\")\n\
         int = C\n",
    );
    assert!(
        errors.is_empty(),
        "a later builtin shadow must not alter earlier builtin calls: {errors:?}"
    );
}

#[test]
fn test_declarations_reassert_identity_after_earlier_alias_assignment() {
    let errors = check(
        "class C:\n\
         \x20   pass\n\
         def outer() -> None:\n\
         \x20   Alias = C\n\
         \x20   def Alias() -> None:\n\
         \x20       pass\n\
         \x20   def inner() -> None:\n\
         \x20       match C():\n\
         \x20           case Alias():\n\
         \x20               pass\n\
         \x20   inner()\n",
    );
    assert_eq!(
        errors
            .iter()
            .filter(|error| error.contains("class pattern target must be a class"))
            .count(),
        1,
        "a later function declaration must replace an earlier class alias: {errors:?}"
    );

    let errors = check(
        "Alias = int\n\
         def Alias() -> None:\n\
         \x20   pass\n\
         match 1:\n\
         \x20   case Alias(value):\n\
         \x20       pass\n",
    );
    assert_eq!(
        errors
            .iter()
            .filter(|error| error.contains("class pattern target must be a class"))
            .count(),
        1,
        "a declaration must clear primitive builtin alias provenance: {errors:?}"
    );

    let errors = check(
        "class C:\n\
         \x20   pass\n\
         def accept(value: C) -> None:\n\
         \x20   pass\n\
         Alias = C\n\
         class Alias:\n\
         \x20   pass\n\
         accept(Alias())\n",
    );
    assert!(
        !errors.is_empty(),
        "a later class declaration must restore its own nominal identity"
    );

    let errors = check(
        "class C:\n\
         \x20   pass\n\
         Alias = C\n\
         enum Alias:\n\
         \x20   One\n\
         match C():\n\
         \x20   case Alias():\n\
         \x20       pass\n",
    );
    assert_eq!(
        errors
            .iter()
            .filter(|error| error.contains("class pattern target must be a class"))
            .count(),
        1,
        "a later enum declaration must replace an earlier class alias: {errors:?}"
    );
}

#[test]
fn test_alias_metadata_is_cleared_by_later_mutations() {
    for mutation in [
        "(Alias,) = (D(),)",
        "del Alias",
        "Alias += 1",
        "type Alias = C",
    ] {
        let source = format!(
            "class C:\n\
             \x20   x: int = 1\n\
             class D:\n\
             \x20   pass\n\
             Alias = C\n\
             {mutation}\n\
             def inner() -> None:\n\
             \x20   match C():\n\
             \x20       case Alias(x=value):\n\
             \x20           keep: str = value\n"
        );
        let errors = check(&source);
        assert_eq!(
            errors
                .iter()
                .filter(|error| error.contains("type mismatch: expected `str`, got `int`"))
                .count(),
            0,
            "later mutation must discard stale class metadata: {errors:?}"
        );
    }
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

#[test]
fn test_set_literal_binops_and_comparisons_typecheck() {
    let errors = check(
        "s = {1, 2}\n\
         a = s - {1}\n\
         b = {1, 2} <= {1, 2, 3}\n\
         c = {1, 2} | {3}\n\
         d = {1, 2} & {2, 3}\n\
         e = {1, 2} ^ {2, 3}\n\
         f = {1, 2} < {1, 2, 3}\n\
         g = {1, 2, 3} >= {1, 2}\n\
         h = {1, 2, 3} > {1, 2}\n",
    );
    assert!(
        errors.is_empty(),
        "set literal binops/comparisons should type-check: {errors:?}"
    );
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
        has_parameter_error(&errors, "object"),
        "iter(_W()) should reject a bare object operand, got: {errors:?}"
    );

    let errors = check("class _W:\n    pass\niter(_W(), None)\n");
    assert!(
        has_parameter_error(&errors, "object"),
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
        .filter(|e| e.contains("got `int` for parameter `value`"))
        .count();
    assert_eq!(
        list_value_errors, 5,
        "list value dunders should reject concrete non-list operands, got: {errors:?}"
    );

    let errors = check(
        "class _W:\n    pass\nobj = []\nobj.__getitem__(_W())\nobj.__delitem__(_W())\nobj.__setitem__(_W(), None)\n",
    );
    let bare_errors = bare_instance_parameter_error_count(&errors);
    assert_eq!(
        bare_errors, 3,
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
        .filter(|e| e.contains("got `int` for parameter `value`"))
        .count();
    assert_eq!(
        tuple_value_errors, 5,
        "tuple value dunders should reject concrete non-tuple operands, got: {errors:?}"
    );

    let errors = check("class _W:\n    pass\nobj = ()\nobj.__getitem__(_W())\n");
    let key_errors = parameter_error_count(&errors, "key");
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
        has_parameter_error(&errors, "subclass"),
        "type.__subclasscheck__ should reject a bare instance subclass, got: {errors:?}"
    );

    let errors =
        check("class _W:\n    pass\nfrom builtins import zip\nobj = object.__new__(zip)\nobj.__new__(_W())\n");
    assert!(
        has_parameter_error(&errors, "iter1"),
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
    assert!(
        errors.is_empty(),
        "slice payloads are unconstrained and must accept user instances: {errors:?}"
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
    let typed_errors = bare_instance_parameter_error_count(&errors);
    assert_eq!(
        typed_errors, 8,
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
    let typed_errors = bare_instance_parameter_error_count(&errors);
    assert_eq!(
        typed_errors, 10,
        "str text-method protocol walls should reject bare instances, got: {errors:?}"
    );
    let scalar_errors = errors.len().saturating_sub(typed_errors);
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
        has_parameter_error(&errors, "func"),
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
    let typed_errors = bare_instance_parameter_error_count(&errors);
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
    let typed_errors = bare_instance_parameter_error_count(&errors);
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
    let typed_errors = bare_instance_parameter_error_count(&errors);
    assert_eq!(
        typed_errors, 5,
        "property callable and owner slots should reject incompatible instances, got: {errors:?}"
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
        has_parameter_error(&errors, "subclass"),
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
        has_parameter_error(&errors, "sequence"),
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

// Strict stdlib argument enforcement.

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
fn test_stdlib_from_import_alias_uses_original_member_contract() {
    let errors = check("from os import strerror as err\nerr(\"x\")\n");
    assert!(
        errors.iter().any(|error| error.contains("argument type mismatch")),
        "a function import alias must retain the imported member: {errors:?}"
    );

    let errors = check(
        "class _W:\n\
         \x20   pass\n\
         from fileinput import FileInput as Input\n\
         Input(_W())\n",
    );
    assert!(
        errors.iter().any(|error| error.contains("argument type mismatch")),
        "a constructor import alias must retain the imported member: {errors:?}"
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
fn test_stdlib_path_or_fd_wall_keeps_valid_overloads_clean() {
    let errors = check("from os import listdir\nlistdir('.')\nlistdir(0)\n");
    assert!(
        errors.is_empty(),
        "listdir(str/int) overloads must stay clean, got: {errors:?}"
    );

    let errors = check("from os import listdir\nlistdir(3.14)\n");
    assert!(
        errors.iter().any(|e| e.contains("argument type mismatch")),
        "listdir(float) should be rejected, got: {errors:?}"
    );

    let errors = check("import os\nos.scandir(1.234)\n");
    assert!(
        !errors.is_empty(),
        "scandir(float) should be rejected through constrained GenericPath: {errors:?}"
    );

    let errors = check("from os import listdir\nclass _W: pass\nlistdir(_W())\n");
    assert!(
        errors.iter().any(|e| e.contains("argument type mismatch")),
        "listdir(bare object) should be rejected, got: {errors:?}"
    );
}

#[test]
fn test_stdlib_fileinput_files_typed_wall_rejected() {
    let errors = check("class _W:\n    pass\nfrom fileinput import input\ninput(_W())\n");
    assert!(
        has_parameter_error(&errors, "files"),
        "fileinput.input(_W()) should reject the files arg, got: {errors:?}"
    );

    let errors = check("class _W:\n    pass\nfrom fileinput import FileInput\nFileInput(_W())\n");
    assert!(
        has_parameter_error(&errors, "files"),
        "fileinput.FileInput(_W()) should reject the files arg, got: {errors:?}"
    );

    let errors = check(
        "from fileinput import input, FileInput\ninput([\"a.txt\"])\nFileInput([\"a.txt\"])\n",
    );
    assert!(
        errors.is_empty(),
        "fileinput files iterables should stay accepted, got: {errors:?}"
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
fn test_stdlib_module_and_nested_callable_aliases_retain_contract() {
    let errors = check(
        "import os\n\
         alias = os.strerror\n\
         alias(\"x\")\n",
    );
    assert!(
        errors.iter().any(|error| error.contains("argument type mismatch")),
        "an attribute-derived callable alias must retain its contract: {errors:?}"
    );

    let errors = check(
        "import multiprocessing.reduction\n\
         multiprocessing.reduction.duplicate(\"bad\")\n\
         duplicate = multiprocessing.reduction.duplicate\n\
         duplicate(\"bad\")\n",
    );
    assert!(
        errors
            .iter()
            .filter(|error| error.contains("argument type mismatch"))
            .count()
            >= 2,
        "loaded nested modules and their aliases must retain contracts: {errors:?}"
    );

    let errors = check(
        "import os\n\
         alias = os.strerror\n\
         alias = lambda value: value\n\
         alias(\"x\")\n",
    );
    assert!(
        errors.is_empty(),
        "rebinding must clear an external callable identity: {errors:?}"
    );
}

#[test]
fn test_stdlib_generated_result_types_propagate_through_chains() {
    let errors = check(
        "from pathlib import Path, PurePath\n\
         p: Path = Path(\"x\")\n\
         text: str = Path(\"x\").as_posix()\n\
         resolved: Path = Path(\"x\").resolve(strict=True)\n\
         current: Path = Path.cwd()\n\
         name: str = Path(\"x\").name\n\
         rebound_self: Path = PurePath.with_name(Path(\"x\"), \"y\")\n",
    );
    assert!(
        errors.is_empty(),
        "constructor, inherited method, Self, classmethod, and property results must propagate: {errors:?}"
    );

    let errors = check(
        "from pathlib import Path\n\
         bad_ctor: int = Path(\"x\")\n\
         bad_method: int = Path(\"x\").as_posix()\n\
         bad_classmethod: int = Path.cwd()\n\
         bad_property: int = Path(\"x\").name\n\
         Path(\"x\").resolve(strict=\"bad\")\n",
    );
    assert!(
        errors
            .iter()
            .filter(|error| error.contains("type mismatch"))
            .count()
            >= 4,
        "generated result identities must reject incompatible annotations: {errors:?}"
    );
    assert!(
        has_parameter_error(&errors, "strict"),
        "a chained generated method must enforce its keyword contract: {errors:?}"
    );
}

#[test]
fn test_stdlib_generated_property_setter_enforces_value_type() {
    let errors = check(
        "from urllib.request import Request\n\
         request = Request(\"https://example.com\")\n\
         request.full_url = \"https://example.org\"\n",
    );
    assert!(
        errors.is_empty(),
        "a valid generated property setter value must remain accepted: {errors:?}"
    );

    let errors = check(
        "from urllib.request import Request\n\
         request = Request(\"https://example.com\")\n\
         request.full_url = 42\n",
    );
    assert!(
        errors
            .iter()
            .any(|error| error.contains("expected `str`, got `int`")),
        "a generated property setter must reject an incompatible value: {errors:?}"
    );
}

#[test]
fn test_stdlib_external_values_preserve_canonical_and_runtime_nominal_identity() {
    let errors = check(
        "from io import StringIO\n\
         from types import ModuleType\n\
         import io\n\
         import os\n\
         direct: StringIO = StringIO()\n\
         dotted: io.StringIO = io.StringIO()\n\
         module: ModuleType = os\n",
    );
    assert!(
        errors.is_empty(),
        "re-exported classes and external runtime values must retain compatible nominal identities: {errors:?}"
    );

    let errors = check(
        "from types import BuiltinFunctionType, BuiltinMethodType, FunctionType\n\
         from pathlib import Path\n\
         import inspect\n\
         import os\n\
         wrong_python_kind: FunctionType = os.strerror\n\
         unknown_function_kind: BuiltinFunctionType = inspect.signature\n\
         unknown_method_kind: BuiltinMethodType = Path.cwd\n",
    );
    assert!(
        errors
            .iter()
            .filter(|error| error.contains("type mismatch"))
            .count()
            >= 3,
        "an unproven external callable runtime kind must not satisfy a concrete nominal class: {errors:?}"
    );
}

#[test]
fn test_stdlib_constructor_prefers_direct_contract_before_inherited_fallback() {
    let errors = check("from ast import Bytes\nBytes(s=b\"x\")\n");
    assert!(
        errors.is_empty(),
        "a direct __new__ contract must win over an inherited broad constructor: {errors:?}"
    );

    let errors = check("from pathlib import Path\nPath(123)\n");
    assert!(
        errors
            .iter()
            .any(|error| error.contains("argument type mismatch")),
        "Path's direct __new__ StrPath contract must reject int: {errors:?}"
    );
}

#[test]
fn test_stdlib_inherited_generic_members_project_receiver_type_arguments() {
    let errors = check(
        "from queue import LifoQueue, PriorityQueue, Queue\n\
         queue: Queue[int] = Queue()\n\
         queue.put(1)\n\
         direct: int = queue.get()\n\
         lifo: LifoQueue[int] = LifoQueue()\n\
         lifo.put(1)\n\
         inherited: int = lifo.get()\n\
         priority: PriorityQueue[int] = PriorityQueue()\n\
         priority.put(1)\n\
         projected: int = priority.get()\n",
    );
    assert!(
        errors.is_empty(),
        "direct and inherited generic members must preserve receiver type arguments: {errors:?}"
    );

    let errors = check(
        "import queue\n\
         value: queue.Queue[int] = queue.Queue()\n\
         value.put(1)\n\
         result: int = value.get()\n",
    );
    assert!(
        errors.is_empty(),
        "dotted external generic annotations must retain their arguments: {errors:?}"
    );
    let errors = check(
        "import queue\n\
         value: queue.Queue[int] = queue.Queue()\n\
         value.put(\"bad\")\n",
    );
    assert!(
        errors.iter().any(|error| error.contains("type mismatch")),
        "dotted external generic annotations must enforce their arguments: {errors:?}"
    );
    let errors = check(
        "import concurrent.futures._base\n\
         future: concurrent.futures._base.Future[int] = concurrent.futures._base.Future()\n\
         future.set_result(1)\n\
         result: int = future.result()\n",
    );
    assert!(
        errors.is_empty(),
        "deep loaded-module prefixes must retain generic annotations: {errors:?}"
    );
    let errors = check(
        "import concurrent.futures._base\n\
         future: concurrent.futures._base.Future[int] = concurrent.futures._base.Future()\n\
         future.set_result(\"bad\")\n",
    );
    assert!(
        errors.iter().any(|error| error.contains("type mismatch")),
        "deep loaded-module prefixes must enforce generic annotations: {errors:?}"
    );

    for (label, source) in [
        (
            "direct parameter",
            "from queue import Queue\nqueue: Queue[int] = Queue()\nqueue.put(\"bad\")\n",
        ),
        (
            "direct return",
            "from queue import Queue\nqueue: Queue[int] = Queue()\nbad: str = queue.get()\n",
        ),
        (
            "inherited parameter",
            "from queue import PriorityQueue\nqueue: PriorityQueue[int] = PriorityQueue()\nqueue.put(\"bad\")\n",
        ),
        (
            "inherited return",
            "from queue import PriorityQueue\nqueue: PriorityQueue[int] = PriorityQueue()\nbad: str = queue.get()\n",
        ),
    ] {
        let errors = check(source);
        assert!(
            errors.iter().any(|error| error.contains("type mismatch")),
            "{label} must use the concrete receiver argument: {errors:?}"
        );
    }

    let errors = check(
        "from queue import PriorityQueue\n\
         class _W:\n\
         \x20   pass\n\
         invalid: PriorityQueue[_W] = PriorityQueue()\n",
    );
    assert!(
        errors
            .iter()
            .any(|error| error.contains("bound violation")),
        "a clearly incompatible external generic bound must still be rejected: {errors:?}"
    );
}

#[test]
fn test_stdlib_generic_property_getter_projects_receiver_type_arguments() {
    let errors = check(
        "value: slice[int, int, int] = slice(1, 2, 3)\n\
         start: int = value.start\n\
         stop: int = value.stop\n\
         step: int = value.step\n",
    );
    assert!(
        errors.is_empty(),
        "generic property getters must preserve receiver type arguments: {errors:?}"
    );

    let errors = check("value: slice[int, int, int] = slice(1, 2, 3)\nbad: str = value.start\n");
    assert!(
        errors.iter().any(|error| error.contains("type mismatch")),
        "a generic property result must reject an incompatible annotation: {errors:?}"
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
fn test_stdlib_unbound_method_explicit_receiver_keeps_param_alignment() {
    let errors = check(
        "class _W:\n    pass\nfrom importlib.metadata import DistributionFinder\nDistributionFinder.find_distributions(object(), _W())\n",
    );
    assert!(
        errors.iter().any(|e| e.contains("argument type mismatch")),
        "DistributionFinder.find_distributions(object(), _W()) should reject the context arg, got: {errors:?}"
    );

    let errors = check(
        "class _W:\n    pass\nfrom importlib.metadata import MetadataPathFinder\nMetadataPathFinder.find_distributions(_W())\n",
    );
    assert!(
        errors.iter().any(|e| e.contains("argument type mismatch")),
        "MetadataPathFinder.find_distributions(_W()) should still reject the first arg, got: {errors:?}"
    );

    let errors = check("from datetime import date\ndate.fromtimestamp(\"not_a_float\")\n");
    assert!(
        errors.iter().any(|e| e.contains("argument type mismatch")),
        "date.fromtimestamp(str) should not be treated as an unbound receiver call, got: {errors:?}"
    );
}

#[test]
fn test_stdlib_constructor_wrong_scalar_rejected() {
    let errors = check("from builtins import SyntaxError\nSyntaxError(12345, None)\n");
    assert!(
        errors.iter().any(|e| e.contains("argument type mismatch")),
        "SyntaxError(non_str_msg, details) should be rejected by the strict type wall, got: {errors:?}"
    );
    let errors = check(
        "from builtins import SyntaxError\nSyntaxError(\"bad\", (\"file.py\", 1, 1, \"line\"))\n",
    );
    assert!(
        errors.is_empty(),
        "SyntaxError(str_msg, valid_details) must be clean at type-check time, got: {errors:?}"
    );
    let errors = check("from builtins import SyntaxError\nSyntaxError(\"bad\", None)\n");
    assert!(
        has_parameter_error(&errors, "info"),
        "SyntaxError details must follow CPython's tuple contract, got: {errors:?}"
    );
}

#[test]
fn test_stdlib_exception_group_typed_method_rejects_bare_instance() {
    let errors = check(
        "from builtins import ExceptionGroup\nclass _W:\n    pass\nobj = ExceptionGroup(\"msg\", [ValueError(\"x\")])\nobj.split(_W())\n",
    );
    assert!(
        has_parameter_error(&errors, "matcher_value"),
        "ExceptionGroup.split(_W()) should reject a bare instance, got: {errors:?}"
    );

    let errors = check(
        "from builtins import BaseExceptionGroup\nclass _W:\n    pass\nobj = BaseExceptionGroup(\"msg\", [ValueError(\"x\")])\nobj.derive(_W())\n",
    );
    assert!(
        has_parameter_error(&errors, "excs"),
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
        has_parameter_error(&errors, "async_iterable"),
        "direct builtin aiter(_W()) should reject a bare instance, got: {errors:?}"
    );

    let errors = check("class _W:\n    pass\nanext(_W(), None)\n");
    assert!(
        has_parameter_error(&errors, "i"),
        "direct builtin anext(_W(), None) should reject a bare instance, got: {errors:?}"
    );

    let errors = check("class _W:\n    pass\ndef aiter(value):\n    return value\naiter(_W())\n");
    assert!(
        errors.is_empty(),
        "user-shadowed aiter must not use the stdlib signature, got: {errors:?}"
    );
}

#[test]
fn generated_builtin_binder_owns_positional_only_calls() {
    for (name, source) in [
        ("chr", "chr(i=65)\n"),
        ("ord", "ord(c=\"a\")\n"),
        (
            "getattr",
            "getattr(object=object(), name=\"missing\", default=None)\n",
        ),
        ("hasattr", "hasattr(obj=object(), name=\"missing\")\n"),
        (
            "setattr",
            "setattr(obj=object(), name=\"value\", value=1)\n",
        ),
        ("format", "format(value=1, format_spec=\"\")\n"),
        (
            "isinstance",
            "isinstance(obj=1, class_or_tuple=int)\n",
        ),
        (
            "issubclass",
            "issubclass(cls=int, class_or_tuple=object)\n",
        ),
    ] {
        let errors = check(source);
        assert_eq!(
            errors.len(),
            1,
            "generated builtins.{name} must be the sole keyword-binding authority: {errors:?}"
        );
    }

    let errors = check("def chr(i):\n    return i\nchr(i=65)\n");
    assert!(
        errors.is_empty(),
        "a user-shadowed builtin name must retain ordinary keyword binding: {errors:?}"
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
        parameter_error_count(&errors, "value") >= 12,
        "set operators should reject bare non-AbstractSet operands, got: {errors:?}"
    );

    let errors = check(
        "from builtins import set\nclass SetLike:\n    def __contains__(self, item):\n        return False\nobj = set()\nobj.__and__(set())\nobj.__ior__(SetLike())\n",
    );
    assert_eq!(
        parameter_error_count(&errors, "value"),
        1,
        "AbstractSet is nominal; __contains__ alone must not satisfy it: {errors:?}"
    );

    let errors = check("obj = set()\nobj.__and__(set())\nobj.__ior__(frozenset())\n");
    assert!(errors.is_empty(), "real set operands must remain valid: {errors:?}");

    let errors = check("class _W: pass\nobj = set()\nobj.__ior__(_W())\n");
    assert!(
        has_parameter_error(&errors, "value"),
        "a builtin-inferred set receiver must use the generated contract: {errors:?}"
    );
}

#[test]
fn test_stdlib_frozenset_operator_bare_instance_rejected() {
    let errors =
        check("from builtins import frozenset\nclass _W:\n    pass\nobj = frozenset()\nobj.__and__(_W())\nobj.__ge__(_W())\nobj.__gt__(_W())\nobj.__le__(_W())\nobj.__lt__(_W())\nobj.__or__(_W())\nobj.__sub__(_W())\nobj.__xor__(_W())\n");
    assert!(
        parameter_error_count(&errors, "value") >= 8,
        "frozenset operators should reject bare non-AbstractSet operands, got: {errors:?}"
    );

    let errors = check(
        "from builtins import frozenset\nclass SetLike:\n    def __contains__(self, item):\n        return False\nobj = frozenset()\nobj.__and__(frozenset())\nobj.__ge__(SetLike())\n",
    );
    assert_eq!(
        parameter_error_count(&errors, "value"),
        1,
        "AbstractSet is nominal; __contains__ alone must not satisfy it: {errors:?}"
    );

    let errors = check("obj = frozenset()\nobj.__and__(set())\nobj.__ge__(frozenset())\n");
    assert!(
        errors.is_empty(),
        "real set/frozenset operands must remain valid: {errors:?}"
    );
}

#[test]
fn test_stdlib_frozenset_new_iterable_rejected() {
    let errors = check(
        "from builtins import frozenset\nclass _W:\n    pass\nfrozenset.__new__(frozenset, _W())\n",
    );
    assert!(
        has_parameter_error(&errors, "iterable"),
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
            .any(|e| e.contains("argument type mismatch")),
        "bytes(_W()) should reject a bare source instance, got: {errors:?}"
    );

    let errors = check("from builtins import bytes\nbytes(12345, \"\")\n");
    assert!(
        has_parameter_error(&errors, "string"),
        "bytes(int, encoding) should reject the dependent overload mismatch, got: {errors:?}"
    );

    let errors = check("from builtins import bytearray\nbytearray(12345, \"\")\n");
    assert!(
        has_parameter_error(&errors, "string"),
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
        has_parameter_error(&errors, "real"),
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
        has_parameter_error(&errors, "ndigits"),
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
fn generated_dict_operator_contracts_replace_mapping_wall() {
    let errors = check("obj: dict[str, int] = {}\nobj.__or__(12345)\nobj.__ror__(\"bad\")\n");
    assert_eq!(
        errors.len(),
        2,
        "each invalid dict union operand must produce one generated diagnostic: {errors:?}"
    );
    assert!(
        parameter_error_count(&errors, "value") == 2,
        "generated dict union contracts should reject concrete scalar operands: {errors:?}"
    );

    let errors = check("class _W:\n    pass\nobj: dict[str, int] = {}\nobj.__ior__(_W())\n");
    assert_eq!(
        errors.len(),
        1,
        "an invalid dict update operand must produce one generated diagnostic: {errors:?}"
    );
    assert!(
        has_parameter_error(&errors, "value"),
        "generated dict.__ior__ must reject a bare non-mapping operand: {errors:?}"
    );

    let errors = check(
        "class MappingLike:\n    def keys(self):\n        return []\n    def __getitem__(self, key):\n        return 1\nobj: dict[str, int] = {}\nobj.__or__(MappingLike())\n",
    );
    assert!(
        has_parameter_error(&errors, "value"),
        "dict.__or__ only accepts dict operands, not arbitrary mappings: {errors:?}"
    );

    let errors = check(
        "class MappingLike:\n    def keys(self):\n        return []\n    def __getitem__(self, key):\n        return 1\nobj: dict[str, int] = {}\nobj.__or__({\"a\": 1})\nobj.__ior__(MappingLike())\nobj.__ior__([(\"b\", 2)])\nvalue: Any = 1\nobj.__ror__(value)\n",
    );
    assert!(
        errors.is_empty(),
        "generated dict contracts must accept mapping-like, iterable-pair, and dynamic operands: {errors:?}"
    );
}

#[test]
fn test_stdlib_classmethod_wrong_bare_instance_rejected() {
    let errors = check(
        "from builtins import classmethod\nclass _W:\n    pass\nobj = classmethod(lambda cls: None)\nobj.__get__(_W())\n",
    );
    assert!(
        errors.is_empty(),
        "classmethod.__get__ accepts an unconstrained instance operand: {errors:?}"
    );

    let errors =
        check("from builtins import classmethod\nclass _W:\n    pass\nclassmethod(_W())\n");
    assert!(
        has_parameter_error(&errors, "f"),
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
        errors.is_empty(),
        "staticmethod.__get__ accepts an unconstrained instance operand: {errors:?}"
    );

    let errors =
        check("from builtins import staticmethod\nclass _W:\n    pass\nstaticmethod(_W())\n");
    assert!(
        has_parameter_error(&errors, "f"),
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
        has_parameter_error(&errors, "owner"),
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
        has_parameter_error(&errors, "x"),
        "int.__new__(int, _W()) should reject a bare x operand, got: {errors:?}"
    );

    let errors =
        check("from builtins import int\nclass _W:\n    pass\nint.__new__(int, _W(), None)\n");
    assert!(
        has_parameter_error(&errors, "x"),
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
        has_parameter_error(&errors, "value"),
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
        has_parameter_error(&errors, "function"),
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
        has_parameter_error(&errors, "class_or_tuple"),
        "isinstance(None, _W()) should reject a bare classinfo operand, got: {errors:?}"
    );

    let errors = check("class _W:\n    pass\nisinstance(None, (int, _W()))\n");
    assert!(
        has_parameter_error(&errors, "class_or_tuple"),
        "isinstance(None, (int, _W())) should reject a bare classinfo tuple element, got: {errors:?}"
    );

    let errors = check(
        "class MyType:\n    pass\nisinstance(None, MyType)\nisinstance(None, (MyType, int))\n",
    );
    assert!(
        errors.is_empty(),
        "valid class and tuple classinfo operands must stay accepted, got: {errors:?}"
    );

    let errors = check(
        "isinstance(None, ValueError)\n\
         isinstance(None, ValueError(\"boom\"))\n",
    );
    assert_eq!(
        parameter_error_count(&errors, "class_or_tuple"),
        1,
        "builtin class objects are valid classinfo but their instances are not: {errors:?}"
    );

    let errors = check(
        "class MyType:\n\
         \x20   pass\n\
         ClassAlias = MyType\n\
         value = MyType()\n\
         isinstance(None, ClassAlias)\n\
         isinstance(None, value)\n",
    );
    assert_eq!(
        parameter_error_count(&errors, "class_or_tuple"),
        1,
        "class-object aliases are valid classinfo while instance variables are not: {errors:?}"
    );

    let errors = check(
        "class MyType:\n\
         \x20   pass\n\
         Alias = MyType\n\
         object.__subclasshook__(Alias)\n",
    );
    assert!(
        errors.is_empty(),
        "Typed contracts must not mistake class-object aliases for bare instances: {errors:?}"
    );
}

#[test]
fn test_stdlib_unknown_and_protocol_contracts_are_distinct() {
    // base64.b64encode(s: ReadableBuffer) -> Unknown: NOT enforceable. Even a
    // blatantly wrong int must NOT be rejected.
    let errors = check("from base64 import b64encode\nb64encode(123)\n");
    assert!(
        errors.is_empty(),
        "b64encode(int) must NOT be rejected (ReadableBuffer->Unknown), got: {errors:?}"
    );
    // SupportsIndex is generated structural data, not an Unknown escape hatch.
    let errors = check("from math import factorial\nfactorial(3.0)\n");
    assert!(
        has_parameter_error(&errors, "x"),
        "factorial(float) must be rejected by SupportsIndex, got: {errors:?}"
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

#[test]
fn test_stdlib_provenance_is_keyed_by_binding_identity() {
    let errors = check(
        "from os import strerror\n\
         def local(strerror):\n\
         \x20   strerror(\"ok\")\n\
         strerror(\"ok\")\n",
    );
    assert_eq!(
        errors
            .iter()
            .filter(|error| error.contains("argument type mismatch"))
            .count(),
        1,
        "a parameter shadow must not inherit import provenance: {errors:?}"
    );

    let errors = check(
        "from os import strerror\n\
         strerror = lambda value: value\n\
         strerror(\"ok\")\n",
    );
    assert!(
        errors.is_empty(),
        "a direct assignment must clear import provenance: {errors:?}"
    );

    let errors = check(
        "from html.parser import HTMLParser\n\
         obj = object.__new__(HTMLParser)\n\
         def local(obj):\n\
         \x20   obj.handle_entityref(12345)\n\
         obj.handle_entityref(12345)\n",
    );
    assert_eq!(
        errors
            .iter()
            .filter(|error| error.contains("argument type mismatch"))
            .count(),
        1,
        "a parameter shadow must not inherit instance provenance: {errors:?}"
    );

    let errors = check(
        "import queue\n\
         Alias = queue.Queue\n\
         def local(Alias):\n\
         \x20   inside: int = Alias()\n\
         outside: int = Alias()\n",
    );
    assert_eq!(
        errors
            .iter()
            .filter(|error| error.contains("type mismatch"))
            .count(),
        1,
        "a parameter shadow must not inherit native class-reference provenance: {errors:?}"
    );
}

#[test]
fn test_stdlib_instance_origin_survives_constructor_rebinding() {
    let errors = check(
        "from html.parser import HTMLParser\n\
         obj = object.__new__(HTMLParser)\n\
         HTMLParser = 0\n\
         obj.handle_entityref(12345)\n",
    );
    assert!(
        errors.iter().any(|error| error.contains("argument type mismatch")),
        "an existing instance must retain its immutable class origin: {errors:?}"
    );
}

#[test]
fn test_stdlib_provenance_propagates_through_value_aliases() {
    let errors = check(
        "from os import strerror\n\
         alias = strerror\n\
         alias(\"x\")\n",
    );
    assert!(
        errors.iter().any(|error| error.contains("argument type mismatch")),
        "a function value alias must retain import provenance: {errors:?}"
    );

    let errors = check(
        "from html.parser import HTMLParser\n\
         Parser = HTMLParser\n\
         obj = object.__new__(Parser)\n\
         obj.handle_entityref(12345)\n",
    );
    assert!(
        errors.iter().any(|error| error.contains("argument type mismatch")),
        "a constructor value alias must retain import provenance: {errors:?}"
    );

    let errors = check(
        "from html.parser import HTMLParser\n\
         obj = object.__new__(HTMLParser)\n\
         alias = obj\n\
         alias.handle_entityref(12345)\n",
    );
    assert!(
        errors.iter().any(|error| error.contains("argument type mismatch")),
        "an instance value alias must retain immutable instance provenance: {errors:?}"
    );
}

#[test]
fn conditional_joins_clear_class_and_stdlib_provenance() {
    for source in [
        "import queue\n\
         Alias = lambda: 0\n\
         if True:\n\
         \x20   Alias = lambda: 0\n\
         else:\n\
         \x20   Alias = queue.Queue\n\
         value: int = Alias()\n",
        "import queue\n\
         Alias = lambda: 0\n\
         match 0:\n\
         \x20   case 0:\n\
         \x20       Alias = lambda: 0\n\
         \x20   case _:\n\
         \x20       Alias = queue.Queue\n\
         value: int = Alias()\n",
        "import queue\n\
         Alias = lambda: 0\n\
         if False:\n\
         \x20   (Alias := queue.Queue)\n\
         value: int = Alias()\n",
        "from html.parser import HTMLParser\n\
         obj = object()\n\
         if True:\n\
         \x20   obj = object()\n\
         else:\n\
         \x20   obj = object.__new__(HTMLParser)\n\
         obj.handle_entityref(12345)\n",
        "import os\n\
         alias = os.strerror\n\
         if True:\n\
         \x20   alias = os.strerror\n\
         else:\n\
         \x20   alias = lambda value: value\n\
         alias(\"x\")\n",
        "import os\n\
         alias = os.strerror if True else (lambda value: value)\n\
         alias(\"x\")\n",
    ] {
        let errors = check(source);
        assert!(
            errors.is_empty(),
            "a mutually exclusive path must not leave definite class provenance: {errors:?}"
        );
    }
}

#[test]
fn expression_control_flow_joins_class_identity() {
    for source in [
        "class C:\n\
         \x20   pass\n\
         def choose(cond: bool) -> None:\n\
         \x20   Alias = C if cond else C()\n\
         \x20   Alias()()\n",
        "class C:\n\
         \x20   pass\n\
         Alias = C\n\
         False and (Alias := C())\n\
         Alias()()\n",
        "class C:\n\
         \x20   pass\n\
         Alias = C\n\
         [(Alias := C()) for item in []]\n\
         Alias()()\n",
        "class C:\n\
         \x20   pass\n\
         def choose(cond: bool) -> None:\n\
         \x20   Alias = C\n\
         \x20   value = (Alias := C) if cond else (Alias := C())\n\
         \x20   Alias()()\n",
    ] {
        let errors = check(source);
        assert!(
            errors.is_empty(),
            "a lazy or mutually exclusive expression must not leave a definite class role: {errors:?}"
        );
    }
}

#[test]
fn property_getter_and_setter_keep_distinct_type_contracts() {
    let valid = check(
        "class Box:\n\
         \x20   @property\n\
         \x20   def value(self) -> int:\n\
         \x20       return 1\n\
         \x20   @value.setter\n\
         \x20   def value(self, new: str) -> None:\n\
         \x20       pass\n\
         box = Box()\n\
         read: int = box.value\n\
         box.value = \"ok\"\n",
    );
    assert!(
        valid.is_empty(),
        "getter reads and setter writes must use separate contracts: {valid:?}"
    );

    let invalid = check(
        "class Box:\n\
         \x20   @property\n\
         \x20   def value(self) -> int:\n\
         \x20       return 1\n\
         \x20   @value.setter\n\
         \x20   def value(self, new: str) -> None:\n\
         \x20       pass\n\
         box = Box()\n\
         box.value = 1\n",
    );
    assert_eq!(
        invalid
            .iter()
            .filter(|error| error.contains("type mismatch in assignment"))
            .count(),
        1,
        "setter annotations must reject the wrong assigned value: {invalid:?}"
    );
}

#[test]
fn typeshed_generic_module_function_propagates_return_type() {
    for source in [
        "from copy import copy\nvalue: int = copy(1)\ntext: str = copy(\"x\")\n",
        "import copy\nvalue: int = copy.copy(1)\ntext: str = copy.copy(\"x\")\n",
    ] {
        let errors = check(source);
        assert!(
            errors.is_empty(),
            "copy.copy must preserve its argument TypeVar in the return: {errors:?}"
        );
    }
}

#[test]
fn typeshed_generic_module_function_rejects_wrong_result_annotation() {
    for source in [
        "from copy import copy\nvalue: str = copy(1)\n",
        "import copy\nvalue: str = copy.copy(1)\n",
    ] {
        let errors = check(source);
        assert!(
            errors
                .iter()
                .any(|error| error.contains("expected `str`, got `int`")),
            "copy.copy(1) must infer int rather than Any: {errors:?}"
        );
    }
}

#[test]
fn typeshed_structured_supports_index_checks_protocol_members() {
    let valid = check(
        "from operator import index\n\
         class Good:\n\
         \x20   def __index__(self) -> int:\n\
         \x20       return 1\n\
         class Inherited(Good):\n\
         \x20   pass\n\
         class Numeric(int):\n\
         \x20   pass\n\
         class Root:\n\
         \x20   def __index__(self) -> str:\n\
         \x20       return \"bad\"\n\
         class Left(Root):\n\
         \x20   pass\n\
         class Right(Root):\n\
         \x20   def __index__(self) -> int:\n\
         \x20       return 1\n\
         class Diamond(Left, Right):\n\
         \x20   pass\n\
         value: int = index(Good())\n\
         inherited: int = index(Inherited())\n\
         numeric_subclass: int = index(Numeric(1))\n\
         diamond: int = index(Diamond())\n\
         builtin: int = index(1)\n",
    );
    assert!(
        valid.is_empty(),
        "a complete SupportsIndex implementation and int must be accepted: {valid:?}"
    );

    let wrong_return = check(
        "from operator import index\n\
         class BadReturn:\n\
         \x20   def __index__(that) -> str:\n\
         \x20       return \"bad\"\n\
         index(BadReturn())\n",
    );
    assert!(
        wrong_return
            .iter()
            .any(|error| error.contains("argument type mismatch")),
        "a protocol method with the wrong return type must be rejected: {wrong_return:?}"
    );

    let missing = check(
        "from operator import index\n\
         class Missing:\n\
         \x20   pass\n\
         index(Missing())\n",
    );
    assert!(
        missing
            .iter()
            .any(|error| error.contains("argument type mismatch")),
        "a closed class missing __index__ must be rejected: {missing:?}"
    );

    let gradual = check("from operator import index\nvalue: Any = object()\nindex(value)\n");
    assert!(
        gradual.is_empty(),
        "Any must keep protocol matching indeterminate rather than rejected: {gradual:?}"
    );
}

#[test]
fn generated_builtin_supports_index_contracts_replace_name_gates() {
    let valid = check(
        "class Good:\n\
         \x20   def __index__(self) -> int:\n\
         \x20       return 65\n\
         char: str = chr(Good())\n\
         hexadecimal: str = hex(Good())\n\
         octal: str = oct(Good())\n\
         binary: str = bin(Good())\n",
    );
    assert!(
        valid.is_empty(),
        "generated SupportsIndex contracts must accept a complete implementation: {valid:?}"
    );

    for (label, source) in [
        (
            "wrong return",
            "class Bad:\n    def __index__(self) -> str:\n        return \"bad\"\nchr(Bad())\n",
        ),
        ("missing method", "class Missing:\n    pass\nhex(Missing())\n"),
        (
            "shadowed builtin",
            "class Good:\n    def __index__(self) -> int:\n        return 65\ndef chr(value: int) -> str:\n    return \"x\"\nchr(Good())\n",
        ),
    ] {
        let errors = check(source);
        assert_eq!(
            errors.len(),
            1,
            "{label} must produce one generated diagnostic: {errors:?}"
        );
        assert!(
            errors.iter().any(|error| error.contains("type mismatch")),
            "{label} must be rejected by the active contract: {errors:?}"
        );
    }
}

#[test]
fn typeshed_protocol_keyword_parameter_names_are_enforced() {
    let valid = check(
        "from argparse import ArgumentParser, HelpFormatter\n\
         class Formatter:\n\
         \x20   def __call__(self, *, prog: str) -> HelpFormatter:\n\
         \x20       pass\n\
         ArgumentParser(formatter_class=Formatter())\n",
    );
    assert!(
        valid.is_empty(),
        "matching keyword-only protocol parameters must be accepted: {valid:?}"
    );

    let invalid = check(
        "from argparse import ArgumentParser, HelpFormatter\n\
         class Formatter:\n\
         \x20   def __call__(self, *, wrong: str) -> HelpFormatter:\n\
         \x20       pass\n\
         ArgumentParser(formatter_class=Formatter())\n",
    );
    assert!(
        invalid
            .iter()
            .any(|error| error.contains("argument type mismatch")),
        "a renamed keyword-only protocol parameter must be rejected: {invalid:?}"
    );
}

#[test]
fn typeshed_protocol_defaulted_parameters_are_structural() {
    let valid = check(
        "from _curses import getwin\n\
         class Reader:\n\
         \x20   def read(self, length: int = -1) -> bytes:\n\
         \x20       return b\"\"\n\
         class InheritedReader(Reader):\n\
         \x20   pass\n\
         class VariadicReader:\n\
         \x20   def read(self, *lengths: int) -> bytes:\n\
         \x20       return b\"\"\n\
         getwin(Reader())\n\
         getwin(InheritedReader())\n\
         getwin(VariadicReader())\n",
    );
    assert!(
        valid.is_empty(),
        "a protocol default must accept an implementation that preserves omission: {valid:?}"
    );

    for (label, method) in [
        (
            "required implementation parameter",
            "def read(self, length: int) -> bytes:\n\x20       return b\"\"",
        ),
        (
            "wrong parameter type",
            "def read(self, length: str = \"\") -> bytes:\n\x20       return b\"\"",
        ),
        (
            "wrong return type",
            "def read(self, length: int = -1) -> str:\n\x20       return \"\"",
        ),
    ] {
        let errors = check(&format!(
            "from _curses import getwin\nclass Reader:\n    {method}\ngetwin(Reader())\n"
        ));
        assert!(
            errors
                .iter()
                .any(|error| error.contains("argument type mismatch")),
            "{label} must violate SupportsRead[bytes]: {errors:?}"
        );
    }
}

#[test]
fn typeshed_protocol_class_type_params_are_substituted_in_members() {
    let valid = check(
        "import json\n\
         class Writer:\n\
         \x20   def write(self, value: str) -> object:\n\
         \x20       return None\n\
         json.dump(None, Writer())\n",
    );
    assert!(
        valid.is_empty(),
        "SupportsWrite[str] must accept its class-owned TypeVar substitution: {valid:?}"
    );

    let invalid = check(
        "import json\n\
         class WrongWriter:\n\
         \x20   def write(self, value: int) -> object:\n\
         \x20       return None\n\
         json.dump(None, WrongWriter())\n",
    );
    assert_eq!(
        invalid
            .iter()
            .filter(|error| error.contains("argument type mismatch"))
            .count(),
        1,
        "SupportsWrite[str] must reject a method using the wrong class TypeVar: {invalid:?}"
    );
}

#[test]
fn typeshed_builtin_exception_names_resolve_across_modules() {
    let valid = check(
        "import traceback\n\
         from warnings import warn\n\
         traceback.print_exception(Exception(\"boom\"))\n\
         warn(Warning(\"careful\"))\n",
    );
    assert!(
        valid.is_empty(),
        "builtin exception instances must satisfy cross-module typeshed contracts: {valid:?}"
    );

    let invalid = check(
        "import traceback\n\
         class NotAnException:\n\
         \x20   pass\n\
         traceback.print_exception(NotAnException())\n",
    );
    assert!(
        invalid
            .iter()
            .any(|error| error.contains("argument type mismatch")),
        "a nominal user class must not satisfy BaseException: {invalid:?}"
    );
}

#[test]
fn typeshed_imported_submodule_names_enforce_nominal_types() {
    let valid = check(
        "from asyncio.events import AbstractEventLoop\n\
         from asyncio.subprocess import SubprocessStreamProtocol\n\
         SubprocessStreamProtocol(1, AbstractEventLoop())\n",
    );
    assert!(
        valid.is_empty(),
        "the canonical imported class identity must remain accepted: {valid:?}"
    );

    let invalid = check(
        "from asyncio.subprocess import SubprocessStreamProtocol\n\
         class NotAnEventLoop:\n\
         \x20   pass\n\
         SubprocessStreamProtocol(1, NotAnEventLoop())\n",
    );
    assert!(
        invalid
            .iter()
            .any(|error| error.contains("argument type mismatch")),
        "an unrelated nominal class must not satisfy AbstractEventLoop: {invalid:?}"
    );
}

#[test]
fn typeshed_typing_extensions_never_defaults_materialize() {
    let valid = check("from select import select\nselect([], [], [], 0)\n");
    assert!(
        valid.is_empty(),
        "select's defaulted TypeVars must accept valid iterables: {valid:?}"
    );

    let invalid = check(
        "from select import select\n\
         class NotIterable:\n\
         \x20   pass\n\
         select(NotIterable(), [], [], 0)\n",
    );
    assert!(
        invalid
            .iter()
            .any(|error| error.contains("argument type mismatch")),
        "select's TypeVar bound must reject a non-iterable: {invalid:?}"
    );
}

#[test]
fn typeshed_callable_paramspec_identity_materializes_without_widening() {
    use crate::types::stdlib_typespec as spec;

    let mut checker = TypeChecker::new();
    let run = spec::overloads("_contextvars", "Context", "run")
        .next()
        .expect("Context.run spec");
    let callable = spec::params(run.params)
        .iter()
        .find(|param| spec::string(param.name) == "callable")
        .expect("callable parameter");
    let callable_ty = checker
        .materialize_stdlib_type(spec::type_use(callable.ty).0)
        .expect("Callable[P, T] must materialize");
    let Ty::Fn {
        params,
        variadic,
        param_spec: Some(param_spec),
        ..
    } = checker.tcx.get(callable_ty)
    else {
        panic!("Callable[P, T] must retain its ParamSpec tail")
    };
    assert!(params.is_empty());
    assert!(!variadic);
    assert_eq!(checker.tcx.get_type_var(*param_spec).kind, TypeVarKind::ParamSpec);
    assert!(checker.tcx.contains_type_var(callable_ty));

    let wrapper = spec::overloads("curses", "", "wrapper")
        .next()
        .expect("curses.wrapper spec");
    let func = spec::params(wrapper.params)
        .iter()
        .find(|param| spec::string(param.name) == "func")
        .expect("func parameter");
    let func_ty = checker
        .materialize_stdlib_type(spec::type_use(func.ty).0)
        .expect("Callable[Concatenate[window, P], T] must materialize");
    let Ty::Fn {
        params,
        variadic,
        param_spec: Some(param_spec),
        ..
    } = checker.tcx.get(func_ty)
    else {
        panic!("Concatenate must retain its prefix and ParamSpec tail")
    };
    assert_eq!(params.len(), 1);
    assert!(!variadic);
    assert_eq!(checker.tcx.get_type_var(*param_spec).kind, TypeVarKind::ParamSpec);
    let Ty::Class {
        external: Some(prefix),
        ..
    } = checker.tcx.get(params[0])
    else {
        panic!("Concatenate prefix must retain curses.window nominal identity")
    };
    assert_eq!((prefix.module.as_str(), prefix.name.as_str()), ("_curses", "window"));
}

#[test]
fn typeshed_unbound_paramspec_rejects_only_definite_noncallables() {
    let valid = check(
        "from _contextvars import Context\n\
         context = Context()\n\
         context.run(lambda: 1)\n",
    );
    assert!(
        valid.is_empty(),
        "an unbound ParamSpec must not reject a known callable: {valid:?}"
    );

    let invalid = check(
        "from _contextvars import Context\n\
         context = Context()\n\
         context.run(12345)\n",
    );
    assert!(
        invalid.iter().any(|error| {
            error.contains("argument type mismatch") && error.contains("parameter `callable`")
        }),
        "a definite non-callable must still be rejected: {invalid:?}"
    );
}

#[test]
fn context_run_binds_paramspec_args_and_kwargs_to_callback_signature() {
    let prelude =
        "from _contextvars import Context\n\
         context = Context()\n\
         def callback(x: int, /, y: str = \"d\", *, flag: bool = False) -> int:\n\
         \x20   return x\n";
    for call in [
        "context.run(callback, 1)\n",
        "context.run(callback, 1, flag=True)\n",
    ] {
        let errors = check(&format!("{prelude}{call}"));
        assert!(
            errors.is_empty(),
            "a valid ParamSpec forwarding call must pass: {call}: {errors:?}"
        );
    }

    for (call, message) in [
        ("context.run(callback)\n", "missing required callback parameter `x`"),
        ("context.run(callback, \"bad\")\n", "parameter `x`"),
        ("context.run(callback, 1, flag=\"bad\")\n", "parameter `flag`"),
        (
            "context.run(callback, x=1)\n",
            "argument that the callback does not accept",
        ),
        (
            "context.run(callback, 1, unknown=True)\n",
            "argument that the callback does not accept",
        ),
        (
            "context.run(callback, 1, \"a\", y=\"b\")\n",
            "multiple values for callback parameter `y`",
        ),
    ] {
        let errors = check(&format!("{prelude}{call}"));
        assert!(
            errors.iter().any(|error| error.contains(message)),
            "invalid ParamSpec forwarding must report `{message}`: {call}: {errors:?}"
        );
    }
}

#[test]
fn context_run_keeps_callable_signature_through_alias_flow() {
    let errors = check(
        "from _contextvars import Context\n\
         context = Context()\n\
         def callback(value: int) -> int:\n\
         \x20   return value\n\
         alias = callback\n\
         context.run(alias, \"bad\")\n",
    );
    assert!(
        errors
            .iter()
            .any(|error| error.contains("parameter `value`")),
        "callable metadata must remain intrinsic after assignment: {errors:?}"
    );
}

#[test]
fn curses_wrapper_binds_concatenate_prefix_before_forwarded_paramspec() {
    let valid = check(
        "from curses import wrapper\n\
         from _curses import window\n\
         def app(screen: window, count: int, *, title: str = \"\") -> int:\n\
         \x20   return count\n\
         wrapper(app, 1, title=\"ok\")\n",
    );
    assert!(
        valid.is_empty(),
        "Concatenate must consume the injected window prefix: {valid:?}"
    );

    let wrong_forwarded = check(
        "from curses import wrapper\n\
         from _curses import window\n\
         def app(screen: window, count: int) -> int:\n\
         \x20   return count\n\
         wrapper(app, \"bad\")\n",
    );
    assert!(
        wrong_forwarded
            .iter()
            .any(|error| error.contains("parameter `count`")),
        "forwarded arguments must be checked after the Concatenate prefix: {wrong_forwarded:?}"
    );

    let wrong_prefix = check(
        "from curses import wrapper\n\
         def app(screen: int, count: int) -> int:\n\
         \x20   return count\n\
         wrapper(app, 1)\n",
    );
    assert!(
        wrong_prefix
            .iter()
            .any(|error| error.contains("argument type mismatch")),
        "the injected window prefix must reject a callback with a different leading type: {wrong_prefix:?}"
    );

    let variadic_prefix_mismatch = check(
        "from curses import wrapper\n\
         def app(*values: int) -> int:\n\
         \x20   return 0\n\
         wrapper(app)\n",
    );
    assert!(
        variadic_prefix_mismatch
            .iter()
            .any(|error| error.contains("callable variadic parameter")),
        "a callback's *args annotation must accept the injected prefix: {variadic_prefix_mismatch:?}"
    );

    let variadic_prefix_valid = check(
        "from curses import wrapper\n\
         def app(*values: object) -> int:\n\
         \x20   return 0\n\
         wrapper(app)\n",
    );
    assert!(
        variadic_prefix_valid.is_empty(),
        "an unconstrained callback *args pack may accept the injected prefix: {variadic_prefix_valid:?}"
    );

    for callback in [
        "def app(*, screen: window) -> int:\n\x20   return 0\n",
        "def app(**values: window) -> int:\n\x20   return 0\n",
    ] {
        let errors = check(&format!(
            "from curses import wrapper\nfrom _curses import window\n{callback}wrapper(app)\n"
        ));
        assert!(
            errors.iter().any(|error| error.contains("positionally")),
            "a keyword-only callback cannot accept the injected positional prefix: {callback}: {errors:?}"
        );
    }
}

#[test]
fn callable_ellipsis_compatibility_is_arity_independent() {
    let mut checker = TypeChecker::new();
    let expected = checker.tcx.intern(Ty::Fn {
        params: Vec::new(),
        ret: checker.tcx.int(),
        variadic: true,
        signature: None,
        param_spec: None,
    });
    let zero = checker.tcx.intern(Ty::Fn {
        params: Vec::new(),
        ret: checker.tcx.int(),
        variadic: false,
        signature: None,
        param_spec: None,
    });
    let two = checker.tcx.intern(Ty::Fn {
        params: vec![checker.tcx.str(), checker.tcx.bool()],
        ret: checker.tcx.int(),
        variadic: false,
        signature: None,
        param_spec: None,
    });
    let wrong_return = checker.tcx.intern(Ty::Fn {
        params: Vec::new(),
        ret: checker.tcx.str(),
        variadic: false,
        signature: None,
        param_spec: None,
    });

    assert!(checker.types_compatible(expected, zero));
    assert!(checker.types_compatible(expected, two));
    assert!(!checker.types_compatible(expected, wrong_return));
}

#[test]
fn typeshed_indeterminate_contract_does_not_fall_back_to_compact_wall() {
    let errors = check(
        "from dataclasses import asdict\n\
         class W:\n\
         \x20   __dataclass_fields__ = {}\n\
         asdict(W())\n",
    );
    assert!(
        errors.is_empty(),
        "an incomplete generated protocol must stay indeterminate instead of falling back to the compact bare-class heuristic: {errors:?}"
    );
}

#[test]
fn typeshed_structured_projects_builtin_containers_to_abc_contracts() {
    let valid = check(
        "from operator import concat\n\
         from operator import delitem\n\
         import timeit\n\
         from distutils.sysconfig import expand_makefile_vars\n\
         concat([1], [2])\n\
         delitem([1], 0)\n\
         timeit.main([\"-n\", \"1\"])\n\
         expand_makefile_vars(\"$(NAME)\", {\"NAME\": \"mamba\"})\n",
    );
    assert!(
        valid.is_empty(),
        "list and dict must project to Sequence and Mapping: {valid:?}"
    );

    for source in [
        "from operator import concat\nconcat(1, [2])\n",
        "from operator import delitem\ndelitem((1,), 0)\n",
        "import timeit\ntimeit.main(1)\n",
        "from distutils.sysconfig import expand_makefile_vars\nexpand_makefile_vars(\"x\", [])\n",
    ] {
        let errors = check(source);
        assert!(
            errors
                .iter()
                .any(|error| error.contains("argument type mismatch")),
            "a non-container must be rejected by its generated ABC contract: {source:?} => {errors:?}"
        );
    }
}

#[test]
fn typeshed_structured_binder_enforces_required_extra_and_duplicate_arguments() {
    for (source, needle) in [
        ("from copy import copy\ncopy()\n", "missing required parameter `x`"),
        (
            "from copy import copy\ncopy(1, 2)\n",
            "argument that no parameter accepts",
        ),
        (
            "import ast\nast.parse(\"x\", source=\"y\")\n",
            "multiple values for parameter `source`",
        ),
    ] {
        let errors = check(source);
        assert!(
            errors.iter().any(|error| error.contains(needle)),
            "structured binder must reject `{source}` with `{needle}`: {errors:?}"
        );
    }

    let errors = check(
        "import ast\n\
         ast.parse(\"x\")\n\
         ast.parse(\"x\", filename=\"demo.py\", type_comments=True)\n",
    );
    assert!(
        errors.is_empty(),
        "defaulted and keyword-only parameters must remain optional: {errors:?}"
    );
}

#[test]
fn typeshed_structured_literal_overloads_match_exact_ast_values() {
    let valid = check(
        "import time\n\
         time.get_clock_info(\"monotonic\")\n\
         time.get_clock_info(\"perf_counter\")\n\
         time.get_clock_info(\"thread_time\")\n",
    );
    assert!(
        valid.is_empty(),
        "exact Literal arguments must select their overloads: {valid:?}"
    );

    let invalid = check("import time\ntime.get_clock_info(\"bogus\")\n");
    assert!(
        invalid
            .iter()
            .any(|error| error.contains("argument type mismatch")),
        "a value outside the Literal set must be rejected: {invalid:?}"
    );

    let gradual = check(
        "import time\n\
         def inspect_clock(name):\n\
         \x20   time.get_clock_info(name)\n",
    );
    assert!(
        gradual.is_empty(),
        "a dynamic value must keep Literal matching indeterminate: {gradual:?}"
    );
}

#[test]
fn typeshed_structured_constructor_binds_without_exposing_receiver() {
    for source in [
        "from modulefinder import Module\nModule()\n",
        "import modulefinder as mf\nmf.Module()\n",
    ] {
        let errors = check(source);
        assert!(
            errors
                .iter()
                .any(|error| error.contains("missing required parameter `name`")),
            "constructor must hide self and require name: {errors:?}"
        );
    }
}

#[test]
fn typeshed_structured_bound_and_builtin_methods_enforce_arguments() {
    for (source, parameter) in [
        (
            "from html.parser import HTMLParser\n\
             parser = HTMLParser()\n\
             parser.handle_entityref(1)\n",
            "name",
        ),
        (
            "import html.parser as hp\n\
             parser = hp.HTMLParser()\n\
             parser.handle_entityref(1)\n",
            "name",
        ),
        ("items = [1]\nitems.sort(reverse=\"bad\")\n", "reverse"),
    ] {
        let errors = check(source);
        assert!(
            errors.iter().any(|error| {
                error.contains("argument type mismatch")
                    && error.contains(&format!("parameter `{parameter}`"))
            }),
            "bound method must enforce `{parameter}` through TypeSpec: {errors:?}"
        );
    }
}

#[test]
fn typeshed_structured_descriptor_access_controls_receiver_visibility() {
    for (source, missing) in [
        (
            "from datetime import date\ndate.fromtimestamp()\n",
            "timestamp",
        ),
        (
            "from email.headerregistry import DateHeader\nDateHeader.value_parser()\n",
            "value",
        ),
        (
            "from html.parser import HTMLParser\n\
             HTMLParser.handle_entityref(object.__new__(HTMLParser))\n",
            "name",
        ),
    ] {
        let errors = check(source);
        assert!(
            errors.iter().any(|error| {
                error.contains(&format!("missing required parameter `{missing}`"))
            }),
            "descriptor binding must expose only the Python-visible parameters: {errors:?}"
        );
    }
}

#[test]
fn typeshed_structured_alias_expansion_enforces_literal_contracts() {
    let valid = check(
        "import logging\n\
         logging.basicConfig(style=\"%\")\n\
         logging.basicConfig(style=\"{\")\n\
         logging.basicConfig(style=\"$\")\n",
    );
    assert!(
        valid.is_empty(),
        "typeshed alias Literal members must remain valid: {valid:?}"
    );

    let invalid = check("import logging\nlogging.basicConfig(style=\"bogus\")\n");
    assert!(
        invalid.iter().any(|error| {
            error.contains("argument type mismatch") && error.contains("parameter `style`")
        }),
        "expanded alias target must reject values outside its Literal set: {invalid:?}"
    );
}
