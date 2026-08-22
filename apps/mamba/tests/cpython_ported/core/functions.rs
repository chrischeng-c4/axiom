//! Ported from Lib/test/test_grammar_functions.py
//! Integration tests: core/functions.rs

use super::super::harness::*;

/// Ported from `Lib/test/test_grammar_functions.py`.
#[test]
fn test_operator_logical_and_short_circuit() {
    let out = jit_capture(
        r#"print(True and True)
print(True and False)
print(False and True)
print(False and False)
"#,
    );
    assert_output(&out, "True\nFalse\nFalse\nFalse\n");
}

/// Ported from `Lib/test/test_grammar_functions.py`.
#[test]
fn test_operator_logical_or_short_circuit() {
    let out = jit_capture(
        r#"print(True or True)
print(True or False)
print(False or True)
print(False or False)
"#,
    );
    assert_output(&out, "True\nTrue\nTrue\nFalse\n");
}

/// Ported from `Lib/test/test_grammar_functions.py`.
#[test]
fn test_operator_is_same_int() {
    let out = jit_capture(
        r#"a = 5
b = a
print(a is b)
"#,
    );
    assert_output(&out, "True\n");
}

/// Ported from `Lib/test/test_grammar_functions.py`.
#[test]
fn test_operator_is_none() {
    let out = jit_capture(
        r#"x = None
print(x is None)
print(x is not None)
"#,
    );
    assert_output(&out, "True\nFalse\n");
}

/// Ported from `Lib/test/test_grammar_functions.py`.
#[test]
fn test_operator_is_not_distinct_lists() {
    let out = jit_capture(
        r#"a = [1, 2]
b = [1, 2]
print(a is b)
print(a is not b)
"#,
    );
    assert_output(&out, "False\nTrue\n");
}

/// Ported from `Lib/test/test_grammar_functions.py`.
#[test]
fn test_operator_logical_and_returns_value() {
    let out = jit_capture(
        r#"print(1 and 2)
print(0 and 2)
print("a" and "b")
print("" and "b")
"#,
    );
    assert_output(&out, "2\n0\nb\n\n");
}

/// Ported from `Lib/test/test_grammar_functions.py`.
#[test]
fn test_operator_logical_or_returns_value() {
    let out = jit_capture(
        r#"print(1 or 2)
print(0 or 2)
print("" or "b")
print("a" or "b")
"#,
    );
    assert_output(&out, "1\n2\nb\na\n");
}

/// Ported from `Lib/test/test_grammar_functions.py`.
#[test]
fn test_operator_not_truthy() {
    let out = jit_capture(
        r#"print(not True)
print(not False)
print(not 0)
print(not 1)
print(not "")
print(not "x")
print(not [])
print(not [1])
"#,
    );
    assert_output(&out, "False\nTrue\nTrue\nFalse\nTrue\nFalse\nTrue\nFalse\n");
}

/// Ported from `Lib/test/test_grammar_functions.py`.
#[test]
fn test_operator_chained_comparison_lt() {
    let out = jit_capture(
        r#"print(1 < 2 < 3)
print(1 < 3 < 2)
print(3 < 2 < 1)
"#,
    );
    assert_output(&out, "True\nFalse\nFalse\n");
}

/// Ported from `Lib/test/test_grammar_functions.py`.
#[test]
fn test_operator_chained_comparison_mixed() {
    let out = jit_capture(
        r#"print(1 < 2 == 2)
print(1 <= 1 < 2)
print(3 > 2 > 1)
"#,
    );
    assert_output(&out, "True\nTrue\nTrue\n");
}

/// Ported from `Lib/test/test_grammar_functions.py`.
#[test]
fn test_operator_unary_negation() {
    let out = jit_capture(
        r#"print(-5)
print(-(-5))
print(--5)
"#,
    );
    assert_output(&out, "-5\n5\n5\n");
}

/// Ported from `Lib/test/test_grammar_functions.py`.
#[test]
fn test_operator_unary_plus() {
    let out = jit_capture(
        r#"print(+5)
print(+(-5))
"#,
    );
    assert_output(&out, "5\n-5\n");
}

/// Ported from `Lib/test/test_grammar_functions.py`.
#[test]
fn test_operator_membership_in_list() {
    let out = jit_capture(
        r#"print(2 in [1, 2, 3])
print(4 in [1, 2, 3])
print(2 not in [1, 2, 3])
print(4 not in [1, 2, 3])
"#,
    );
    assert_output(&out, "True\nFalse\nFalse\nTrue\n");
}

/// Ported from `Lib/test/test_grammar_functions.py`.
#[test]
fn test_operator_membership_in_str() {
    let out = jit_capture(
        r#"print("ll" in "hello")
print("xy" in "hello")
print("ll" not in "hello")
"#,
    );
    assert_output(&out, "True\nFalse\nFalse\n");
}

/// Ported from `Lib/test/test_grammar_functions.py`.
#[test]
fn test_operator_membership_in_tuple() {
    let out = jit_capture(
        r#"print(2 in (1, 2, 3))
print(4 in (1, 2, 3))
"#,
    );
    assert_output(&out, "True\nFalse\n");
}

/// Ported from `Lib/test/test_grammar_functions.py`.
#[test]
fn test_operator_membership_in_dict() {
    let out = jit_capture(
        r#"d = {"a": 1, "b": 2}
print("a" in d)
print("c" in d)
print("a" not in d)
"#,
    );
    assert_output(&out, "True\nFalse\nFalse\n");
}

/// Ported from `Lib/test/test_grammar_functions.py`.
#[test]
fn test_operator_comparison_strings_lex() {
    let out = jit_capture(
        r#"print("a" < "b")
print("apple" < "banana")
print("abc" == "abc")
print("abc" != "abd")
"#,
    );
    assert_output(&out, "True\nTrue\nTrue\nTrue\n");
}

/// Ported from `Lib/test/test_grammar_functions.py`.
#[test]
fn test_operator_arith_precedence() {
    let out = jit_capture(
        r#"print(1 + 2 * 3)
print((1 + 2) * 3)
print(2 ** 3 ** 2)
print(20 // 3 % 4)
"#,
    );
    assert_output(&out, "7\n9\n512\n2\n");
}

/// Ported from `Lib/test/test_grammar_functions.py`.
#[test]
fn test_operator_bitwise_combined() {
    let out = jit_capture(
        r#"print(0b1100 & 0b1010)
print(0b1100 | 0b1010)
print(0b1100 ^ 0b1010)
print(~0)
print(1 << 4)
print(16 >> 2)
"#,
    );
    assert_output(&out, "8\n14\n6\n-1\n16\n4\n");
}

/// Ported from `Lib/test/test_grammar_functions.py`.
#[test]
fn test_operator_augmented_assignment_chain() {
    let out = jit_capture(
        r#"x = 10
x += 5
x -= 2
x *= 3
x //= 2
x %= 5
print(x)
"#,
    );
    assert_output(&out, "4\n");
}

/// Ported from `Lib/test/test_grammar_functions.py`.
#[test]
fn test_with_basic_enter_exit_order() {
    let out = jit_capture(
        r#"class CM:
    def __enter__(self):
        print("enter")
        return self
    def __exit__(self, exc_type, exc_val, tb):
        print("exit")
        return False

with CM():
    print("inside")
"#,
    );
    assert_output(&out, "enter\ninside\nexit\n");
}

/// Ported from `Lib/test/test_grammar_functions.py`.
#[test]
fn test_with_as_binding() {
    let out = jit_capture(
        r#"class CM:
    def __enter__(self):
        print("enter")
        return 42
    def __exit__(self, exc_type, exc_val, tb):
        print("exit")
        return False

with CM() as v:
    print(v)
"#,
    );
    assert_output(&out, "enter\n42\nexit\n");
}

/// Ported from `Lib/test/test_grammar_functions.py`.
#[test]
fn test_with_returns_self() {
    let out = jit_capture(
        r#"class CM:
    def __init__(self, label):
        self.label = label
    def __enter__(self):
        return self
    def __exit__(self, exc_type, exc_val, tb):
        return False

with CM("hello") as cm:
    print(cm.label)
"#,
    );
    assert_output(&out, "hello\n");
}

/// Ported from `Lib/test/test_grammar_functions.py`.
#[test]
fn test_with_exit_called_on_normal_completion() {
    let out = jit_capture(
        r#"class CM:
    def __init__(self, name):
        self.name = name
    def __enter__(self):
        print("enter", self.name)
        return self
    def __exit__(self, exc_type, exc_val, tb):
        print("exit", self.name)
        return False

with CM("a"):
    print("body")
print("after")
"#,
    );
    assert_output(&out, "enter a\nbody\nexit a\nafter\n");
}

/// Ported from `Lib/test/test_grammar_functions.py`.
#[test]
fn test_with_nested() {
    let out = jit_capture(
        r#"class CM:
    def __init__(self, name):
        self.name = name
    def __enter__(self):
        print("enter", self.name)
        return self
    def __exit__(self, exc_type, exc_val, tb):
        print("exit", self.name)
        return False

with CM("outer"):
    with CM("inner"):
        print("body")
"#,
    );
    assert_output(
        &out,
        "enter outer\nenter inner\nbody\nexit inner\nexit outer\n",
    );
}

/// Ported from `Lib/test/test_grammar_functions.py`.
#[test]
fn test_with_multiple_managers_one_with() {
    let out = jit_capture(
        r#"class CM:
    def __init__(self, name):
        self.name = name
    def __enter__(self):
        print("enter", self.name)
        return self
    def __exit__(self, exc_type, exc_val, tb):
        print("exit", self.name)
        return False

with CM("a"), CM("b"):
    print("body")
"#,
    );
    assert_output(&out, "enter a\nenter b\nbody\nexit b\nexit a\n");
}

/// Ported from `Lib/test/test_grammar_functions.py`.
#[test]
fn test_with_multiple_managers_with_as_binding() {
    let out = jit_capture(
        r#"class CM:
    def __init__(self, val):
        self.val = val
    def __enter__(self):
        return self.val
    def __exit__(self, exc_type, exc_val, tb):
        return False

with CM(1) as a, CM(2) as b:
    print(a + b)
"#,
    );
    assert_output(&out, "3\n");
}

/// Ported from `Lib/test/test_grammar_functions.py`.
#[test]
fn test_with_exit_receives_none_when_no_exception() {
    let out = jit_capture(
        r#"class CM:
    def __enter__(self):
        return self
    def __exit__(self, exc_type, exc_val, tb):
        print(exc_type is None)
        print(exc_val is None)
        print(tb is None)
        return False

with CM():
    pass
"#,
    );
    assert_output(&out, "True\nTrue\nTrue\n");
}

/// Ported from `Lib/test/test_grammar_functions.py`.
#[test]
fn test_dunder_add_returns_new_instance() {
    let out = jit_capture(
        r#"class Vec:
    def __init__(self, x, y):
        self.x = x
        self.y = y
    def __add__(self, other):
        return Vec(self.x + other.x, self.y + other.y)
    def __repr__(self):
        return f"Vec({self.x}, {self.y})"

a = Vec(1, 2)
b = Vec(3, 4)
print(a + b)
"#,
    );
    assert_output(&out, "Vec(4, 6)\n");
}

/// Ported from `Lib/test/test_grammar_functions.py`.
#[test]
fn test_dunder_eq_compares_by_field() {
    let out = jit_capture(
        r#"class Vec:
    def __init__(self, x, y):
        self.x = x
        self.y = y
    def __eq__(self, other):
        return self.x == other.x and self.y == other.y

print(Vec(1, 2) == Vec(1, 2))
print(Vec(1, 2) == Vec(3, 4))
"#,
    );
    assert_output(&out, "True\nFalse\n");
}

/// Ported from `Lib/test/test_grammar_functions.py`.
#[test]
fn test_dunder_len_drives_builtin_len() {
    let out = jit_capture(
        r#"class Bag:
    def __init__(self):
        self.items = []
    def __len__(self):
        return len(self.items)
    def add(self, x):
        self.items.append(x)

b = Bag()
print(len(b))
b.add(1)
b.add(2)
b.add(3)
print(len(b))
"#,
    );
    assert_output(&out, "0\n3\n");
}

/// Ported from `Lib/test/test_grammar_functions.py`.
#[test]
fn test_inner_def_captures_outer_local() {
    let out = jit_capture(
        r#"def make_adder(x):
    def add(y):
        return x + y
    return add
add5 = make_adder(5)
print(add5(3))
print(add5(10))
"#,
    );
    assert_output(&out, "8\n15\n");
}

/// Ported from `Lib/test/test_grammar_functions.py`.
#[test]
fn test_nested_def_called_inside_outer() {
    let out = jit_capture(
        r#"def outer(n):
    def double(x):
        return x * 2
    return double(n) + 1
print(outer(3))
print(outer(7))
"#,
    );
    assert_output(&out, "7\n15\n");
}

/// Ported from `Lib/test/test_grammar_functions.py`.
#[test]
fn test_function_list_iteration() {
    let out = jit_capture(
        r#"def add(a, b):
    return a + b

def mul(a, b):
    return a * b

ops = [add, mul]
for op in ops:
    print(op(3, 4))
"#,
    );
    assert_output(&out, "7\n12\n");
}

/// Ported from `Lib/test/test_grammar_functions.py`.
#[test]
fn test_function_passed_as_argument() {
    let out = jit_capture(
        r#"def add(a, b):
    return a + b

def mul(a, b):
    return a * b

def apply(f, x, y):
    return f(x, y)

print(apply(add, 10, 20))
print(apply(mul, 5, 6))
"#,
    );
    assert_output(&out, "30\n30\n");
}

/// Ported from `Lib/test/test_grammar_functions.py`.
#[test]
fn test_function_dispatch_via_dict() {
    let out = jit_capture(
        r#"def add(a, b):
    return a + b

def sub(a, b):
    return a - b

def mul(a, b):
    return a * b

ops = {"+": add, "-": sub, "*": mul}
print(ops["+"](7, 3))
print(ops["-"](7, 3))
print(ops["*"](7, 3))
"#,
    );
    assert_output(&out, "10\n4\n21\n");
}

