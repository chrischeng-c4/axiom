use super::super::super::super::harness::*;

/// Ported from `tests/cpython/behavior/pep/634/always_equal_does_not_match_none.py`.
#[test]
fn test_gen_behavior_pep_634_always_equal_does_not_match_none() {
    let out = jit_capture(r###"# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "pep"
# lib = "634"
# dimension = "behavior"
# case = "always_equal_does_not_match_none"
# subject = "match.singleton_pattern"
# kind = "semantic"
# xfail = ""
# mem_carveout = ""
# source = ""
# status = "filled"
# ///
"""match.singleton_pattern: an __eq__ that always returns True does not let a value match the None singleton pattern"""

# An __eq__ that always returns True does not let a value match the None pattern.
class AlwaysEqual:
    def __eq__(self, other):
        return True


probe = AlwaysEqual()
matched_none = False
match probe:
    case None:
        matched_none = True
assert matched_none is False
print("always_equal_does_not_match_none OK")
"###);
    assert_output(&out, r###"always_equal_does_not_match_none OK
"###);
}

/// Ported from `tests/cpython/behavior/pep/634/as_pattern_binds_whole_and_subparts.py`.
#[test]
fn test_gen_behavior_pep_634_as_pattern_binds_whole_and_subparts() {
    let out = jit_capture(r###"# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "pep"
# lib = "634"
# dimension = "behavior"
# case = "as_pattern_binds_whole_and_subparts"
# subject = "match.as_pattern"
# kind = "semantic"
# xfail = ""
# mem_carveout = ""
# source = ""
# status = "filled"
# ///
"""match.as_pattern: an AS pattern binds the whole matched value while inner subpatterns bind too"""

# An AS pattern binds the whole matched value while subpatterns bind too.
match [1, 2]:
    case [a, b] as whole:
        pass
assert a == 1 and b == 2 and whole == [1, 2]
print("as_pattern_binds_whole_and_subparts OK")
"###);
    assert_output(&out, r###"as_pattern_binds_whole_and_subparts OK
"###);
}

/// Ported from `tests/cpython/behavior/pep/634/bare_name_is_irrefutable_capture.py`.
#[test]
fn test_gen_behavior_pep_634_bare_name_is_irrefutable_capture() {
    let out = jit_capture(r###"# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "pep"
# lib = "634"
# dimension = "behavior"
# case = "bare_name_is_irrefutable_capture"
# subject = "match.capture_pattern"
# kind = "semantic"
# xfail = ""
# mem_carveout = ""
# source = ""
# status = "filled"
# ///
"""match.capture_pattern: a bare name is an irrefutable capture: it always matches and binds across value kinds"""

# A bare name is an irrefutable capture: it always matches and binds.
def capture_all(x):
    match x:
        case got:
            return got


assert capture_all(42) == 42
assert capture_all(None) is None
assert capture_all((1, 2)) == (1, 2)
print("bare_name_is_irrefutable_capture OK")
"###);
    assert_output(&out, r###"bare_name_is_irrefutable_capture OK
"###);
}

/// Ported from `tests/cpython/behavior/pep/634/bool_literal_distinct_from_int.py`.
#[test]
fn test_gen_behavior_pep_634_bool_literal_distinct_from_int() {
    let out = jit_capture(r###"# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "pep"
# lib = "634"
# dimension = "behavior"
# case = "bool_literal_distinct_from_int"
# subject = "match.literal_pattern"
# kind = "semantic"
# xfail = ""
# mem_carveout = ""
# source = ""
# status = "filled"
# ///
"""match.literal_pattern: True matches the True literal and 1 the int literal even though 1 == True; False reaches 0-case order"""

# bool is matched by literal identity; True is not the int pattern 1.
def truthy(x):
    match x:
        case True:
            return "true-lit"
        case 1:
            return "one-lit"
    return "none"


assert truthy(True) == "true-lit"
assert truthy(1) == "one-lit"  # 1 is not True even though 1 == True

# False reaches the 0-case only when 0 is written first; case order matters.
def zeroish(x):
    match x:
        case False:
            return "false-lit"
        case 0:
            return "zero-lit"
    return "none"


assert zeroish(0) == "zero-lit"  # 0 reaches the 0-case, not the False-case
assert zeroish(False) == "false-lit"
print("bool_literal_distinct_from_int OK")
"###);
    assert_output(&out, r###"bool_literal_distinct_from_int OK
"###);
}

/// Ported from `tests/cpython/behavior/pep/634/builtin_single_capture_binds_subject.py`.
#[test]
fn test_gen_behavior_pep_634_builtin_single_capture_binds_subject() {
    let out = jit_capture(r###"# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "pep"
# lib = "634"
# dimension = "behavior"
# case = "builtin_single_capture_binds_subject"
# subject = "match.class_pattern"
# kind = "semantic"
# xfail = ""
# mem_carveout = ""
# source = ""
# status = "filled"
# ///
"""match.class_pattern: a builtin class pattern with one capture (tuple(z)) binds the subject object itself"""

# A builtin class pattern with one capture binds the subject object itself.
empty = ()
match empty:
    case tuple(z):
        pass
assert z is empty
print("builtin_single_capture_binds_subject OK")
"###);
    assert_output(&out, r###"builtin_single_capture_binds_subject OK
"###);
}

/// Ported from `tests/cpython/behavior/pep/634/builtin_single_positional_binds_whole.py`.
#[test]
fn test_gen_behavior_pep_634_builtin_single_positional_binds_whole() {
    let out = jit_capture(r###"# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "pep"
# lib = "634"
# dimension = "behavior"
# case = "builtin_single_positional_binds_whole"
# subject = "match.class_pattern"
# kind = "semantic"
# xfail = ""
# mem_carveout = ""
# source = ""
# status = "filled"
# ///
"""match.class_pattern: a single positional pattern against a builtin (bool/int/str) binds the whole subject; bool before int"""

# A single positional pattern against a builtin type binds the whole subject.
def kind(x):
    match x:
        case bool(b):
            return ("bool", b)
        case int(n):
            return ("int", n)
        case str(s):
            return ("str", s)
    return "other"


assert kind(True) == ("bool", True)  # bool checked before int
assert kind(7) == ("int", 7)
assert kind("hi") == ("str", "hi")
assert kind(1.5) == "other"
print("builtin_single_positional_binds_whole OK")
"###);
    assert_output(&out, r###"builtin_single_positional_binds_whole OK
"###);
}

/// Ported from `tests/cpython/behavior/pep/634/capture_binds_by_identity.py`.
#[test]
fn test_gen_behavior_pep_634_capture_binds_by_identity() {
    let out = jit_capture(r###"# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "pep"
# lib = "634"
# dimension = "behavior"
# case = "capture_binds_by_identity"
# subject = "match.capture_pattern"
# kind = "semantic"
# xfail = ""
# mem_carveout = ""
# source = ""
# status = "filled"
# ///
"""match.capture_pattern: a capture binds to the same object (identity), not a copy"""

# A capture binds to the same object (identity), not a copy.
src = [1, 2, 3]
match {"data": src}:
    case {"data": captured}:
        pass
assert captured is src
print("capture_binds_by_identity OK")
"###);
    assert_output(&out, r###"capture_binds_by_identity OK
"###);
}

/// Ported from `tests/cpython/behavior/pep/634/capture_vs_wildcard_binding.py`.
#[test]
fn test_gen_behavior_pep_634_capture_vs_wildcard_binding() {
    let out = jit_capture(r###"# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "pep"
# lib = "634"
# dimension = "behavior"
# case = "capture_vs_wildcard_binding"
# subject = "match.capture_pattern"
# kind = "semantic"
# xfail = ""
# mem_carveout = ""
# source = ""
# status = "filled"
# ///
"""match.capture_pattern: a capture pattern binds the subject to the name; a literal case before it takes precedence"""

# A capture pattern binds the subject; an earlier literal case takes precedence.
def label(x):
    match x:
        case 0:
            return "zero"
        case other:
            return ("captured", other)


assert label(0) == "zero"
assert label(99) == ("captured", 99)
print("capture_vs_wildcard_binding OK")
"###);
    assert_output(&out, r###"capture_vs_wildcard_binding OK
"###);
}

/// Ported from `tests/cpython/behavior/pep/634/captured_names_stay_bound_after_match.py`.
#[test]
fn test_gen_behavior_pep_634_captured_names_stay_bound_after_match() {
    let out = jit_capture(r###"# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "pep"
# lib = "634"
# dimension = "behavior"
# case = "captured_names_stay_bound_after_match"
# subject = "match.capture_pattern"
# kind = "semantic"
# xfail = ""
# mem_carveout = ""
# source = ""
# status = "filled"
# ///
"""match.capture_pattern: names captured by the matched case remain bound after the match statement"""

# Names captured by the matched case stay bound after the match statement.
match (10, 20):
    case (a, b):
        pass
assert a == 10 and b == 20
print("captured_names_stay_bound_after_match OK")
"###);
    assert_output(&out, r###"captured_names_stay_bound_after_match OK
"###);
}

/// Ported from `tests/cpython/behavior/pep/634/class_pattern_is_isinstance_check.py`.
#[test]
fn test_gen_behavior_pep_634_class_pattern_is_isinstance_check() {
    let out = jit_capture(r###"# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "pep"
# lib = "634"
# dimension = "behavior"
# case = "class_pattern_is_isinstance_check"
# subject = "match.class_pattern"
# kind = "semantic"
# xfail = ""
# mem_carveout = ""
# source = ""
# status = "filled"
# ///
"""match.class_pattern: a class pattern is an isinstance check; a subclass matches the base-class pattern"""

# A class pattern is an isinstance check; subclasses match the base pattern.
class Animal:
    pass


class Dog(Animal):
    pass


def is_animal(x):
    match x:
        case Animal():
            return True
    return False


assert is_animal(Dog()) is True
assert is_animal(object()) is False
print("class_pattern_is_isinstance_check OK")
"###);
    assert_output(&out, r###"class_pattern_is_isinstance_check OK
"###);
}

/// Ported from `tests/cpython/behavior/pep/634/class_pattern_no_args_checks_type.py`.
#[test]
fn test_gen_behavior_pep_634_class_pattern_no_args_checks_type() {
    let out = jit_capture(r###"# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "pep"
# lib = "634"
# dimension = "behavior"
# case = "class_pattern_no_args_checks_type"
# subject = "match.class_pattern"
# kind = "semantic"
# xfail = ""
# mem_carveout = ""
# source = ""
# status = "filled"
# ///
"""match.class_pattern: a class pattern with no subpatterns just checks the type; first matching case wins"""

# A class pattern with no subpatterns just checks the type; first match wins.
def first_match_wins(x):
    match x:
        case int():
            return "int"
        case object():
            return "object"


assert first_match_wins(3) == "int"
assert first_match_wins("s") == "object"
print("class_pattern_no_args_checks_type OK")
"###);
    assert_output(&out, r###"class_pattern_no_args_checks_type OK
"###);
}

/// Ported from `tests/cpython/behavior/pep/634/class_positional_and_keyword_subpatterns.py`.
#[test]
fn test_gen_behavior_pep_634_class_positional_and_keyword_subpatterns() {
    let out = jit_capture(r###"# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "pep"
# lib = "634"
# dimension = "behavior"
# case = "class_positional_and_keyword_subpatterns"
# subject = "match.class_pattern"
# kind = "semantic"
# xfail = ""
# mem_carveout = ""
# source = ""
# status = "filled"
# ///
"""match.class_pattern: positional class patterns read __match_args__; keyword subpatterns bind by attribute name"""


# Positional class patterns read __match_args__; keyword subpatterns bind by name.
class Point:
    __match_args__ = ("x", "y")

    def __init__(self, x, y):
        self.x = x
        self.y = y


def describe(p):
    match p:
        case Point(0, 0):
            return "origin"
        case Point(x=0, y=yy):
            return ("on-y", yy)
        case Point(a, b):
            return ("point", a, b)
    return "not-point"


assert describe(Point(0, 0)) == "origin"
assert describe(Point(0, 5)) == ("on-y", 5)  # keyword subpattern
assert describe(Point(1, 2)) == ("point", 1, 2)
assert describe("nope") == "not-point"
print("class_positional_and_keyword_subpatterns OK")
"###);
    assert_output(&out, r###"class_positional_and_keyword_subpatterns OK
"###);
}

/// Ported from `tests/cpython/behavior/pep/634/dotted_value_matched_by_equality.py`.
#[test]
fn test_gen_behavior_pep_634_dotted_value_matched_by_equality() {
    let out = jit_capture(r###"# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "pep"
# lib = "634"
# dimension = "behavior"
# case = "dotted_value_matched_by_equality"
# subject = "match.value_pattern"
# kind = "semantic"
# xfail = ""
# mem_carveout = ""
# source = ""
# status = "filled"
# ///
"""match.value_pattern: dotted values (enum members, class attributes) are compared by equality"""

import enum


# Dotted values (enum members, class attributes) are compared by equality.
class Color(enum.Enum):
    RED = 0
    GREEN = 1
    BLUE = 2


def name_of(c):
    match c:
        case Color.RED:
            return "red"
        case Color.GREEN:
            return "green"
        case Color.BLUE:
            return "blue"
    return "unknown"


assert name_of(Color.RED) == "red"
assert name_of(Color.BLUE) == "blue"
assert name_of(99) == "unknown"
print("dotted_value_matched_by_equality OK")
"###);
    assert_output(&out, r###"dotted_value_matched_by_equality OK
"###);
}

/// Ported from `tests/cpython/behavior/pep/634/fixed_length_sequence_requires_exact_count.py`.
#[test]
fn test_gen_behavior_pep_634_fixed_length_sequence_requires_exact_count() {
    let out = jit_capture(r###"# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "pep"
# lib = "634"
# dimension = "behavior"
# case = "fixed_length_sequence_requires_exact_count"
# subject = "match.sequence_pattern"
# kind = "semantic"
# xfail = ""
# mem_carveout = ""
# source = ""
# status = "filled"
# ///
"""match.sequence_pattern: a fixed-length sequence pattern requires an exact element count"""

# A fixed-length sequence pattern requires an exact element count.
def triple(seq):
    match seq:
        case [a, b, c]:
            return (a, b, c)
    return None


assert triple((1, 2, 3)) == (1, 2, 3)
assert triple((1, 2)) is None
assert triple((1, 2, 3, 4)) is None
print("fixed_length_sequence_requires_exact_count OK")
"###);
    assert_output(&out, r###"fixed_length_sequence_requires_exact_count OK
"###);
}

/// Ported from `tests/cpython/behavior/pep/634/guard_falls_through_when_false.py`.
#[test]
fn test_gen_behavior_pep_634_guard_falls_through_when_false() {
    let out = jit_capture(r###"# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "pep"
# lib = "634"
# dimension = "behavior"
# case = "guard_falls_through_when_false"
# subject = "match.guard_pattern"
# kind = "semantic"
# xfail = ""
# mem_carveout = ""
# source = ""
# status = "filled"
# ///
"""match.guard_pattern: guards run only after the pattern matches; a false guard falls through to the next case"""

# Guards run only after the pattern matches; a false guard falls through.
def guarded(x):
    match x:
        case n if n < 0:
            return "neg"
        case n if n == 0:
            return "zero"
        case n:
            return "pos"


assert guarded(-3) == "neg"
assert guarded(0) == "zero"
assert guarded(5) == "pos"
print("guard_falls_through_when_false OK")
"###);
    assert_output(&out, r###"guard_falls_through_when_false OK
"###);
}

/// Ported from `tests/cpython/behavior/pep/634/iterator_not_matched_left_unconsumed.py`.
#[test]
fn test_gen_behavior_pep_634_iterator_not_matched_left_unconsumed() {
    let out = jit_capture(r###"# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "pep"
# lib = "634"
# dimension = "behavior"
# case = "iterator_not_matched_left_unconsumed"
# subject = "match.sequence_pattern"
# kind = "semantic"
# xfail = ""
# mem_carveout = ""
# source = ""
# status = "filled"
# ///
"""match.sequence_pattern: an iterator is not matched by a sequence pattern and is left unconsumed"""

# An iterator is not matched by a sequence pattern, and is left unconsumed.
it = iter([1, 2, 3])
matched_seq = False
match it:
    case []:
        matched_seq = True
assert matched_seq is False
assert list(it) == [1, 2, 3]
print("iterator_not_matched_left_unconsumed OK")
"###);
    assert_output(&out, r###"iterator_not_matched_left_unconsumed OK
"###);
}

/// Ported from `tests/cpython/behavior/pep/634/literal_case_binds_nothing.py`.
#[test]
fn test_gen_behavior_pep_634_literal_case_binds_nothing() {
    let out = jit_capture(r###"# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "pep"
# lib = "634"
# dimension = "behavior"
# case = "literal_case_binds_nothing"
# subject = "match.literal_pattern"
# kind = "semantic"
# xfail = ""
# mem_carveout = ""
# source = ""
# status = "filled"
# ///
"""match.literal_pattern: a literal-only case introduces no new names into scope"""

# A literal-only case introduces no new names into scope.
def literal_case_binds_nothing():
    seen_before = set(locals())
    match 1:
        case 1 | 2 | 3:
            pass
    return set(locals()) - seen_before - {"seen_before"}


assert literal_case_binds_nothing() == set()
print("literal_case_binds_nothing OK")
"###);
    assert_output(&out, r###"literal_case_binds_nothing OK
"###);
}

/// Ported from `tests/cpython/behavior/pep/634/literal_first_match_wins.py`.
#[test]
fn test_gen_behavior_pep_634_literal_first_match_wins() {
    let out = jit_capture(r###"# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "pep"
# lib = "634"
# dimension = "behavior"
# case = "literal_first_match_wins"
# subject = "match.literal_pattern"
# kind = "semantic"
# xfail = ""
# mem_carveout = ""
# source = ""
# status = "filled"
# ///
"""match.literal_pattern: literal patterns compare by equality and the first matching case wins; str != int literal"""

# Literal patterns compare by equality; the first matching case wins.
def http_error(status):
    match status:
        case 400:
            return "bad"
        case 401 | 403 | 404:
            return "denied"
        case 418:
            return "teapot"
    return None  # no wildcard -> falls through to None


assert http_error(400) == "bad"
assert http_error(403) == "denied"
assert http_error(418) == "teapot"
assert http_error(123) is None
assert http_error("400") is None  # str does not equal int literal
print("literal_first_match_wins OK")
"###);
    assert_output(&out, r###"literal_first_match_wins OK
"###);
}

/// Ported from `tests/cpython/behavior/pep/634/mapping_abc_userdict_matched.py`.
#[test]
fn test_gen_behavior_pep_634_mapping_abc_userdict_matched() {
    let out = jit_capture(r###"# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "pep"
# lib = "634"
# dimension = "behavior"
# case = "mapping_abc_userdict_matched"
# subject = "match.mapping_pattern"
# kind = "semantic"
# xfail = ""
# mem_carveout = ""
# source = ""
# status = "filled"
# ///
"""match.mapping_pattern: a UserDict (mapping ABC) is matched, and **rest collects the other keys"""

import collections


# UserDict (mapping ABC) is also matched, and **rest collects the others.
ud = collections.UserDict({0: 1, 2: 3})
match ud:
    case {2: 3, **others}:
        pass
assert others == {0: 1}
print("mapping_abc_userdict_matched OK")
"###);
    assert_output(&out, r###"mapping_abc_userdict_matched OK
"###);
}

/// Ported from `tests/cpython/behavior/pep/634/mapping_does_not_insert_keys.py`.
#[test]
fn test_gen_behavior_pep_634_mapping_does_not_insert_keys() {
    let out = jit_capture(r###"# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "pep"
# lib = "634"
# dimension = "behavior"
# case = "mapping_does_not_insert_keys"
# subject = "match.mapping_pattern"
# kind = "semantic"
# xfail = ""
# mem_carveout = ""
# source = ""
# status = "filled"
# ///
"""match.mapping_pattern: matching a defaultdict does not auto-create missing keys; the subject is unchanged"""

import collections


# defaultdict is matched as-is; matching does NOT auto-create missing keys.
dd = collections.defaultdict(int)
match dd:
    case {0: 0}:
        which = "had-zero"
    case {**everything}:
        which = "empty"
assert which == "empty"
assert dd == {}  # matching {0: 0} did not insert key 0
print("mapping_does_not_insert_keys OK")
"###);
    assert_output(&out, r###"mapping_does_not_insert_keys OK
"###);
}

/// Ported from `tests/cpython/behavior/pep/634/mapping_matches_present_keys_extra_ok.py`.
#[test]
fn test_gen_behavior_pep_634_mapping_matches_present_keys_extra_ok() {
    let out = jit_capture(r###"# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "pep"
# lib = "634"
# dimension = "behavior"
# case = "mapping_matches_present_keys_extra_ok"
# subject = "match.mapping_pattern"
# kind = "semantic"
# xfail = ""
# mem_carveout = ""
# source = ""
# status = "filled"
# ///
"""match.mapping_pattern: a mapping pattern matches when the named keys are present; extra keys are ignored, missing keys fail"""

# A mapping pattern matches when the named keys are present; extra keys are OK.
def route(cfg):
    match cfg:
        case {"bandwidth": b, "latency": l}:
            return (b, l)
    return None


assert route({"bandwidth": 0, "latency": 1}) == (0, 1)
assert route({"bandwidth": 0, "latency": 1, "extra": 2}) == (0, 1)  # extra ignored
assert route({"bandwidth": 0}) is None  # missing required key
print("mapping_matches_present_keys_extra_ok OK")
"###);
    assert_output(&out, r###"mapping_matches_present_keys_extra_ok OK
"###);
}

/// Ported from `tests/cpython/behavior/pep/634/mapping_pattern_order_independent.py`.
#[test]
fn test_gen_behavior_pep_634_mapping_pattern_order_independent() {
    let out = jit_capture(r###"# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "pep"
# lib = "634"
# dimension = "behavior"
# case = "mapping_pattern_order_independent"
# subject = "match.mapping_pattern"
# kind = "semantic"
# xfail = ""
# mem_carveout = ""
# source = ""
# status = "filled"
# ///
"""match.mapping_pattern: mapping patterns are order-independent in both subject and pattern"""

# Mapping patterns are order-independent in both subject and pattern.
match {"latency": 1, "bandwidth": 0}:
    case {"bandwidth": b2, "latency": l2}:
        pass
assert b2 == 0 and l2 == 1
print("mapping_pattern_order_independent OK")
"###);
    assert_output(&out, r###"mapping_pattern_order_independent OK
"###);
}

/// Ported from `tests/cpython/behavior/pep/634/mapping_rest_captures_remaining_dict.py`.
#[test]
fn test_gen_behavior_pep_634_mapping_rest_captures_remaining_dict() {
    let out = jit_capture(r###"# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "pep"
# lib = "634"
# dimension = "behavior"
# case = "mapping_rest_captures_remaining_dict"
# subject = "match.mapping_pattern"
# kind = "semantic"
# xfail = ""
# mem_carveout = ""
# source = ""
# status = "filled"
# ///
"""match.mapping_pattern: **rest captures the remaining keys as a plain dict, and is the empty dict when all keys are consumed"""

# **rest captures the remaining keys as a plain dict.
match {"x": 1, "y": 2, "z": 3}:
    case {"x": x, **rest}:
        pass
assert x == 1
assert rest == {"y": 2, "z": 3}
assert type(rest) is dict

# **rest is the empty dict when all keys are consumed.
match {"only": 9}:
    case {"only": v, **leftover}:
        pass
assert v == 9 and leftover == {}
print("mapping_rest_captures_remaining_dict OK")
"###);
    assert_output(&out, r###"mapping_rest_captures_remaining_dict OK
"###);
}

/// Ported from `tests/cpython/behavior/pep/634/mapping_value_subpattern_must_match.py`.
#[test]
fn test_gen_behavior_pep_634_mapping_value_subpattern_must_match() {
    let out = jit_capture(r###"# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "pep"
# lib = "634"
# dimension = "behavior"
# case = "mapping_value_subpattern_must_match"
# subject = "match.mapping_pattern"
# kind = "semantic"
# xfail = ""
# mem_carveout = ""
# source = ""
# status = "filled"
# ///
"""match.mapping_pattern: a value subpattern in a mapping must also match the stored value or the case fails"""

# A value subpattern in a mapping must also match the stored value.
def has_yy(d):
    match d:
        case {"x": xv, "y": "yy", "z": zv}:
            return (xv, zv)
    return None


assert has_yy({"x": "x", "y": "yy", "z": "z"}) == ("x", "z")
assert has_yy({"x": "x", "y": "OTHER", "z": "z"}) is None
print("mapping_value_subpattern_must_match OK")
"###);
    assert_output(&out, r###"mapping_value_subpattern_must_match OK
"###);
}

/// Ported from `tests/cpython/behavior/pep/634/nested_as_patterns_bind_all_levels.py`.
#[test]
fn test_gen_behavior_pep_634_nested_as_patterns_bind_all_levels() {
    let out = jit_capture(r###"# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "pep"
# lib = "634"
# dimension = "behavior"
# case = "nested_as_patterns_bind_all_levels"
# subject = "match.as_pattern"
# kind = "semantic"
# xfail = ""
# mem_carveout = ""
# source = ""
# status = "filled"
# ///
"""match.as_pattern: nested as-patterns bind every intermediate value and the whole structure"""

# Nested as-patterns bind every intermediate value and the whole structure.
match ((0, 1), (2, 3)):
    case [(p as q, r) as left, (s, t) as right]:
        pass
assert p == 0 and q == 0 and r == 1 and left == (0, 1)
assert s == 2 and t == 3 and right == (2, 3)
print("nested_as_patterns_bind_all_levels OK")
"###);
    assert_output(&out, r###"nested_as_patterns_bind_all_levels OK
"###);
}

/// Ported from `tests/cpython/behavior/pep/634/no_matching_case_falls_through.py`.
#[test]
fn test_gen_behavior_pep_634_no_matching_case_falls_through() {
    let out = jit_capture(r###"# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "pep"
# lib = "634"
# dimension = "behavior"
# case = "no_matching_case_falls_through"
# subject = "match.statement"
# kind = "semantic"
# xfail = ""
# mem_carveout = ""
# source = ""
# status = "filled"
# ///
"""match.statement: a match with no matching case and no wildcard falls through without raising"""

# A match with no matching case and no wildcard falls through (no raise).
def classify(x):
    match x:
        case 1:
            return "one"
        case "two":
            return "two"
    return "unmatched"


assert classify(1) == "one"
assert classify("two") == "two"
assert classify(99) == "unmatched"
print("no_matching_case_falls_through OK")
"###);
    assert_output(&out, r###"no_matching_case_falls_through OK
"###);
}

/// Ported from `tests/cpython/behavior/pep/634/only_taken_branch_binds.py`.
#[test]
fn test_gen_behavior_pep_634_only_taken_branch_binds() {
    let out = jit_capture(r###"# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "pep"
# lib = "634"
# dimension = "behavior"
# case = "only_taken_branch_binds"
# subject = "match.capture_pattern"
# kind = "semantic"
# xfail = ""
# mem_carveout = ""
# source = ""
# status = "filled"
# ///
"""match.capture_pattern: captures from an unmatched case are not bound; only the taken branch binds names"""

# Captures from an unmatched case are NOT bound; only the taken branch binds.
def which_branch(v):
    taken = None
    match v:
        case 0:
            taken = "literal"
        case [head, *_]:
            taken = ("seq", head)
        case other:
            taken = ("capture", other)
    return taken


assert which_branch(0) == "literal"
assert which_branch([7, 8]) == ("seq", 7)
assert which_branch("z") == ("capture", "z")
print("only_taken_branch_binds OK")
"###);
    assert_output(&out, r###"only_taken_branch_binds OK
"###);
}

/// Ported from `tests/cpython/behavior/pep/634/or_pattern_binds_same_name.py`.
#[test]
fn test_gen_behavior_pep_634_or_pattern_binds_same_name() {
    let out = jit_capture(r###"# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "pep"
# lib = "634"
# dimension = "behavior"
# case = "or_pattern_binds_same_name"
# subject = "match.or_pattern"
# kind = "semantic"
# xfail = ""
# mem_carveout = ""
# source = ""
# status = "filled"
# ///
"""match.or_pattern: OR patterns may bind names; the matching alternative supplies the binding"""

# OR patterns may bind names; the matching alternative supplies the binding.
match (2, 9):
    case (0 as v) | (v, 9):
        pass
assert v == 2  # second alternative matched, bound v to the first element
print("or_pattern_binds_same_name OK")
"###);
    assert_output(&out, r###"or_pattern_binds_same_name OK
"###);
}

/// Ported from `tests/cpython/behavior/pep/634/rebind_subject_name_in_case.py`.
#[test]
fn test_gen_behavior_pep_634_rebind_subject_name_in_case() {
    let out = jit_capture(r###"# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "pep"
# lib = "634"
# dimension = "behavior"
# case = "rebind_subject_name_in_case"
# subject = "match.capture_pattern"
# kind = "semantic"
# xfail = ""
# mem_carveout = ""
# source = ""
# status = "filled"
# ///
"""match.capture_pattern: rebinding the same name as the subject (case x: from match x) is allowed and yields the value"""

# Rebinding the same name as the subject is allowed (case x: from match x).
x = 0
match x:
    case x:
        rebound = x
assert rebound == 0 and x == 0
print("rebind_subject_name_in_case OK")
"###);
    assert_output(&out, r###"rebind_subject_name_in_case OK
"###);
}

/// Ported from `tests/cpython/behavior/pep/634/sequence_matches_list_tuple_range_array_memoryview.py`.
#[test]
fn test_gen_behavior_pep_634_sequence_matches_list_tuple_range_array_memoryview() {
    let out = jit_capture(r###"# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "pep"
# lib = "634"
# dimension = "behavior"
# case = "sequence_matches_list_tuple_range_array_memoryview"
# subject = "match.sequence_pattern"
# kind = "semantic"
# xfail = ""
# mem_carveout = ""
# source = ""
# status = "filled"
# ///
"""match.sequence_pattern: a sequence pattern matches list, tuple, range, array and memoryview"""

import array


# A sequence pattern matches list, tuple, range, array and memoryview.
def first_last(seq):
    match seq:
        case [head, *_, tail]:
            return (head, tail)
        case [only]:
            return ("single", only)
        case []:
            return "empty"
    return "no-match"


assert first_last([10, 20, 30]) == (10, 30)
assert first_last((10, 20, 30)) == (10, 30)
assert first_last(range(3)) == (0, 2)
assert first_last(array.array("b", b"abc")) == (97, 99)
assert first_last(memoryview(b"abc")) == (97, 99)
assert first_last(()) == "empty"
assert first_last([5]) == ("single", 5)
print("sequence_matches_list_tuple_range_array_memoryview OK")
"###);
    assert_output(&out, r###"sequence_matches_list_tuple_range_array_memoryview OK
"###);
}

/// Ported from `tests/cpython/behavior/pep/634/star_captures_middle_as_list.py`.
#[test]
fn test_gen_behavior_pep_634_star_captures_middle_as_list() {
    let out = jit_capture(r###"# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "pep"
# lib = "634"
# dimension = "behavior"
# case = "star_captures_middle_as_list"
# subject = "match.sequence_pattern"
# kind = "semantic"
# xfail = ""
# mem_carveout = ""
# source = ""
# status = "filled"
# ///
"""match.sequence_pattern: a star target captures the middle/leading/trailing slice as a list in every position"""

# A star target captures the slice as a list, in every position.
match (0, 1, 2, 3):
    case [first, *middle, last]:
        pass
assert first == 0 and last == 3 and middle == [1, 2]

match (0, 1, 2):
    case [*rest, 2]:
        pass
assert rest == [0, 1]

match (0, 1, 2):
    case [0, 1, 2, *tailrest]:
        pass
assert tailrest == []
print("star_captures_middle_as_list OK")
"###);
    assert_output(&out, r###"star_captures_middle_as_list OK
"###);
}

/// Ported from `tests/cpython/behavior/pep/634/str_and_bytes_not_sequence_patterns.py`.
#[test]
fn test_gen_behavior_pep_634_str_and_bytes_not_sequence_patterns() {
    let out = jit_capture(r###"# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "pep"
# lib = "634"
# dimension = "behavior"
# case = "str_and_bytes_not_sequence_patterns"
# subject = "match.sequence_pattern"
# kind = "semantic"
# xfail = ""
# mem_carveout = ""
# source = ""
# status = "filled"
# ///
"""match.sequence_pattern: str and bytes are not treated as sequence patterns"""

# str and bytes are NOT treated as sequence patterns.
def as_seq(seq):
    match seq:
        case [head, *_, tail]:
            return (head, tail)
    return "no-match"


assert as_seq("abc") == "no-match"
assert as_seq(b"abc") == "no-match"
assert as_seq([1, 2, 3]) == (1, 3)  # a real sequence still matches
print("str_and_bytes_not_sequence_patterns OK")
"###);
    assert_output(&out, r###"str_and_bytes_not_sequence_patterns OK
"###);
}

/// Ported from `tests/cpython/behavior/pep/634/walrus_in_guard_binds.py`.
#[test]
fn test_gen_behavior_pep_634_walrus_in_guard_binds() {
    let out = jit_capture(r###"# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "pep"
# lib = "634"
# dimension = "behavior"
# case = "walrus_in_guard_binds"
# subject = "match.guard_pattern"
# kind = "semantic"
# xfail = ""
# mem_carveout = ""
# source = ""
# status = "filled"
# ///
"""match.guard_pattern: a walrus assignment inside a guard binds and is visible in the case body"""

# A walrus assignment inside a guard binds and is visible in the case body.
match 7:
    case n if (doubled := n * 2):
        result = doubled
assert result == 14
print("walrus_in_guard_binds OK")
"###);
    assert_output(&out, r###"walrus_in_guard_binds OK
"###);
}

/// Ported from `tests/cpython/behavior/pep/634/wildcard_catches_anything.py`.
#[test]
fn test_gen_behavior_pep_634_wildcard_catches_anything() {
    let out = jit_capture(r###"# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "pep"
# lib = "634"
# dimension = "behavior"
# case = "wildcard_catches_anything"
# subject = "match.wildcard_pattern"
# kind = "semantic"
# xfail = ""
# mem_carveout = ""
# source = ""
# status = "filled"
# ///
"""match.wildcard_pattern: the wildcard pattern (case _) matches any unmatched subject"""

# The wildcard pattern matches any unmatched subject.
def with_wildcard(x):
    match x:
        case 1:
            return "one"
        case _:
            return "other"


assert with_wildcard(1) == "one"
assert with_wildcard("anything") == "other"
print("wildcard_catches_anything OK")
"###);
    assert_output(&out, r###"wildcard_catches_anything OK
"###);
}
