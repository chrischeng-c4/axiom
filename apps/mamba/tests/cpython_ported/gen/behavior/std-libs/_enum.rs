use super::super::super::super::harness::*;

/// Ported from `tests/cpython/behavior/std-libs/enum/intenum_compares_and_arithmetic.py`.
#[test]
fn test_gen_behavior_std_libs_enum_intenum_compares_and_arithmetic() {
    let out = jit_capture(r###"# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "enum"
# dimension = "behavior"
# case = "intenum_compares_and_arithmetic"
# subject = "enum.IntEnum"
# kind = "semantic"
# xfail = ""
# mem_carveout = ""
# source = "Lib/test/test_enum.py"
# status = "filled"
# ///
"""enum.IntEnum: IntEnum members are real ints: they compare to ints and to each other, and arithmetic on them returns a plain int"""
import enum


class Score(enum.IntEnum):
    LOW = 1
    MID = 5
    HIGH = 10


# IntEnum members compare to ints and to one another.
assert Score.MID == 5, "IntEnum member equals its int value"
assert Score.HIGH > 5, "IntEnum compares to a bare int"
assert Score.HIGH > Score.LOW, "IntEnum members compare to each other"
assert isinstance(Score.HIGH, int), f"IntEnum is int: {type(Score.HIGH)!r}"

# Arithmetic on IntEnum members returns a plain int, not an enum member.
total = Score.MID + Score.LOW
assert total == 6, f"IntEnum add = {total!r}"
assert isinstance(total, int) and not isinstance(total, Score), "add returns plain int"

print("intenum_compares_and_arithmetic OK")
"###);
    assert_output(&out, r###"intenum_compares_and_arithmetic OK
"###);
}

/// Ported from `tests/cpython/behavior/std-libs/enum/match_case_on_members.py`.
#[test]
fn test_gen_behavior_std_libs_enum_match_case_on_members() {
    let out = jit_capture(r###"# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "enum"
# dimension = "behavior"
# case = "match_case_on_members"
# subject = "enum.Enum"
# kind = "semantic"
# xfail = ""
# mem_carveout = ""
# source = "Lib/test/test_enum.py"
# status = "filled"
# ///
"""enum.Enum: an Enum member works as a PEP 634 match/case pattern, including an OR-pattern (case A | B) and the wildcard fallback"""
import enum


class Suit(enum.Enum):
    CLUBS = 1
    DIAMONDS = 2
    HEARTS = 3
    SPADES = 4


def describe(s):
    match s:
        case Suit.HEARTS | Suit.DIAMONDS:
            return "red"
        case _:
            return "black"


assert describe(Suit.HEARTS) == "red", "OR-pattern matches HEARTS"
assert describe(Suit.DIAMONDS) == "red", "OR-pattern matches DIAMONDS"
assert describe(Suit.CLUBS) == "black", "wildcard fallback for CLUBS"

print("match_case_on_members OK")
"###);
    assert_output(&out, r###"match_case_on_members OK
"###);
}
