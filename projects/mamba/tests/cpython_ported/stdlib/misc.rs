//! Ported from Lib/test/test_misc_ported.py
//! Integration tests: stdlib/misc.rs

use super::super::harness::*;

/// Ported from `Lib/test/test_misc_ported.py`.
#[test]
fn test_join_basic_and_edge_cases() {
    let out = jit_capture(
        r#"print(",".join(["a", "b", "c"]))
print("-".join(["one"]))
print("".join(["h", "e", "l", "l", "o"]))
print(" ".join([]))
"#,
    );
    assert_output(&out, "a,b,c\none\nhello\n\n");
}

/// Ported from `Lib/test/test_misc_ported.py`.
#[test]
fn test_join_over_listcomp() {
    let out = jit_capture(
        r#"print(":".join([str(x) for x in range(5)]))
"#,
    );
    assert_output(&out, "0:1:2:3:4\n");
}

/// Ported from `Lib/test/test_misc_ported.py`.
#[test]
fn test_join_over_genexp_and_tuple() {
    let out = jit_capture(
        r#"print(",".join(str(i) for i in range(6)))
print("|".join(("x", "y", "z")))
"#,
    );
    assert_output(&out, "0,1,2,3,4,5\nx|y|z\n");
}

/// Ported from `Lib/test/test_misc_ported.py`.
#[test]
fn test_enumerate_default_start_zero() {
    let out = jit_capture(
        r#"for i, c in enumerate(['a', 'b', 'c']):
    print(i, c)
"#,
    );
    assert_output(&out, "0 a\n1 b\n2 c\n");
}

/// Ported from `Lib/test/test_misc_ported.py`.
#[test]
fn test_enumerate_with_start_kwarg() {
    let out = jit_capture(
        r#"for i, c in enumerate(['x', 'y'], start=10):
    print(i, c)
"#,
    );
    assert_output(&out, "10 x\n11 y\n");
}

/// Ported from `Lib/test/test_misc_ported.py`.
#[test]
fn test_enumerate_to_list_of_pairs() {
    let out = jit_capture(
        r#"print(list(enumerate(['p', 'q'])))
"#,
    );
    assert_output(&out, "[(0, 'p'), (1, 'q')]\n");
}

/// Ported from `Lib/test/test_misc_ported.py`.
#[test]
fn test_positional_and_named_placeholders() {
    let out = jit_capture(
        r#"print("Name: {}, Age: {}".format("Alice", 30))
print("{0} + {1} = {2}".format(2, 3, 5))
print("{name} is {age}".format(name="Bob", age=25))
"#,
    );
    assert_output(&out, "Name: Alice, Age: 30\n2 + 3 = 5\nBob is 25\n");
}

/// Ported from `Lib/test/test_misc_ported.py`.
#[test]
fn test_int_width_zero_pad_and_precision() {
    let out = jit_capture(
        r#"print("{:5d}".format(42))
print("{:05d}".format(42))
print("{:.2f}".format(3.14159))
print("{:.0f}".format(2.5))
print("{:6.3f}".format(1.5))
"#,
    );
    assert_output(&out, "   42\n00042\n3.14\n2\n 1.500\n");
}

/// Ported from `Lib/test/test_misc_ported.py`.
#[test]
fn test_align_right_left_center() {
    let out = jit_capture(
        r#"print("{:>10}".format("right"))
print("{:<10}|".format("left"))
print("{:^10}|".format("mid"))
print("{:*^9}".format("hi"))
"#,
    );
    assert_output(&out, "     right\nleft      |\n   mid    |\n***hi****\n");
}

/// Ported from `Lib/test/test_misc_ported.py`.
#[test]
fn test_hashlib_md5_hello() {
    let out = jit_capture(
        r#"import hashlib
print(hashlib.md5(b"hello").hexdigest())
"#,
    );
    assert_output(&out, "5d41402abc4b2a76b9719d911017c592\n");
}

/// Ported from `Lib/test/test_misc_ported.py`.
#[test]
fn test_hashlib_sha1_hello() {
    let out = jit_capture(
        r#"import hashlib
print(hashlib.sha1(b"hello").hexdigest())
"#,
    );
    assert_output(&out, "aaf4c61ddcc5e8a2dabede0f3b482cd9aea9434d\n");
}

/// Ported from `Lib/test/test_misc_ported.py`.
#[test]
fn test_hashlib_sha256_hello() {
    let out = jit_capture(
        r#"import hashlib
print(hashlib.sha256(b"hello").hexdigest())
"#,
    );
    assert_output(
        &out,
        "2cf24dba5fb0a30e26e83b2ac5b9e29e1b161e5c1fa7425e73043362938b9824\n",
    );
}

/// Ported from `Lib/test/test_misc_ported.py`.
#[test]
fn test_hashlib_md5_empty() {
    let out = jit_capture(
        r#"import hashlib
print(hashlib.md5(b"").hexdigest())
"#,
    );
    assert_output(&out, "d41d8cd98f00b204e9800998ecf8427e\n");
}

/// Ported from `Lib/test/test_misc_ported.py`.
#[test]
fn test_hashlib_sha256_empty() {
    let out = jit_capture(
        r#"import hashlib
print(hashlib.sha256(b"").hexdigest())
"#,
    );
    assert_output(
        &out,
        "e3b0c44298fc1c149afbf4c8996fb92427ae41e4649b934ca495991b7852b855\n",
    );
}

/// Ported from `Lib/test/test_misc_ported.py`.
#[test]
fn test_hashlib_sha512_hello() {
    let out = jit_capture(
        r#"import hashlib
print(hashlib.sha512(b"hello").hexdigest())
"#,
    );
    assert_output(
        &out,
        "9b71d224bd62f3785d96d46ad3ea3d73319bfbc2890caadae2dff72519673ca72323c3d99ba5c11d7c7acc6e14b8c5da0c4663475c2e5c3adef46f73bcdec043\n",
    );
}

/// Ported from `Lib/test/test_misc_ported.py`.
#[test]
fn test_star_args_arity() {
    let out = jit_capture(
        r#"def variadic(*args):
    return (len(args), sum(args))

print(variadic())
print(variadic(10))
print(variadic(1, 2, 3))
print(variadic(1, 2, 3, 4, 5))
"#,
    );
    assert_output(&out, "(0, 0)\n(1, 10)\n(3, 6)\n(5, 15)\n");
}

/// Ported from `Lib/test/test_misc_ported.py`.
#[test]
fn test_kwargs_only() {
    let out = jit_capture(
        r#"def kw_only(**kwargs):
    return sorted(kwargs.items())

print(kw_only())
print(kw_only(a=1, b=2))
print(kw_only(x=10, y=20, z=30))
"#,
    );
    assert_output(
        &out,
        "[]\n[('a', 1), ('b', 2)]\n[('x', 10), ('y', 20), ('z', 30)]\n",
    );
}

/// Ported from `Lib/test/test_misc_ported.py`.
#[test]
fn test_fixed_then_star_args() {
    let out = jit_capture(
        r#"def head_rest(first, *rest):
    return (first, list(rest))

print(head_rest(10))
print(head_rest(10, 20))
print(head_rest("a", "b", "c", "d"))
"#,
    );
    assert_output(&out, "(10, [])\n(10, [20])\n('a', ['b', 'c', 'd'])\n");
}

/// Ported from `Lib/test/test_misc_ported.py`.
#[test]
fn test_default_argument_value() {
    let out = jit_capture(
        r#"def greet(name, greeting="Hello"):
    print(greeting, name)
greet("World")
greet("Alice", "Hi")
"#,
    );
    assert_output(&out, "Hello World\nHi Alice\n");
}

/// Ported from `Lib/test/test_misc_ported.py`.
#[test]
fn test_keyword_argument_at_call_site() {
    let out = jit_capture(
        r#"def make(a, b, c):
    print(a, b, c)
make(1, 2, 3)
make(a=1, b=2, c=3)
make(1, c=3, b=2)
"#,
    );
    assert_output(&out, "1 2 3\n1 2 3\n1 2 3\n");
}

/// Ported from `Lib/test/test_misc_ported.py`.
#[test]
fn test_default_overridden_by_keyword() {
    let out = jit_capture(
        r#"def power(base, exp=2):
    return base ** exp
print(power(3))
print(power(3, 3))
print(power(3, exp=4))
"#,
    );
    assert_output(&out, "9\n27\n81\n");
}

/// Ported from `Lib/test/test_misc_ported.py`.
#[test]
fn test_custom_iter_in_for_loop_countdown() {
    let out = jit_capture(
        r#"class CountDown:
    def __init__(self, n):
        self.n = n
    def __iter__(self):
        return self
    def __next__(self):
        if self.n <= 0:
            raise StopIteration
        self.n -= 1
        return self.n + 1

for v in CountDown(3):
    print(v)
"#,
    );
    assert_output(&out, "3\n2\n1\n");
}

/// Ported from `Lib/test/test_misc_ported.py`.
#[test]
fn test_custom_iter_consumed_by_list() {
    let out = jit_capture(
        r#"class Range3:
    def __init__(self):
        self.i = 0
    def __iter__(self):
        return self
    def __next__(self):
        if self.i >= 3:
            raise StopIteration
        v = self.i
        self.i += 1
        return v

print(list(Range3()))
"#,
    );
    assert_output(&out, "[0, 1, 2]\n");
}

/// Ported from `Lib/test/test_misc_ported.py`.
#[test]
fn test_getattr_with_and_without_default() {
    let out = jit_capture(
        r#"class Box:
    pass
b = Box()
b.x = 10
print(getattr(b, "x"))
print(getattr(b, "y", 99))
print(getattr(b, "z", "missing"))
"#,
    );
    assert_output(&out, "10\n99\nmissing\n");
}

/// Ported from `Lib/test/test_misc_ported.py`.
#[test]
fn test_hasattr_present_and_absent() {
    let out = jit_capture(
        r#"class Box:
    pass
b = Box()
b.x = 1
print(hasattr(b, "x"))
print(hasattr(b, "z"))
"#,
    );
    assert_output(&out, "True\nFalse\n");
}

/// Ported from `Lib/test/test_misc_ported.py`.
#[test]
fn test_setattr_mutation_visible_via_attribute_and_getattr() {
    let out = jit_capture(
        r#"class Box:
    pass
b = Box()
setattr(b, "y", 20)
print(b.y)
print(getattr(b, "y"))
setattr(b, "y", 30)
print(b.y)
"#,
    );
    assert_output(&out, "20\n20\n30\n");
}

/// Ported from `Lib/test/test_misc_ported.py`.
#[test]
fn test_dict_from_zip() {
    let out = jit_capture(
        r#"keys = ["a", "b", "c"]
vals = [1, 2, 3]
d = dict(zip(keys, vals))
print(sorted(d.items()))
"#,
    );
    assert_output(&out, "[('a', 1), ('b', 2), ('c', 3)]\n");
}

/// Ported from `Lib/test/test_misc_ported.py`.
#[test]
fn test_list_of_zip_tuples_and_strings() {
    let out = jit_capture(
        r#"print(list(zip([1, 2, 3], ["a", "b", "c"])))
print(list(zip("abc", "xyz")))
"#,
    );
    assert_output(
        &out,
        "[(1, 'a'), (2, 'b'), (3, 'c')]\n[('a', 'x'), ('b', 'y'), ('c', 'z')]\n",
    );
}

/// Ported from `Lib/test/test_misc_ported.py`.
#[test]
fn test_zip_stops_at_shorter() {
    let out = jit_capture(
        r#"print(list(zip([1, 2, 3, 4, 5], ["a", "b"])))
print(list(zip([], [1, 2, 3])))
"#,
    );
    assert_output(&out, "[(1, 'a'), (2, 'b')]\n[]\n");
}

/// Ported from `Lib/test/test_misc_ported.py`.
#[test]
fn test_global_rebinds_module_variable() {
    let out = jit_capture(
        r#"counter = 0
def inc():
    global counter
    counter += 1
inc()
inc()
inc()
print(counter)
"#,
    );
    assert_output(&out, "3\n");
}

/// Ported from `Lib/test/test_misc_ported.py`.
#[test]
fn test_nonlocal_rebinds_enclosing_local() {
    let out = jit_capture(
        r#"def outer():
    x = 10
    def inner():
        nonlocal x
        x = 20
    inner()
    return x
print(outer())
"#,
    );
    assert_output(&out, "20\n");
}

/// Ported from `Lib/test/test_misc_ported.py`.
#[test]
fn test_global_seen_across_multiple_functions() {
    let out = jit_capture(
        r#"total = 0
def add(n):
    global total
    total += n
def show():
    print(total)
add(5)
add(7)
show()
"#,
    );
    assert_output(&out, "12\n");
}

/// Ported from `Lib/test/test_misc_ported.py`.
#[test]
fn test_while_index_summation() {
    let out = jit_capture(
        r#"xs = [1, 2, 3, 4, 5]
total = 0
i = 0
while i < len(xs):
    total = total + xs[i]
    i = i + 1
print(total)
"#,
    );
    assert_output(&out, "15\n");
}

/// Ported from `Lib/test/test_misc_ported.py`.
#[test]
fn test_while_countdown_with_step() {
    let out = jit_capture(
        r#"i = 10
while i > 0:
    print(i, end=" ")
    i = i - 2
print()
"#,
    );
    assert_output(&out, "10 8 6 4 2 \n");
}

/// Ported from `Lib/test/test_misc_ported.py`.
#[test]
fn test_while_true_break() {
    let out = jit_capture(
        r#"n = 0
while True:
    n = n + 1
    if n >= 5:
        break
print(n)
"#,
    );
    assert_output(&out, "5\n");
}

/// Ported from `Lib/test/test_misc_ported.py`.
#[test]
fn test_difflib_get_close_matches_typo() {
    let out = jit_capture(
        r#"import difflib
print(difflib.get_close_matches("appel", ["apple", "ape", "banana"]))
"#,
    );
    assert_output(&out, "['apple', 'ape']\n");
}

/// Ported from `Lib/test/test_misc_ported.py`.
#[test]
fn test_difflib_get_close_matches_no_hit() {
    let out = jit_capture(
        r#"import difflib
print(difflib.get_close_matches("xyz", ["apple", "banana", "cherry"]))
"#,
    );
    assert_output(&out, "[]\n");
}

/// Ported from `Lib/test/test_misc_ported.py`.
#[test]
fn test_difflib_get_close_matches_exact() {
    let out = jit_capture(
        r#"import difflib
print(difflib.get_close_matches("apple", ["apple", "ape", "banana"]))
"#,
    );
    assert_output(&out, "['apple', 'ape']\n");
}

/// Ported from `Lib/test/test_misc_ported.py`.
#[test]
fn test_binascii_hexlify_short() {
    let out = jit_capture(
        r#"import binascii
print(binascii.hexlify(b"hi").decode())
"#,
    );
    assert_output(&out, "6869\n");
}

/// Ported from `Lib/test/test_misc_ported.py`.
#[test]
fn test_binascii_unhexlify_short() {
    let out = jit_capture(
        r#"import binascii
print(binascii.unhexlify("6869").decode())
"#,
    );
    assert_output(&out, "hi\n");
}

/// Ported from `Lib/test/test_misc_ported.py`.
#[test]
fn test_binascii_hexlify_roundtrip() {
    let out = jit_capture(
        r#"import binascii
data = b"hello world"
encoded = binascii.hexlify(data)
print(binascii.unhexlify(encoded) == data)
"#,
    );
    assert_output(&out, "True\n");
}

/// Ported from `Lib/test/test_misc_ported.py`.
#[test]
fn test_binascii_b2a_a2b_hex_alias() {
    let out = jit_capture(
        r#"import binascii
print(binascii.b2a_hex(b"AB").decode())
print(binascii.a2b_hex("4142").decode())
"#,
    );
    assert_output(&out, "4142\nAB\n");
}

/// Ported from `Lib/test/test_misc_ported.py`.
#[test]
fn test_format_positional_and_numbered_fields() {
    let out = jit_capture(
        r#"print("{} {}".format("hello", "world"))
print("{0} {1} {0}".format("a", "b"))
"#,
    );
    assert_output(&out, "hello world\na b a\n");
}

/// Ported from `Lib/test/test_misc_ported.py`.
#[test]
fn test_format_keyword_fields() {
    let out = jit_capture(
        r#"print("{name} is {age}".format(name="Alice", age=30))
print("{a}+{b}={c}".format(a=1, b=2, c=3))
"#,
    );
    assert_output(&out, "Alice is 30\n1+2=3\n");
}

/// Ported from `Lib/test/test_misc_ported.py`.
#[test]
fn test_format_alignment_specifiers() {
    let out = jit_capture(
        r#"print(repr("{:>5}".format("hi")))
print(repr("{:<5}".format("hi")))
print(repr("{:^5}".format("hi")))
"#,
    );
    assert_output(&out, "'   hi'\n'hi   '\n' hi  '\n");
}

/// Ported from `Lib/test/test_misc_ported.py`.
#[test]
fn test_format_numeric_specifiers() {
    let out = jit_capture(
        r#"print("{:05d}".format(42))
print("{:.2f}".format(3.14159))
print("{:.4f}".format(2.71828))
"#,
    );
    assert_output(&out, "00042\n3.14\n2.7183\n");
}

/// Ported from `Lib/test/test_misc_ported.py`.
#[test]
fn test_copy_shallow_list_independence() {
    let out = jit_capture(
        r#"import copy
a = [1, 2, 3]
b = copy.copy(a)
b.append(4)
print(a)
print(b)
"#,
    );
    assert_output(&out, "[1, 2, 3]\n[1, 2, 3, 4]\n");
}

/// Ported from `Lib/test/test_misc_ported.py`.
#[test]
fn test_copy_shallow_nested_shares_inner() {
    let out = jit_capture(
        r#"import copy
a = [1, [2, 3]]
b = copy.copy(a)
b[1].append(99)
print(a)
print(b)
"#,
    );
    assert_output(&out, "[1, [2, 3, 99]]\n[1, [2, 3, 99]]\n");
}

/// Ported from `Lib/test/test_misc_ported.py`.
#[test]
fn test_copy_deepcopy_isolates_nested_list() {
    let out = jit_capture(
        r#"import copy
a = [1, [2, 3]]
b = copy.deepcopy(a)
b[1].append(99)
print(a)
print(b)
"#,
    );
    assert_output(&out, "[1, [2, 3]]\n[1, [2, 3, 99]]\n");
}

/// Ported from `Lib/test/test_misc_ported.py`.
#[test]
fn test_copy_deepcopy_isolates_nested_dict() {
    let out = jit_capture(
        r#"import copy
d = {"a": [1, 2], "b": {"c": 3}}
e = copy.deepcopy(d)
e["a"].append(99)
e["b"]["c"] = 999
print(d["a"])
print(d["b"]["c"])
print(e["a"])
print(e["b"]["c"])
"#,
    );
    assert_output(&out, "[1, 2]\n3\n[1, 2, 99]\n999\n");
}

/// Ported from `Lib/test/test_misc_ported.py`.
#[test]
fn test_copy_copy_dict_top_level_independence() {
    let out = jit_capture(
        r#"import copy
d = {"a": 1, "b": 2}
e = copy.copy(d)
e["c"] = 3
print("c" in d)
print("c" in e)
"#,
    );
    assert_output(&out, "False\nTrue\n");
}

/// Ported from `Lib/test/test_misc_ported.py`.
#[test]
fn test_starred_head_and_tail() {
    let out = jit_capture(
        r#"first, *rest = [1, 2, 3, 4, 5]
print(first)
print(rest)
*head, last = [1, 2, 3, 4, 5]
print(head)
print(last)
"#,
    );
    assert_output(&out, "1\n[2, 3, 4, 5]\n[1, 2, 3, 4]\n5\n");
}

/// Ported from `Lib/test/test_misc_ported.py`.
#[test]
fn test_starred_middle_target() {
    let out = jit_capture(
        r#"a, *mid, z = [1, 2, 3, 4, 5]
print(a, mid, z)
"#,
    );
    assert_output(&out, "1 [2, 3, 4] 5\n");
}

/// Ported from `Lib/test/test_misc_ported.py`.
#[test]
fn test_tuple_unpack_for_loop_and_swap() {
    let out = jit_capture(
        r#"x, y = 10, 20
x, y = y, x
print(x, y)

pairs = [(1, "a"), (2, "b"), (3, "c")]
for n, s in pairs:
    print(n, s)
"#,
    );
    assert_output(&out, "20 10\n1 a\n2 b\n3 c\n");
}

/// Ported from `Lib/test/test_misc_ported.py`.
#[test]
fn test_default_value_used_when_omitted() {
    let out = jit_capture(
        r#"def greet(name="world"):
    return "Hello, " + name + "!"

print(greet())
print(greet("Alice"))
print(greet(name="Bob"))
"#,
    );
    assert_output(&out, "Hello, world!\nHello, Alice!\nHello, Bob!\n");
}

/// Ported from `Lib/test/test_misc_ported.py`.
#[test]
fn test_multiple_defaults_and_keyword_call() {
    let out = jit_capture(
        r#"def power(base, exp=2):
    return base ** exp

print(power(3))
print(power(3, 3))
print(power(base=4))
"#,
    );
    assert_output(&out, "9\n27\n16\n");
}

/// Ported from `Lib/test/test_misc_ported.py`.
#[test]
fn test_keyword_args_out_of_order() {
    let out = jit_capture(
        r#"def power(base, exp=2):
    return base ** exp

print(power(exp=4, base=2))
print(power(exp=0, base=99))
print(power(base=5, exp=3))
"#,
    );
    assert_output(&out, "16\n1\n125\n");
}

/// Ported from `Lib/test/test_misc_ported.py`.
#[test]
fn test_bool_coercion_across_types() {
    let out = jit_capture(
        r#"print(bool(0))
print(bool(1))
print(bool(""))
print(bool("hi"))
print(bool([]))
print(bool([0]))
print(bool({}))
print(bool({1: 2}))
print(bool(None))
"#,
    );
    assert_output(
        &out,
        "False\nTrue\nFalse\nTrue\nFalse\nTrue\nFalse\nTrue\nFalse\n",
    );
}

/// Ported from `Lib/test/test_misc_ported.py`.
#[test]
fn test_truthiness_in_if() {
    let out = jit_capture(
        r#"if [] or [1]:
    print("nonempty")
if not 0 and "x":
    print("not0 and x")
if None or 0 or "":
    print("never")
else:
    print("all falsy")
if "" or {} or 0:
    print("never2")
else:
    print("else2")
"#,
    );
    assert_output(&out, "nonempty\nnot0 and x\nall falsy\nelse2\n");
}

/// Ported from `Lib/test/test_misc_ported.py`.
#[test]
fn test_short_circuit_returns_operand() {
    let out = jit_capture(
        r#"print(0 or "fallback")
print("first" or "second")
print(0 and "never")
print("yes" and "winner")
print([] or [1, 2])
print([3] and [4, 5])
"#,
    );
    assert_output(&out, "fallback\nfirst\n0\nwinner\n[1, 2]\n[4, 5]\n");
}

/// Ported from `Lib/test/test_misc_ported.py`.
#[test]
fn test_is_none_and_is_not_none() {
    let out = jit_capture(
        r#"x = None
y = "abc"
print(x is None)
print(y is None)
print(x is not None)
print(y is not None)
print(None is None)
"#,
    );
    assert_output(&out, "True\nFalse\nFalse\nTrue\nTrue\n");
}

/// Ported from `Lib/test/test_misc_ported.py`.
#[test]
fn test_filter_none_from_list() {
    let out = jit_capture(
        r#"values = [None, 1, None, 2, None]
non_none = [v for v in values if v is not None]
print(non_none)
just_none = [v for v in values if v is None]
print(len(just_none))
"#,
    );
    assert_output(&out, "[1, 2]\n3\n");
}

/// Ported from `Lib/test/test_misc_ported.py`.
#[test]
fn test_count_none_via_loop() {
    let out = jit_capture(
        r#"vs = [None, "a", None, "b", None, "c"]
count = 0
present = 0
for v in vs:
    if v is None:
        count = count + 1
    else:
        present = present + 1
print(count)
print(present)
"#,
    );
    assert_output(&out, "3\n3\n");
}

/// Ported from `Lib/test/test_misc_ported.py`.
#[test]
fn test_round_to_integer_uses_bankers_rounding() {
    let out = jit_capture(
        r#"print(round(3.7))
print(round(3.4))
print(round(2.5))
print(round(3.5))
"#,
    );
    assert_output(&out, "4\n3\n2\n4\n");
}

/// Ported from `Lib/test/test_misc_ported.py`.
#[test]
fn test_round_with_ndigits() {
    let out = jit_capture(
        r#"print(round(3.14159, 2))
print(round(3.14159, 4))
print(round(2.71828, 3))
"#,
    );
    assert_output(&out, "3.14\n3.1416\n2.718\n");
}

/// Ported from `Lib/test/test_misc_ported.py`.
#[test]
fn test_sum_over_int_float_range() {
    let out = jit_capture(
        r#"print(sum([1, 2, 3, 4]))
print(sum([1.5, 2.5]))
print(sum(range(10)))
print(sum([]))
"#,
    );
    assert_output(&out, "10\n4.0\n45\n0\n");
}

/// Ported from `Lib/test/test_misc_ported.py`.
#[test]
fn test_chained_less_than_in_order() {
    let out = jit_capture(
        r#"print(1 < 2 < 3)
print(1 < 3 < 2)
print(5 > 3 > 1)
"#,
    );
    assert_output(&out, "True\nFalse\nTrue\n");
}

/// Ported from `Lib/test/test_misc_ported.py`.
#[test]
fn test_chained_le_and_equality() {
    let out = jit_capture(
        r#"print(1 <= 1 <= 2)
print(1 == 1 == 1)
print(1 != 2 != 3)
"#,
    );
    assert_output(&out, "True\nTrue\nTrue\n");
}

/// Ported from `Lib/test/test_misc_ported.py`.
#[test]
fn test_chained_range_check_via_variable() {
    let out = jit_capture(
        r#"x = 5
print(0 < x < 10)
print(0 < x < 3)
print(10 > x > 0)
"#,
    );
    assert_output(&out, "True\nFalse\nTrue\n");
}

/// Ported from `Lib/test/test_misc_ported.py`.
#[test]
fn test_pickle_roundtrip_int() {
    let out = jit_capture(
        r#"import pickle
print(pickle.loads(pickle.dumps(42)) == 42)
"#,
    );
    assert_output(&out, "True\n");
}

/// Ported from `Lib/test/test_misc_ported.py`.
#[test]
fn test_pickle_roundtrip_string() {
    let out = jit_capture(
        r#"import pickle
print(pickle.loads(pickle.dumps("hello world")) == "hello world")
"#,
    );
    assert_output(&out, "True\n");
}

/// Ported from `Lib/test/test_misc_ported.py`.
#[test]
fn test_pickle_roundtrip_list() {
    let out = jit_capture(
        r#"import pickle
print(pickle.loads(pickle.dumps([1, 2, 3])) == [1, 2, 3])
"#,
    );
    assert_output(&out, "True\n");
}

/// Ported from `Lib/test/test_misc_ported.py`.
#[test]
fn test_pickle_roundtrip_tuple() {
    let out = jit_capture(
        r#"import pickle
print(pickle.loads(pickle.dumps((1, 2, 3))) == (1, 2, 3))
"#,
    );
    assert_output(&out, "True\n");
}

/// Ported from `Lib/test/test_misc_ported.py`.
#[test]
fn test_pickle_roundtrip_nested_dict() {
    let out = jit_capture(
        r#"import pickle
data = {"a": 1, "b": [2, 3], "c": {"d": 4}}
r = pickle.loads(pickle.dumps(data))
print(r["a"])
print(r["b"])
print(r["c"]["d"])
"#,
    );
    assert_output(&out, "1\n[2, 3]\n4\n");
}

/// Ported from `Lib/test/test_misc_ported.py`.
#[test]
fn test_pickle_roundtrip_bool() {
    let out = jit_capture(
        r#"import pickle
print(pickle.loads(pickle.dumps(True)) == True)
print(pickle.loads(pickle.dumps(False)) == False)
"#,
    );
    assert_output(&out, "True\nTrue\n");
}

/// Ported from `Lib/test/test_misc_ported.py`.
#[test]
fn test_pickle_roundtrip_none() {
    let out = jit_capture(
        r#"import pickle
print(pickle.loads(pickle.dumps(None)) is None)
"#,
    );
    assert_output(&out, "True\n");
}

/// Ported from `Lib/test/test_misc_ported.py`.
#[test]
fn test_pickle_roundtrip_float() {
    let out = jit_capture(
        r#"import pickle
print(pickle.loads(pickle.dumps(3.14)) == 3.14)
"#,
    );
    assert_output(&out, "True\n");
}

/// Ported from `Lib/test/test_misc_ported.py`.
#[test]
fn test_pct_format_single_conversion() {
    let out = jit_capture(
        r#"print("%d" % 42)
print("%s" % "hi")
print("%d" % -7)
"#,
    );
    assert_output(&out, "42\nhi\n-7\n");
}

/// Ported from `Lib/test/test_misc_ported.py`.
#[test]
fn test_pct_format_tuple_substitution() {
    let out = jit_capture(
        r#"print("%d + %d = %d" % (1, 2, 3))
print("name=%s age=%d" % ("Alice", 30))
"#,
    );
    assert_output(&out, "1 + 2 = 3\nname=Alice age=30\n");
}

/// Ported from `Lib/test/test_misc_ported.py`.
#[test]
fn test_pct_format_width_and_precision() {
    let out = jit_capture(
        r#"print(repr("%5d" % 7))
print(repr("%-5d" % 7))
print("%05d" % 7)
print("%.2f" % 3.14159)
"#,
    );
    assert_output(&out, "'    7'\n'7    '\n00007\n3.14\n");
}

/// Ported from `Lib/test/test_misc_ported.py`.
#[test]
fn test_chr_produces_expected_ascii_characters() {
    let out = jit_capture(
        r#"print(chr(65))
print(chr(97))
print(chr(48))
"#,
    );
    assert_output(&out, "A\na\n0\n");
}

/// Ported from `Lib/test/test_misc_ported.py`.
#[test]
fn test_ord_returns_codepoint_of_single_char() {
    let out = jit_capture(
        r#"print(ord("A"))
print(ord("a"))
print(ord("0"))
"#,
    );
    assert_output(&out, "65\n97\n48\n");
}

/// Ported from `Lib/test/test_misc_ported.py`.
#[test]
fn test_ord_chr_ascii_roundtrip() {
    let out = jit_capture(
        r#"print(ord('A'))
print(ord('z'))
print(chr(65))
print(chr(122))
print(chr(ord('A') + 1))
"#,
    );
    assert_output(&out, "65\n122\nA\nz\nB\n");
}

/// Ported from `Lib/test/test_misc_ported.py`.
#[test]
fn test_math_floor_ceil_positive_and_negative() {
    let out = jit_capture(
        r#"import math
print(math.floor(3.7))
print(math.ceil(3.2))
print(math.floor(-2.3))
print(math.ceil(-2.7))
"#,
    );
    assert_output(&out, "3\n4\n-3\n-2\n");
}

/// Ported from `Lib/test/test_misc_ported.py`.
#[test]
fn test_math_sqrt_int_and_float() {
    let out = jit_capture(
        r#"import math
print(math.sqrt(16))
print(math.sqrt(2.0))
"#,
    );
    assert_output(&out, "4.0\n1.4142135623730951\n");
}

/// Ported from `Lib/test/test_misc_ported.py`.
#[test]
fn test_filter_with_lambda_predicate() {
    let out = jit_capture(
        r#"print(list(filter(lambda x: x > 2, [1, 2, 3, 4, 5])))
print(list(filter(lambda x: x % 2 == 0, [1, 2, 3, 4, 5, 6])))
"#,
    );
    assert_output(&out, "[3, 4, 5]\n[2, 4, 6]\n");
}

/// Ported from `Lib/test/test_misc_ported.py`.
#[test]
fn test_filter_none_keeps_truthy_items() {
    let out = jit_capture(
        r#"print(list(filter(None, [0, 1, 2, 0, 3])))
print(list(filter(None, ["", "a", "", "b"])))
"#,
    );
    assert_output(&out, "[1, 2, 3]\n['a', 'b']\n");
}

/// Ported from `Lib/test/test_misc_ported.py`.
#[test]
fn test_map_with_single_iterable() {
    let out = jit_capture(
        r#"print(list(map(lambda x: x * 2, [1, 2, 3])))
print(list(map(lambda x: x + 10, [1, 2, 3])))
print(list(map(str, [1, 2, 3])))
"#,
    );
    assert_output(&out, "[2, 4, 6]\n[11, 12, 13]\n['1', '2', '3']\n");
}

/// Ported from `Lib/test/test_misc_ported.py`.
#[test]
fn test_gc_isenabled_default() {
    let out = jit_capture(
        r#"import gc
print(gc.isenabled())
"#,
    );
    assert_output(&out, "True\n");
}

/// Ported from `Lib/test/test_misc_ported.py`.
#[test]
fn test_gc_get_count_three_generations() {
    let out = jit_capture(
        r#"import gc
counts = gc.get_count()
print(len(counts))
"#,
    );
    assert_output(&out, "3\n");
}

/// Ported from `Lib/test/test_misc_ported.py`.
#[test]
fn test_gc_collect_returns_non_negative() {
    let out = jit_capture(
        r#"import gc
n = gc.collect()
print(n >= 0)
"#,
    );
    assert_output(&out, "True\n");
}

/// Ported from `Lib/test/test_misc_ported.py`.
#[test]
fn test_aug_assign_numeric_chain() {
    let out = jit_capture(
        r#"x = 10
x += 5
print(x)
x -= 3
print(x)
x *= 2
print(x)
x //= 4
print(x)
x **= 2
print(x)
"#,
    );
    assert_output(&out, "15\n12\n24\n6\n36\n");
}

/// Ported from `Lib/test/test_misc_ported.py`.
#[test]
fn test_aug_assign_list_and_str() {
    let out = jit_capture(
        r#"y = [1, 2, 3]
y += [4, 5]
print(y)
z = "ab"
z *= 3
print(z)
"#,
    );
    assert_output(&out, "[1, 2, 3, 4, 5]\nababab\n");
}

/// Ported from `Lib/test/test_misc_ported.py`.
#[test]
fn test_bitwise_and_or_xor_not() {
    let out = jit_capture(
        r#"print(0b1100 & 0b1010)
print(0b1100 | 0b1010)
print(0b1100 ^ 0b1010)
print(~5)
"#,
    );
    assert_output(&out, "8\n14\n6\n-6\n");
}

/// Ported from `Lib/test/test_misc_ported.py`.
#[test]
fn test_bitwise_shift_left_right() {
    let out = jit_capture(
        r#"print(1 << 4)
print(64 >> 2)
print(0b1 << 8)
print(0xff >> 4)
"#,
    );
    assert_output(&out, "16\n16\n256\n15\n");
}

/// Ported from `Lib/test/test_misc_ported.py`.
#[test]
fn test_chained_simple_assignment() {
    let out = jit_capture(
        r#"a = b = c = 10
print(a, b, c)
"#,
    );
    assert_output(&out, "10 10 10\n");
}

/// Ported from `Lib/test/test_misc_ported.py`.
#[test]
fn test_join_over_list_of_strings() {
    let out = jit_capture(
        r#"print(", ".join(["a", "b", "c"]))
print("-".join(["x"]))
print(" | ".join(["one", "two", "three"]))
"#,
    );
    assert_output(&out, "a, b, c\nx\none | two | three\n");
}

/// Ported from `Lib/test/test_misc_ported.py`.
#[test]
fn test_join_over_comprehension_and_generator() {
    let out = jit_capture(
        r#"print("-".join([str(x) for x in range(4)]))
print(" ".join(c for c in "hello"))
"#,
    );
    assert_output(&out, "0-1-2-3\nh e l l o\n");
}

/// Ported from `Lib/test/test_misc_ported.py`.
#[test]
fn test_join_on_empty_and_singleton() {
    let out = jit_capture(
        r#"print(repr(",".join([])))
print("x".join(["a"]))
print(repr("-".join([])))
"#,
    );
    assert_output(&out, "''\na\n''\n");
}

/// Ported from `Lib/test/test_misc_ported.py`.
#[test]
fn test_or_returns_first_truthy() {
    let out = jit_capture(
        r#"print(1 or 2)
print(0 or 5)
print(0 or 0 or 7)
"#,
    );
    assert_output(&out, "1\n5\n7\n");
}

/// Ported from `Lib/test/test_misc_ported.py`.
#[test]
fn test_and_returns_first_falsy_or_last() {
    let out = jit_capture(
        r#"print(1 and 2)
print(0 and 5)
print(3 and 4 and 5)
print(3 and 0 and 5)
"#,
    );
    assert_output(&out, "2\n0\n5\n0\n");
}

/// Ported from `Lib/test/test_misc_ported.py`.
#[test]
fn test_not_operator_flips_truthiness() {
    let out = jit_capture(
        r#"print(not True)
print(not False)
print(not 0)
print(not 1)
print(not "")
print(not "x")
"#,
    );
    assert_output(&out, "False\nTrue\nTrue\nFalse\nTrue\nFalse\n");
}

/// Ported from `Lib/test/test_misc_ported.py`.
#[test]
fn test_or_default_pattern() {
    let out = jit_capture(
        r#"name = "" or "anonymous"
print(name)
count = 0 or 10
print(count)
items = [] or [1, 2]
print(items)
"#,
    );
    assert_output(&out, "anonymous\n10\n[1, 2]\n");
}

/// Ported from `Lib/test/test_misc_ported.py`.
#[test]
fn test_format_positional_single() {
    let out = jit_capture(
        r#"print("hello {}".format("world"))
"#,
    );
    assert_output(&out, "hello world\n");
}

/// Ported from `Lib/test/test_misc_ported.py`.
#[test]
fn test_format_positional_multi() {
    let out = jit_capture(
        r#"print("{} + {} = {}".format(1, 2, 3))
"#,
    );
    assert_output(&out, "1 + 2 = 3\n");
}

/// Ported from `Lib/test/test_misc_ported.py`.
#[test]
fn test_format_named_kwarg() {
    let out = jit_capture(
        r#"print("{name}={val}".format(name="x", val=42))
"#,
    );
    assert_output(&out, "x=42\n");
}

/// Ported from `Lib/test/test_misc_ported.py`.
#[test]
fn test_format_indexed_reuse() {
    let out = jit_capture(
        r#"print("{0} {1} {0}".format("a", "b"))
"#,
    );
    assert_output(&out, "a b a\n");
}

/// Ported from `Lib/test/test_misc_ported.py`.
#[test]
fn test_try_else_runs_on_success_only() {
    let out = jit_capture(
        r#"def f(x):
    try:
        y = 100 // x
    except ZeroDivisionError:
        return "div by zero"
    else:
        return f"ok {y}"
print(f(0))
print(f(5))
print(f(10))
"#,
    );
    assert_output(&out, "div by zero\nok 20\nok 10\n");
}

/// Ported from `Lib/test/test_misc_ported.py`.
#[test]
fn test_for_else_skipped_on_break() {
    let out = jit_capture(
        r#"for i in range(3):
    if i == 1:
        break
    print(i)
else:
    print("not reached")
print("after")
"#,
    );
    assert_output(&out, "0\nafter\n");
}

/// Ported from `Lib/test/test_misc_ported.py`.
#[test]
fn test_while_else_runs_after_natural_exit() {
    let out = jit_capture(
        r#"i = 0
while i < 3:
    print(i)
    i += 1
else:
    print("while done")
"#,
    );
    assert_output(&out, "0\n1\n2\nwhile done\n");
}

/// Ported from `Lib/test/test_misc_ported.py`.
#[test]
fn test_parallel_assignment_and_swap() {
    let out = jit_capture(
        r#"a, b = 1, 2
print(a, b)
a, b = b, a
print(a, b)
"#,
    );
    assert_output(&out, "1 2\n2 1\n");
}

/// Ported from `Lib/test/test_misc_ported.py`.
#[test]
fn test_unpack_list_into_named_targets() {
    let out = jit_capture(
        r#"x, y, z = [10, 20, 30]
print(x, y, z)
p, q = (100, 200)
print(p, q)
"#,
    );
    assert_output(&out, "10 20 30\n100 200\n");
}

/// Ported from `Lib/test/test_misc_ported.py`.
#[test]
fn test_starred_target_front_middle_end() {
    let out = jit_capture(
        r#"first, *rest = [1, 2, 3, 4, 5]
print(first, rest)
*init, last = [1, 2, 3, 4, 5]
print(init, last)
a, *middle, b = [1, 2, 3, 4, 5]
print(a, middle, b)
"#,
    );
    assert_output(&out, "1 [2, 3, 4, 5]\n[1, 2, 3, 4] 5\n1 [2, 3, 4] 5\n");
}

/// Ported from `Lib/test/test_misc_ported.py`.
#[test]
fn test_max_min_iterable_and_args() {
    let out = jit_capture(
        r#"print(max([3, 1, 4, 1, 5, 9, 2, 6]))
print(min([3, 1, 4, 1, 5, 9, 2, 6]))
print(max("hello"))
print(min("hello"))
print(max(-3, -1, -2))
print(min(-3, -1, -2))
"#,
    );
    assert_output(&out, "9\n1\no\ne\n-1\n-3\n");
}

/// Ported from `Lib/test/test_misc_ported.py`.
#[test]
fn test_max_min_default_on_empty() {
    let out = jit_capture(
        r#"print(max([], default=-1))
print(min([], default=99))
"#,
    );
    assert_output(&out, "-1\n99\n");
}

/// Ported from `Lib/test/test_misc_ported.py`.
#[test]
fn test_max_min_with_key_callable() {
    let out = jit_capture(
        r#"print(max([(1, 9), (2, 4), (3, 7)], key=lambda x: x[1]))
print(min([(1, 9), (2, 4), (3, 7)], key=lambda x: x[1]))
"#,
    );
    assert_output(&out, "(1, 9)\n(2, 4)\n");
}

/// Ported from `Lib/test/test_misc_ported.py`.
#[test]
fn test_sorted_with_reverse_and_key() {
    let out = jit_capture(
        r#"xs = [3, 1, 4, 1, 5, 9, 2, 6]
print(sorted(xs))
print(sorted(xs, reverse=True))

words = ["banana", "apple", "cherry"]
print(sorted(words, key=len))
print(sorted(words, key=len, reverse=True))
"#,
    );
    assert_output(
        &out,
        "[1, 1, 2, 3, 4, 5, 6, 9]\n[9, 6, 5, 4, 3, 2, 1, 1]\n['apple', 'banana', 'cherry']\n['banana', 'cherry', 'apple']\n",
    );
}

/// Ported from `Lib/test/test_misc_ported.py`.
#[test]
fn test_reversed_over_list_and_range() {
    let out = jit_capture(
        r#"print(list(reversed([1, 2, 3, 4, 5])))
print(list(reversed(range(5))))
"#,
    );
    assert_output(&out, "[5, 4, 3, 2, 1]\n[4, 3, 2, 1, 0]\n");
}

/// Ported from `Lib/test/test_misc_ported.py`.
#[test]
fn test_list_inplace_sort_and_reverse() {
    let out = jit_capture(
        r#"ys = [5, 2, 8, 1]
ys.sort()
print(ys)
ys.sort(reverse=True)
print(ys)

zs = [1, 2, 3, 4]
zs.reverse()
print(zs)
"#,
    );
    assert_output(&out, "[1, 2, 5, 8]\n[8, 5, 2, 1]\n[4, 3, 2, 1]\n");
}

/// Ported from `Lib/test/test_misc_ported.py`.
#[test]
fn test_zip_lockstep_iteration_prints_pairs() {
    let out = jit_capture(
        r#"a = [1, 2, 3]
b = ['x', 'y', 'z']
for i, c in zip(a, b):
    print(i, c)
"#,
    );
    assert_output(&out, "1 x\n2 y\n3 z\n");
}

/// Ported from `Lib/test/test_misc_ported.py`.
#[test]
fn test_zip_to_list_of_tuples() {
    let out = jit_capture(
        r#"print(list(zip([1, 2, 3], [4, 5, 6])))
"#,
    );
    assert_output(&out, "[(1, 4), (2, 5), (3, 6)]\n");
}

/// Ported from `Lib/test/test_misc_ported.py`.
#[test]
fn test_zip_truncates_to_shortest() {
    let out = jit_capture(
        r#"print(list(zip([1, 2, 3], [4, 5])))
"#,
    );
    assert_output(&out, "[(1, 4), (2, 5)]\n");
}

/// Ported from `Lib/test/test_misc_ported.py`.
#[test]
fn test_lambda_binary_value() {
    let out = jit_capture(
        r#"add = lambda x, y: x + y
print(add(3, 4))
"#,
    );
    assert_output(&out, "7\n");
}

/// Ported from `Lib/test/test_misc_ported.py`.
#[test]
fn test_lambda_with_map() {
    let out = jit_capture(
        r#"nums = [1, 2, 3, 4]
print(list(map(lambda x: x * 10, nums)))
"#,
    );
    assert_output(&out, "[10, 20, 30, 40]\n");
}

/// Ported from `Lib/test/test_misc_ported.py`.
#[test]
fn test_lambda_with_filter_even() {
    let out = jit_capture(
        r#"nums = [1, 2, 3, 4, 5, 6]
print(list(filter(lambda x: x % 2 == 0, nums)))
"#,
    );
    assert_output(&out, "[2, 4, 6]\n");
}

/// Ported from `Lib/test/test_misc_ported.py`.
#[test]
fn test_lambda_as_sort_key() {
    let out = jit_capture(
        r#"pairs = [(1, "b"), (3, "a"), (2, "c")]
pairs.sort(key=lambda p: p[1])
print(pairs)
"#,
    );
    assert_output(&out, "[(3, 'a'), (1, 'b'), (2, 'c')]\n");
}

/// Ported from `Lib/test/test_misc_ported.py`.
#[test]
fn test_list_copy_independent_of_original() {
    let out = jit_capture(
        r#"a = [1, 2, 3]
b = a.copy()
b.append(4)
print(a)
print(b)
"#,
    );
    assert_output(&out, "[1, 2, 3]\n[1, 2, 3, 4]\n");
}

/// Ported from `Lib/test/test_misc_ported.py`.
#[test]
fn test_dict_copy_independent_of_original() {
    let out = jit_capture(
        r#"d1 = {"x": 1, "y": 2}
d2 = d1.copy()
d2["z"] = 3
print(sorted(d1.items()))
print(sorted(d2.items()))
"#,
    );
    assert_output(
        &out,
        "[('x', 1), ('y', 2)]\n[('x', 1), ('y', 2), ('z', 3)]\n",
    );
}

/// Ported from `Lib/test/test_misc_ported.py`.
#[test]
fn test_set_copy_independent_of_original() {
    let out = jit_capture(
        r#"s1 = {1, 2, 3}
s2 = s1.copy()
s2.add(4)
print(sorted(s1))
print(sorted(s2))
"#,
    );
    assert_output(&out, "[1, 2, 3]\n[1, 2, 3, 4]\n");
}

/// Ported from `Lib/test/test_misc_ported.py`.
#[test]
fn test_calendar_isleap_2024() {
    let out = jit_capture(
        r#"import calendar
print(calendar.isleap(2024))
"#,
    );
    assert_output(&out, "True\n");
}

/// Ported from `Lib/test/test_misc_ported.py`.
#[test]
fn test_calendar_isleap_2023() {
    let out = jit_capture(
        r#"import calendar
print(calendar.isleap(2023))
"#,
    );
    assert_output(&out, "False\n");
}

/// Ported from `Lib/test/test_misc_ported.py`.
#[test]
fn test_calendar_isleap_century_rule() {
    let out = jit_capture(
        r#"import calendar
print(calendar.isleap(1900))
print(calendar.isleap(2000))
"#,
    );
    assert_output(&out, "False\nTrue\n");
}

/// Ported from `Lib/test/test_misc_ported.py`.
#[test]
fn test_calendar_weekday_known_date() {
    let out = jit_capture(
        r#"import calendar
print(calendar.weekday(2026, 1, 1))
"#,
    );
    assert_output(&out, "3\n");
}

/// Ported from `Lib/test/test_misc_ported.py`.
#[test]
fn test_calendar_monthrange_february_leap() {
    let out = jit_capture(
        r#"import calendar
print(calendar.monthrange(2024, 2)[1])
"#,
    );
    assert_output(&out, "29\n");
}

/// Ported from `Lib/test/test_misc_ported.py`.
#[test]
fn test_calendar_monthrange_february_nonleap() {
    let out = jit_capture(
        r#"import calendar
print(calendar.monthrange(2023, 2)[1])
"#,
    );
    assert_output(&out, "28\n");
}

/// Ported from `Lib/test/test_misc_ported.py`.
#[test]
fn test_uuid_uuid4_string_length() {
    let out = jit_capture(
        r#"import uuid
u = uuid.uuid4()
print(len(str(u)))
"#,
    );
    assert_output(&out, "36\n");
}

/// Ported from `Lib/test/test_misc_ported.py`.
#[test]
fn test_uuid_uuid4_dash_count() {
    let out = jit_capture(
        r#"import uuid
u = uuid.uuid4()
print(str(u).count("-"))
"#,
    );
    assert_output(&out, "4\n");
}

/// Ported from `Lib/test/test_misc_ported.py`.
#[test]
fn test_uuid_distinct_uuids() {
    let out = jit_capture(
        r#"import uuid
a = uuid.uuid4()
b = uuid.uuid4()
print(a != b)
"#,
    );
    assert_output(&out, "True\n");
}

/// Ported from `Lib/test/test_misc_ported.py`.
#[test]
fn test_chained_numeric_comparisons() {
    let out = jit_capture(
        r#"print(1 < 2 < 3)
print(1 < 5 < 3)
print(1 <= 1 <= 2)
print(3 > 2 > 1)
print(1 == 1 == 1)
x = 5
print(0 < x < 10)
"#,
    );
    assert_output(&out, "True\nFalse\nTrue\nTrue\nTrue\nTrue\n");
}

/// Ported from `Lib/test/test_misc_ported.py`.
#[test]
fn test_membership_in_and_not_in() {
    let out = jit_capture(
        r#"print(3 in [1, 2, 3])
print(5 in [1, 2, 3])
print("b" in "abc")
print("z" not in "abc")
print(2 in {1: "a", 2: "b"})
print(3 in (1, 2, 3))
"#,
    );
    assert_output(&out, "True\nFalse\nTrue\nTrue\nTrue\nTrue\n");
}

/// Ported from `Lib/test/test_misc_ported.py`.
#[test]
fn test_ternary_expression_in_comprehension() {
    let out = jit_capture(
        r#"x = 5
print("big" if x > 3 else "small")
nums = [1, 2, 3]
print([n * 10 if n > 1 else n for n in nums])
"#,
    );
    assert_output(&out, "big\n[1, 20, 30]\n");
}

/// Ported from `Lib/test/test_misc_ported.py`.
#[test]
fn test_unicodedata_category_uppercase_letter() {
    let out = jit_capture(
        r#"import unicodedata
print(unicodedata.category("A"))
"#,
    );
    assert_output(&out, "Lu\n");
}

/// Ported from `Lib/test/test_misc_ported.py`.
#[test]
fn test_unicodedata_category_lowercase_letter() {
    let out = jit_capture(
        r#"import unicodedata
print(unicodedata.category("a"))
"#,
    );
    assert_output(&out, "Ll\n");
}

/// Ported from `Lib/test/test_misc_ported.py`.
#[test]
fn test_unicodedata_category_decimal_digit() {
    let out = jit_capture(
        r#"import unicodedata
print(unicodedata.category("5"))
"#,
    );
    assert_output(&out, "Nd\n");
}

/// Ported from `Lib/test/test_misc_ported.py`.
#[test]
fn test_unicodedata_normalize_nfc_identity_ascii() {
    let out = jit_capture(
        r#"import unicodedata
print(unicodedata.normalize("NFC", "hello"))
"#,
    );
    assert_output(&out, "hello\n");
}

/// Ported from `Lib/test/test_misc_ported.py`.
#[test]
fn test_chr_ord_arithmetic_roundtrip() {
    let out = jit_capture(
        r#"print(chr(ord("A") + 1))
print(chr(ord("a") + 2))
print(ord(chr(100)))
"#,
    );
    assert_output(&out, "B\nc\n100\n");
}

/// Ported from `Lib/test/test_misc_ported.py`.
#[test]
fn test_true_vs_floor_division_and_mod() {
    let out = jit_capture(
        r#"print(10 / 3)
print(10 // 3)
print(10 % 3)
print(-10 // 3)
print(-10 % 3)
print(10 / 5)
print(10 // 5)
"#,
    );
    assert_output(&out, "3.3333333333333335\n3\n1\n-4\n2\n2.0\n2\n");
}

/// Ported from `Lib/test/test_misc_ported.py`.
#[test]
fn test_pow_int_and_float() {
    let out = jit_capture(
        r#"print(2 ** 10)
print(3 ** 4)
print(2 ** 0)
print(1.5 ** 2)
print(2.0 ** 3)
print(4 ** 0.5)
"#,
    );
    assert_output(&out, "1024\n81\n1\n2.25\n8.0\n2.0\n");
}

/// Ported from `Lib/test/test_misc_ported.py`.
#[test]
fn test_unary_sign_and_abs() {
    let out = jit_capture(
        r#"a = 10
b = -3.5
print(-a)
print(+a)
print(abs(-a))
print(abs(b))
print(-(-a))
print(+(-a))
"#,
    );
    assert_output(&out, "-10\n10\n10\n3.5\n10\n-10\n");
}

/// Ported from `Lib/test/test_misc_ported.py`.
#[test]
fn test_walrus_in_if_condition() {
    let out = jit_capture(
        r#"data = [1, 2, 3, 4, 5]
if (n := len(data)) > 3:
    print(f"got {n}")
"#,
    );
    assert_output(&out, "got 5\n");
}

/// Ported from `Lib/test/test_misc_ported.py`.
#[test]
fn test_walrus_in_list_literal() {
    let out = jit_capture(
        r#"print([y := 10, y + 1, y + 2])
"#,
    );
    assert_output(&out, "[10, 11, 12]\n");
}

/// Ported from `Lib/test/test_misc_ported.py`.
#[test]
fn test_walrus_in_while_drain_index() {
    let out = jit_capture(
        r#"nums = [1, 2, 3, 4, 5]
total = 0
i = 0
while (n := nums[i] if i < len(nums) else None) is not None:
    total += n
    i += 1
print(total)
"#,
    );
    assert_output(&out, "15\n");
}

/// Ported from `Lib/test/test_misc_ported.py`.
#[test]
fn test_walrus_captures_for_later_use() {
    let out = jit_capture(
        r#"if (x := 7 * 6) > 40:
    print(x)
print(x + 1)
"#,
    );
    assert_output(&out, "42\n43\n");
}

/// Ported from `Lib/test/test_misc_ported.py`.
#[test]
fn test_string_slice_forms() {
    let out = jit_capture(
        r#"s = "hello world"
print(s[:5])
print(s[6:])
print(s[::2])
print(s[::-1])
print(s[1:8:2])
"#,
    );
    assert_output(&out, "hello\nworld\nhlowrd\ndlrow olleh\nel o\n");
}

/// Ported from `Lib/test/test_misc_ported.py`.
#[test]
fn test_list_slice_forms() {
    let out = jit_capture(
        r#"xs = list(range(10))
print(xs[2:7])
print(xs[::3])
print(xs[::-1])
print(xs[-3:])
"#,
    );
    assert_output(
        &out,
        "[2, 3, 4, 5, 6]\n[0, 3, 6, 9]\n[9, 8, 7, 6, 5, 4, 3, 2, 1, 0]\n[7, 8, 9]\n",
    );
}

/// Ported from `Lib/test/test_misc_ported.py`.
#[test]
fn test_slice_empty_and_clamped_bounds() {
    let out = jit_capture(
        r#"xs = [10, 20, 30, 40]
print(xs[100:])
print(xs[:0])
print(xs[2:2])
print(xs[-100:])
print(xs[:100])
"#,
    );
    assert_output(&out, "[]\n[]\n[]\n[10, 20, 30, 40]\n[10, 20, 30, 40]\n");
}

/// Ported from `Lib/test/test_misc_ported.py`.
#[test]
fn test_abs_int_and_float() {
    let out = jit_capture(
        r#"print(abs(5))
print(abs(-3))
print(abs(0))
print(abs(-3.14))
print(abs(2.5))
"#,
    );
    assert_output(&out, "5\n3\n0\n3.14\n2.5\n");
}

/// Ported from `Lib/test/test_misc_ported.py`.
#[test]
fn test_divmod_positive_and_negative() {
    let out = jit_capture(
        r#"print(divmod(17, 5))
print(divmod(-17, 5))
print(divmod(20, 4))
print(divmod(0, 7))
"#,
    );
    assert_output(&out, "(3, 2)\n(-4, 3)\n(5, 0)\n(0, 0)\n");
}

/// Ported from `Lib/test/test_misc_ported.py`.
#[test]
fn test_pow_two_and_three_args() {
    let out = jit_capture(
        r#"print(pow(2, 10))
print(pow(3, 4))
print(pow(2, 8, 100))
print(pow(7, 3, 13))
print(pow(5, 0))
"#,
    );
    assert_output(&out, "1024\n81\n56\n5\n1\n");
}

/// Ported from `Lib/test/test_misc_ported.py`.
#[test]
fn test_base64_b64encode_hello() {
    let out = jit_capture(
        r#"import base64
print(base64.b64encode(b"hello").decode())
"#,
    );
    assert_output(&out, "aGVsbG8=\n");
}

/// Ported from `Lib/test/test_misc_ported.py`.
#[test]
fn test_base64_b64decode_hello() {
    let out = jit_capture(
        r#"import base64
print(base64.b64decode("aGVsbG8=").decode())
"#,
    );
    assert_output(&out, "hello\n");
}

/// Ported from `Lib/test/test_misc_ported.py`.
#[test]
fn test_base64_b64_roundtrip() {
    let out = jit_capture(
        r#"import base64
data = b"the quick brown fox"
encoded = base64.b64encode(data)
decoded = base64.b64decode(encoded)
print(decoded == data)
"#,
    );
    assert_output(&out, "True\n");
}

/// Ported from `Lib/test/test_misc_ported.py`.
#[test]
fn test_base64_b64encode_empty() {
    let out = jit_capture(
        r#"import base64
print(base64.b64encode(b"").decode())
"#,
    );
    assert_output(&out, "\n");
}

/// Ported from `Lib/test/test_misc_ported.py`.
#[test]
fn test_base64_urlsafe_roundtrip() {
    let out = jit_capture(
        r#"import base64
data = b"\xfb\xff\xff"
encoded = base64.urlsafe_b64encode(data)
print(encoded.decode())
print(base64.urlsafe_b64decode(encoded) == data)
"#,
    );
    assert_output(&out, "-___\nTrue\n");
}

/// Ported from `Lib/test/test_misc_ported.py`.
#[test]
fn test_base64_b16encode() {
    let out = jit_capture(
        r#"import base64
print(base64.b16encode(b"hi").decode())
print(base64.b16decode("6869").decode())
"#,
    );
    assert_output(&out, "6869\nhi\n");
}

/// Ported from `Lib/test/test_misc_ported.py`.
#[test]
fn test_elif_chain_picks_first_match() {
    let out = jit_capture(
        r#"x = 5
if x > 10:
    print("big")
elif x > 3:
    print("medium")
else:
    print("small")
"#,
    );
    assert_output(&out, "medium\n");
}

/// Ported from `Lib/test/test_misc_ported.py`.
#[test]
fn test_else_runs_when_no_branch_matches() {
    let out = jit_capture(
        r#"if False:
    print("no")
else:
    print("else")
if True:
    print("yes")
"#,
    );
    assert_output(&out, "else\nyes\n");
}

/// Ported from `Lib/test/test_misc_ported.py`.
#[test]
fn test_conditional_expression_ternary() {
    let out = jit_capture(
        r#"score = 85
grade = "A" if score >= 90 else "B" if score >= 80 else "C"
print(grade)
print("yes" if 1 else "no")
print("yes" if 0 else "no")
"#,
    );
    assert_output(&out, "B\nyes\nno\n");
}

/// Ported from `Lib/test/test_misc_ported.py`.
#[test]
fn test_slice_list_basic() {
    let out = jit_capture(
        r#"a = [1, 2, 3, 4, 5]
print(a[1:4])
"#,
    );
    assert_output(&out, "[2, 3, 4]\n");
}

/// Ported from `Lib/test/test_misc_ported.py`.
#[test]
fn test_slice_list_default_start() {
    let out = jit_capture(
        r#"a = [1, 2, 3, 4, 5]
print(a[:3])
"#,
    );
    assert_output(&out, "[1, 2, 3]\n");
}

/// Ported from `Lib/test/test_misc_ported.py`.
#[test]
fn test_slice_list_default_stop() {
    let out = jit_capture(
        r#"a = [1, 2, 3, 4, 5]
print(a[2:])
"#,
    );
    assert_output(&out, "[3, 4, 5]\n");
}

/// Ported from `Lib/test/test_misc_ported.py`.
#[test]
fn test_slice_list_full_copy() {
    let out = jit_capture(
        r#"a = [1, 2, 3]
print(a[:])
"#,
    );
    assert_output(&out, "[1, 2, 3]\n");
}

/// Ported from `Lib/test/test_misc_ported.py`.
#[test]
fn test_slice_list_negative_start() {
    let out = jit_capture(
        r#"a = [1, 2, 3, 4, 5]
print(a[-3:])
"#,
    );
    assert_output(&out, "[3, 4, 5]\n");
}

/// Ported from `Lib/test/test_misc_ported.py`.
#[test]
fn test_slice_list_negative_stop() {
    let out = jit_capture(
        r#"a = [1, 2, 3, 4, 5]
print(a[:-2])
"#,
    );
    assert_output(&out, "[1, 2, 3]\n");
}

/// Ported from `Lib/test/test_misc_ported.py`.
#[test]
fn test_slice_list_negative_both() {
    let out = jit_capture(
        r#"a = [1, 2, 3, 4, 5]
print(a[-4:-1])
"#,
    );
    assert_output(&out, "[2, 3, 4]\n");
}

/// Ported from `Lib/test/test_misc_ported.py`.
#[test]
fn test_slice_list_empty_when_start_ge_stop() {
    let out = jit_capture(
        r#"a = [1, 2, 3]
print(a[2:1])
print(a[3:3])
"#,
    );
    assert_output(&out, "[]\n[]\n");
}

/// Ported from `Lib/test/test_misc_ported.py`.
#[test]
fn test_slice_list_step_positive() {
    let out = jit_capture(
        r#"a = [1, 2, 3, 4, 5, 6]
print(a[::2])
"#,
    );
    assert_output(&out, "[1, 3, 5]\n");
}

/// Ported from `Lib/test/test_misc_ported.py`.
#[test]
fn test_slice_list_step_with_bounds() {
    let out = jit_capture(
        r#"a = [1, 2, 3, 4, 5, 6, 7, 8]
print(a[1:7:2])
"#,
    );
    assert_output(&out, "[2, 4, 6]\n");
}

/// Ported from `Lib/test/test_misc_ported.py`.
#[test]
fn test_slice_list_step_negative_full_reverse() {
    let out = jit_capture(
        r#"a = [1, 2, 3, 4, 5]
print(a[::-1])
"#,
    );
    assert_output(&out, "[5, 4, 3, 2, 1]\n");
}

/// Ported from `Lib/test/test_misc_ported.py`.
#[test]
fn test_slice_list_step_negative_with_bounds() {
    let out = jit_capture(
        r#"a = [1, 2, 3, 4, 5]
print(a[4:1:-1])
"#,
    );
    assert_output(&out, "[5, 4, 3]\n");
}

/// Ported from `Lib/test/test_misc_ported.py`.
#[test]
fn test_slice_list_out_of_range_start() {
    let out = jit_capture(
        r#"a = [1, 2, 3]
print(a[10:])
"#,
    );
    assert_output(&out, "[]\n");
}

/// Ported from `Lib/test/test_misc_ported.py`.
#[test]
fn test_slice_list_out_of_range_stop_clamps() {
    let out = jit_capture(
        r#"a = [1, 2, 3]
print(a[:100])
"#,
    );
    assert_output(&out, "[1, 2, 3]\n");
}

/// Ported from `Lib/test/test_misc_ported.py`.
#[test]
fn test_slice_str_basic() {
    let out = jit_capture(
        r#"s = "hello"
print(s[1:4])
"#,
    );
    assert_output(&out, "ell\n");
}

/// Ported from `Lib/test/test_misc_ported.py`.
#[test]
fn test_slice_str_default_bounds() {
    let out = jit_capture(
        r#"s = "hello"
print(s[:3])
print(s[2:])
print(s[:])
"#,
    );
    assert_output(&out, "hel\nllo\nhello\n");
}

/// Ported from `Lib/test/test_misc_ported.py`.
#[test]
fn test_slice_str_negative() {
    let out = jit_capture(
        r#"s = "hello"
print(s[-3:])
print(s[:-2])
"#,
    );
    assert_output(&out, "llo\nhel\n");
}

/// Ported from `Lib/test/test_misc_ported.py`.
#[test]
fn test_slice_str_step() {
    let out = jit_capture(
        r#"s = "abcdef"
print(s[::2])
"#,
    );
    assert_output(&out, "ace\n");
}

/// Ported from `Lib/test/test_misc_ported.py`.
#[test]
fn test_slice_str_reverse() {
    let out = jit_capture(
        r#"s = "hello"
print(s[::-1])
"#,
    );
    assert_output(&out, "olleh\n");
}

/// Ported from `Lib/test/test_misc_ported.py`.
#[test]
fn test_slice_str_empty() {
    let out = jit_capture(
        r#"s = "hello"
print(s[2:2])
print(len(s[2:2]))
"#,
    );
    assert_output(&out, "\n0\n");
}

/// Ported from `Lib/test/test_misc_ported.py`.
#[test]
fn test_slice_tuple_basic() {
    let out = jit_capture(
        r#"t = (1, 2, 3, 4, 5)
print(t[1:4])
"#,
    );
    assert_output(&out, "(2, 3, 4)\n");
}

/// Ported from `Lib/test/test_misc_ported.py`.
#[test]
fn test_slice_tuple_reverse() {
    let out = jit_capture(
        r#"t = (1, 2, 3, 4, 5)
print(t[::-1])
"#,
    );
    assert_output(&out, "(5, 4, 3, 2, 1)\n");
}

/// Ported from `Lib/test/test_misc_ported.py`.
#[test]
fn test_slice_tuple_negative() {
    let out = jit_capture(
        r#"t = (1, 2, 3, 4, 5)
print(t[-3:])
"#,
    );
    assert_output(&out, "(3, 4, 5)\n");
}

/// Ported from `Lib/test/test_misc_ported.py`.
#[test]
fn test_slice_list_assignment_basic() {
    let out = jit_capture(
        r#"a = [1, 2, 3, 4, 5]
a[1:3] = [20, 30, 40]
print(a)
"#,
    );
    assert_output(&out, "[1, 20, 30, 40, 4, 5]\n");
}

/// Ported from `Lib/test/test_misc_ported.py`.
#[test]
fn test_slice_list_assignment_empty_replacement() {
    let out = jit_capture(
        r#"a = [1, 2, 3, 4, 5]
a[1:3] = []
print(a)
"#,
    );
    assert_output(&out, "[1, 4, 5]\n");
}

/// Ported from `Lib/test/test_misc_ported.py`.
#[test]
fn test_slice_list_yields_new_object() {
    let out = jit_capture(
        r#"a = [1, 2, 3]
b = a[:]
b.append(4)
print(a)
print(b)
"#,
    );
    assert_output(&out, "[1, 2, 3]\n[1, 2, 3, 4]\n");
}

/// Ported from `Lib/test/test_misc_ported.py`.
#[test]
fn test_bisect_left_insertion_point() {
    let out = jit_capture(
        r#"import bisect
print(bisect.bisect_left([1, 3, 5, 7, 9], 4))
"#,
    );
    assert_output(&out, "2\n");
}

/// Ported from `Lib/test/test_misc_ported.py`.
#[test]
fn test_bisect_right_insertion_point() {
    let out = jit_capture(
        r#"import bisect
print(bisect.bisect_right([1, 3, 5, 7, 9], 5))
"#,
    );
    assert_output(&out, "3\n");
}

/// Ported from `Lib/test/test_misc_ported.py`.
#[test]
fn test_bisect_insort_maintains_order() {
    let out = jit_capture(
        r#"import bisect
xs = [1, 3, 5, 7, 9]
bisect.insort(xs, 4)
print(xs)
"#,
    );
    assert_output(&out, "[1, 3, 4, 5, 7, 9]\n");
}

/// Ported from `Lib/test/test_misc_ported.py`.
#[test]
fn test_bisect_insort_left_duplicates() {
    let out = jit_capture(
        r#"import bisect
xs = [1, 2, 3, 4]
bisect.insort_left(xs, 2)
print(xs)
"#,
    );
    assert_output(&out, "[1, 2, 2, 3, 4]\n");
}

/// Ported from `Lib/test/test_misc_ported.py`.
#[test]
fn test_bisect_bisect_left_boundaries() {
    let out = jit_capture(
        r#"import bisect
print(bisect.bisect_left([1, 2, 3], 0))
print(bisect.bisect_left([1, 2, 3], 4))
"#,
    );
    assert_output(&out, "0\n3\n");
}

/// Ported from `Lib/test/test_misc_ported.py`.
#[test]
fn test_secrets_token_hex_length() {
    let out = jit_capture(
        r#"import secrets
print(len(secrets.token_hex(8)))
print(len(secrets.token_hex(16)))
"#,
    );
    assert_output(&out, "16\n32\n");
}

/// Ported from `Lib/test/test_misc_ported.py`.
#[test]
fn test_secrets_randbelow_range() {
    let out = jit_capture(
        r#"import secrets
v = secrets.randbelow(100)
print(0 <= v < 100)
"#,
    );
    assert_output(&out, "True\n");
}

/// Ported from `Lib/test/test_misc_ported.py`.
#[test]
fn test_secrets_choice_from_population() {
    let out = jit_capture(
        r#"import secrets
v = secrets.choice([1, 2, 3, 4, 5])
print(v in [1, 2, 3, 4, 5])
"#,
    );
    assert_output(&out, "True\n");
}

/// Ported from `Lib/test/test_misc_ported.py`.
#[test]
fn test_bitwise_and_or_xor() {
    let out = jit_capture(
        r#"print(5 & 3)
print(5 | 3)
print(5 ^ 3)
"#,
    );
    assert_output(&out, "1\n7\n6\n");
}

/// Ported from `Lib/test/test_misc_ported.py`.
#[test]
fn test_bitwise_not_negates_plus_one() {
    let out = jit_capture(
        r#"print(~5)
print(~0)
print(~-1)
"#,
    );
    assert_output(&out, "-6\n-1\n0\n");
}

/// Ported from `Lib/test/test_misc_ported.py`.
#[test]
fn test_bitwise_shifts() {
    let out = jit_capture(
        r#"print(1 << 4)
print(16 >> 2)
print(1 << 0)
print(255 >> 4)
"#,
    );
    assert_output(&out, "16\n4\n1\n15\n");
}

/// Ported from `Lib/test/test_misc_ported.py`.
#[test]
fn test_bitwise_with_binary_and_hex_literals() {
    let out = jit_capture(
        r#"print(0b1010 | 0b0101)
print(0xff & 0x0f)
print(0b1100 ^ 0b1010)
"#,
    );
    assert_output(&out, "15\n15\n6\n");
}

/// Ported from `Lib/test/test_misc_ported.py`.
#[test]
fn test_heapq_heapify_min_heap_property() {
    let out = jit_capture(
        r#"import heapq
h = [9, 5, 7, 3, 1]
heapq.heapify(h)
print(h[0])
"#,
    );
    assert_output(&out, "1\n");
}

/// Ported from `Lib/test/test_misc_ported.py`.
#[test]
fn test_heapq_heappop_ascending_order() {
    let out = jit_capture(
        r#"import heapq
h = [3, 1, 4, 1, 5, 9, 2, 6]
heapq.heapify(h)
out = []
while h:
    out.append(heapq.heappop(h))
print(out)
"#,
    );
    assert_output(&out, "[1, 1, 2, 3, 4, 5, 6, 9]\n");
}

/// Ported from `Lib/test/test_misc_ported.py`.
#[test]
fn test_heapq_heappush_maintains_heap() {
    let out = jit_capture(
        r#"import heapq
h = []
for x in [5, 2, 8, 1, 9, 3]:
    heapq.heappush(h, x)
print(heapq.heappop(h))
print(heapq.heappop(h))
print(heapq.heappop(h))
"#,
    );
    assert_output(&out, "1\n2\n3\n");
}

/// Ported from `Lib/test/test_misc_ported.py`.
#[test]
fn test_heapq_heappushpop_smaller_than_min() {
    let out = jit_capture(
        r#"import heapq
print(heapq.heappushpop([1, 2, 3], 0))
"#,
    );
    assert_output(&out, "0\n");
}

/// Ported from `Lib/test/test_misc_ported.py`.
#[test]
fn test_heapq_heapreplace_returns_old_min() {
    let out = jit_capture(
        r#"import heapq
h = [1, 2, 3]
print(heapq.heapreplace(h, 5))
print(h[0])
"#,
    );
    assert_output(&out, "1\n2\n");
}

/// Ported from `Lib/test/test_misc_ported.py`.
#[test]
fn test_heapq_nsmallest_three() {
    let out = jit_capture(
        r#"import heapq
print(heapq.nsmallest(3, [5, 1, 9, 2, 8, 4]))
"#,
    );
    assert_output(&out, "[1, 2, 4]\n");
}

/// Ported from `Lib/test/test_misc_ported.py`.
#[test]
fn test_heapq_nlargest_three() {
    let out = jit_capture(
        r#"import heapq
print(heapq.nlargest(3, [5, 1, 9, 2, 8, 4]))
"#,
    );
    assert_output(&out, "[9, 8, 5]\n");
}

/// Ported from `Lib/test/test_misc_ported.py`.
#[test]
fn test_zlib_compress_decompress_roundtrip() {
    let out = jit_capture(
        r#"import zlib
data = b"hello world"
c = zlib.compress(data)
print(zlib.decompress(c) == data)
"#,
    );
    assert_output(&out, "True\n");
}

/// Ported from `Lib/test/test_misc_ported.py`.
#[test]
fn test_zlib_crc32_known_value() {
    let out = jit_capture(
        r#"import zlib
print(zlib.crc32(b"hello"))
"#,
    );
    assert_output(&out, "907060870\n");
}

/// Ported from `Lib/test/test_misc_ported.py`.
#[test]
fn test_zlib_decompress_handles_repeated() {
    let out = jit_capture(
        r#"import zlib
data = b"abc" * 100
c = zlib.compress(data)
print(zlib.decompress(c) == data)
print(len(c) < len(data))
"#,
    );
    assert_output(&out, "True\nTrue\n");
}

/// Ported from `Lib/test/test_misc_ported.py`.
#[test]
fn test_any_over_list_inputs() {
    let out = jit_capture(
        r#"print(any([False, False, True]))
print(any([False, False, False]))
print(any([0, 0, 1]))
print(any([0, 0, 0]))
"#,
    );
    assert_output(&out, "True\nFalse\nTrue\nFalse\n");
}

/// Ported from `Lib/test/test_misc_ported.py`.
#[test]
fn test_all_over_list_inputs() {
    let out = jit_capture(
        r#"print(all([True, True, True]))
print(all([True, False, True]))
print(all([1, 2, 3]))
print(all([1, 0, 3]))
"#,
    );
    assert_output(&out, "True\nFalse\nTrue\nFalse\n");
}

/// Ported from `Lib/test/test_misc_ported.py`.
#[test]
fn test_any_all_on_empty_and_generators() {
    let out = jit_capture(
        r#"print(any([]))
print(all([]))
print(any(x > 5 for x in [1, 2, 3]))
print(any(x > 5 for x in [1, 6, 3]))
print(all(x > 0 for x in [1, 2, 3]))
"#,
    );
    assert_output(&out, "False\nTrue\nFalse\nTrue\nTrue\n");
}

/// Ported from `Lib/test/test_misc_ported.py`.
#[test]
fn test_queue_fifo_order() {
    let out = jit_capture(
        r#"import queue
q = queue.Queue()
q.put(1)
q.put(2)
q.put(3)
print(q.get())
print(q.get())
print(q.get())
"#,
    );
    assert_output(&out, "1\n2\n3\n");
}

/// Ported from `Lib/test/test_misc_ported.py`.
#[test]
fn test_queue_qsize_tracks_inserts() {
    let out = jit_capture(
        r#"import queue
q = queue.Queue()
q.put(1)
q.put(2)
q.put(3)
print(q.qsize())
q.get()
print(q.qsize())
"#,
    );
    assert_output(&out, "3\n2\n");
}

/// Ported from `Lib/test/test_misc_ported.py`.
#[test]
fn test_queue_empty_state_transitions() {
    let out = jit_capture(
        r#"import queue
q = queue.Queue()
print(q.empty())
q.put("a")
print(q.empty())
q.get()
print(q.empty())
"#,
    );
    assert_output(&out, "True\nFalse\nTrue\n");
}

/// Ported from `Lib/test/test_misc_ported.py`.
#[test]
fn test_bytearray_empty_constructor() {
    let out = jit_capture(
        r#"b = bytearray()
print(len(b))
"#,
    );
    assert_output(&out, "0\n");
}

/// Ported from `Lib/test/test_misc_ported.py`.
#[test]
fn test_bytearray_from_bytes() {
    let out = jit_capture(
        r#"b = bytearray(b"hello")
print(len(b))
print(b.decode())
"#,
    );
    assert_output(&out, "5\nhello\n");
}

/// Ported from `Lib/test/test_misc_ported.py`.
#[test]
fn test_bytearray_from_int_zero_filled() {
    let out = jit_capture(
        r#"b = bytearray(5)
print(len(b))
print(b[0])
print(b[4])
"#,
    );
    assert_output(&out, "5\n0\n0\n");
}

/// Ported from `Lib/test/test_misc_ported.py`.
#[test]
fn test_bytearray_from_list_of_ints() {
    let out = jit_capture(
        r#"b = bytearray([97, 98, 99])
print(len(b))
print(b.decode())
"#,
    );
    assert_output(&out, "3\nabc\n");
}

/// Ported from `Lib/test/test_misc_ported.py`.
#[test]
fn test_bytearray_indexing_returns_int() {
    let out = jit_capture(
        r#"b = bytearray(b"abc")
print(b[0])
print(b[1])
print(b[2])
"#,
    );
    assert_output(&out, "97\n98\n99\n");
}

/// Ported from `Lib/test/test_misc_ported.py`.
#[test]
fn test_bytearray_indexing_negative() {
    let out = jit_capture(
        r#"b = bytearray(b"abc")
print(b[-1])
print(b[-3])
"#,
    );
    assert_output(&out, "99\n97\n");
}

/// Ported from `Lib/test/test_misc_ported.py`.
#[test]
fn test_bytearray_decode_utf8() {
    let out = jit_capture(
        r#"b = bytearray(b"hello")
print(b.decode())
print(b.decode("utf-8"))
"#,
    );
    assert_output(&out, "hello\nhello\n");
}

/// Ported from `Lib/test/test_misc_ported.py`.
#[test]
fn test_bytearray_equality() {
    let out = jit_capture(
        r#"print(bytearray(b"abc") == bytearray(b"abc"))
print(bytearray(b"abc") == bytearray(b"xyz"))
print(bytearray() == bytearray())
"#,
    );
    assert_output(&out, "True\nFalse\nTrue\n");
}

/// Ported from `Lib/test/test_misc_ported.py`.
#[test]
fn test_bytearray_bool_empty_is_false() {
    let out = jit_capture(
        r#"print(bool(bytearray()))
"#,
    );
    assert_output(&out, "False\n");
}

/// Ported from `Lib/test/test_misc_ported.py`.
#[test]
fn test_bytearray_bool_nonempty_is_true() {
    let out = jit_capture(
        r#"print(bool(bytearray(b"x")))
"#,
    );
    assert_output(&out, "True\n");
}

/// Ported from `Lib/test/test_misc_ported.py`.
#[test]
fn test_bytearray_contains_int() {
    let out = jit_capture(
        r#"b = bytearray(b"abc")
print(97 in b)
print(120 in b)
"#,
    );
    assert_output(&out, "True\nFalse\n");
}

/// Ported from `Lib/test/test_misc_ported.py`.
#[test]
fn test_bytearray_iteration_sum() {
    let out = jit_capture(
        r#"total = 0
for x in bytearray(b"abc"):
    total = total + x
print(total)
"#,
    );
    assert_output(&out, "294\n");
}

/// Ported from `Lib/test/test_misc_ported.py`.
#[test]
fn test_bytearray_setitem_int() {
    let out = jit_capture(
        r#"b = bytearray(b"abc")
b[0] = 120
print(b.decode())
"#,
    );
    assert_output(&out, "xbc\n");
}

/// Ported from `Lib/test/test_misc_ported.py`.
#[test]
fn test_bytearray_append() {
    let out = jit_capture(
        r#"b = bytearray(b"ab")
b.append(99)
print(len(b))
print(b.decode())
"#,
    );
    assert_output(&out, "3\nabc\n");
}

/// Ported from `Lib/test/test_misc_ported.py`.
#[test]
fn test_bytearray_extend_with_bytes() {
    let out = jit_capture(
        r#"b = bytearray(b"hello")
b.extend(b" world")
print(b.decode())
"#,
    );
    assert_output(&out, "hello world\n");
}

/// Ported from `Lib/test/test_misc_ported.py`.
#[test]
fn test_bytearray_startswith_endswith() {
    let out = jit_capture(
        r#"b = bytearray(b"hello world")
print(b.startswith(b"hello"))
print(b.endswith(b"world"))
print(b.startswith(b"world"))
"#,
    );
    assert_output(&out, "True\nTrue\nFalse\n");
}

/// Ported from `Lib/test/test_misc_ported.py`.
#[test]
fn test_bytearray_count() {
    let out = jit_capture(
        r#"b = bytearray(b"banana")
print(b.count(b"a"))
"#,
    );
    assert_output(&out, "3\n");
}

/// Ported from `Lib/test/test_misc_ported.py`.
#[test]
fn test_bytearray_find_present_and_absent() {
    let out = jit_capture(
        r#"b = bytearray(b"hello world")
print(b.find(b"world"))
print(b.find(b"xyz"))
"#,
    );
    assert_output(&out, "6\n-1\n");
}

/// Ported from `Lib/test/test_misc_ported.py`.
#[test]
fn test_global_rebinds_module_name() {
    let out = jit_capture(
        r#"count = 0
def inc():
    global count
    count += 1
inc()
inc()
inc()
print(count)
"#,
    );
    assert_output(&out, "3\n");
}

/// Ported from `Lib/test/test_misc_ported.py`.
#[test]
fn test_nonlocal_rebinds_enclosing_function_name() {
    let out = jit_capture(
        r#"def outer():
    x = 10
    def inner():
        nonlocal x
        x += 5
    inner()
    inner()
    return x
print(outer())
"#,
    );
    assert_output(&out, "20\n");
}

/// Ported from `Lib/test/test_misc_ported.py`.
#[test]
fn test_closure_captures_state_across_calls() {
    let out = jit_capture(
        r#"def make_counter():
    n = 0
    def step():
        nonlocal n
        n += 1
        return n
    return step

c = make_counter()
print(c())
print(c())
print(c())
"#,
    );
    assert_output(&out, "1\n2\n3\n");
}

/// Ported from `Lib/test/test_misc_ported.py`.
#[test]
fn test_grade_table_via_elif_chain() {
    let out = jit_capture(
        r#"for s in [95, 85, 72, 65, 50]:
    if s >= 90:
        print(s, "A")
    elif s >= 80:
        print(s, "B")
    elif s >= 70:
        print(s, "C")
    elif s >= 60:
        print(s, "D")
    else:
        print(s, "F")
"#,
    );
    assert_output(&out, "95 A\n85 B\n72 C\n65 D\n50 F\n");
}

/// Ported from `Lib/test/test_misc_ported.py`.
#[test]
fn test_signed_range_classifier_via_elif() {
    let out = jit_capture(
        r#"for n in [-3, 0, 5, 42, 999]:
    if n < 0:
        print(n, "neg")
    elif n == 0:
        print(n, "zero")
    elif n < 10:
        print(n, "small")
    elif n < 100:
        print(n, "med")
    else:
        print(n, "big")
"#,
    );
    assert_output(&out, "-3 neg\n0 zero\n5 small\n42 med\n999 big\n");
}

/// Ported from `Lib/test/test_misc_ported.py`.
#[test]
fn test_three_way_elif_discriminator() {
    let out = jit_capture(
        r#"for n in [-2, -1, 0, 1, 2]:
    if n == 0:
        print(n, "zero")
    elif n % 2 == 0:
        print(n, "even")
    else:
        print(n, "odd")
"#,
    );
    assert_output(&out, "-2 even\n-1 odd\n0 zero\n1 odd\n2 even\n");
}

