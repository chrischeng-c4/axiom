use super::super::super::super::harness::*;

/// Ported from `tests/cpython/behavior/std-libs/token/exact_token_types_maps_op_to_int.py`.
#[test]
fn test_gen_behavior_std_libs_token_exact_token_types_maps_op_to_int() {
    let out = jit_capture(r###"# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "token"
# dimension = "behavior"
# case = "exact_token_types_maps_op_to_int"
# subject = "token.EXACT_TOKEN_TYPES"
# kind = "semantic"
# xfail = ""
# mem_carveout = ""
# source = "Lib/token.py"
# status = "filled"
# ///
"""token.EXACT_TOKEN_TYPES: EXACT_TOKEN_TYPES maps an operator string to its token-type int ('+' -> PLUS=14) and omits unknown operators"""
import token

# EXACT_TOKEN_TYPES is a dict[str, int] keyed by the operator's literal text.
assert isinstance(token.EXACT_TOKEN_TYPES, dict), type(token.EXACT_TOKEN_TYPES).__name__

# '+' resolves to PLUS (14); the mapped value equals the named constant.
assert token.EXACT_TOKEN_TYPES["+"] == 14, token.EXACT_TOKEN_TYPES["+"]
assert token.EXACT_TOKEN_TYPES["+"] == token.PLUS, token.EXACT_TOKEN_TYPES["+"]
assert token.EXACT_TOKEN_TYPES["=="] == token.EQEQUAL, token.EXACT_TOKEN_TYPES["=="]

# Unknown operator strings are simply absent (no entry, not a sentinel).
assert "ZZ" not in token.EXACT_TOKEN_TYPES
assert "NAME" not in token.EXACT_TOKEN_TYPES

print("exact_token_types_maps_op_to_int OK")
"###);
    assert_output(&out, r###"exact_token_types_maps_op_to_int OK
"###);
}

/// Ported from `tests/cpython/behavior/std-libs/token/iseof_only_for_endmarker.py`.
#[test]
fn test_gen_behavior_std_libs_token_iseof_only_for_endmarker() {
    let out = jit_capture(r###"# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "token"
# dimension = "behavior"
# case = "iseof_only_for_endmarker"
# subject = "token.ISEOF"
# kind = "semantic"
# xfail = ""
# mem_carveout = ""
# source = "Lib/token.py"
# status = "filled"
# ///
"""token.ISEOF: ISEOF is True only for the ENDMARKER type (0) and False for ordinary token types"""
import token

# ISEOF flags the end-of-input marker only.
assert token.ENDMARKER == 0, token.ENDMARKER
assert token.ISEOF(token.ENDMARKER) is True
assert token.ISEOF(0) is True

# Every ordinary token type is not EOF.
for tok_type in [token.NAME, token.NUMBER, token.OP, token.STRING, 1, 256]:
    assert token.ISEOF(tok_type) is False, tok_type

print("iseof_only_for_endmarker OK")
"###);
    assert_output(&out, r###"iseof_only_for_endmarker OK
"###);
}

/// Ported from `tests/cpython/behavior/std-libs/token/isterminal_partitions_at_nt_offset.py`.
#[test]
fn test_gen_behavior_std_libs_token_isterminal_partitions_at_nt_offset() {
    let out = jit_capture(r###"# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "token"
# dimension = "behavior"
# case = "isterminal_partitions_at_nt_offset"
# subject = "token.ISTERMINAL"
# kind = "semantic"
# xfail = ""
# mem_carveout = ""
# source = "Lib/token.py"
# status = "filled"
# ///
"""token.ISTERMINAL: ISTERMINAL is True below NT_OFFSET and False at/above it, the exact complement of ISNONTERMINAL"""
import token

# NT_OFFSET (256) is the boundary between terminal and non-terminal types.
assert token.NT_OFFSET == 256, token.NT_OFFSET

# Below the offset: terminal tokens. At/above: non-terminals.
for tok_type in [0, 1, token.NAME, token.NT_OFFSET - 1]:
    assert token.ISTERMINAL(tok_type) is True, tok_type
    assert token.ISNONTERMINAL(tok_type) is False, tok_type

for tok_type in [token.NT_OFFSET, token.NT_OFFSET + 1, 300]:
    assert token.ISTERMINAL(tok_type) is False, tok_type
    assert token.ISNONTERMINAL(tok_type) is True, tok_type

# The two predicates are exact complements across the boundary.
for tok_type in [0, 1, 255, 256, 257]:
    assert token.ISTERMINAL(tok_type) != token.ISNONTERMINAL(tok_type), tok_type

print("isterminal_partitions_at_nt_offset OK")
"###);
    assert_output(&out, r###"isterminal_partitions_at_nt_offset OK
"###);
}

/// Ported from `tests/cpython/behavior/std-libs/token/tok_name_maps_int_to_name.py`.
#[test]
fn test_gen_behavior_std_libs_token_tok_name_maps_int_to_name() {
    let out = jit_capture(r###"# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "token"
# dimension = "behavior"
# case = "tok_name_maps_int_to_name"
# subject = "token.tok_name"
# kind = "semantic"
# xfail = ""
# mem_carveout = ""
# source = "Lib/token.py"
# status = "filled"
# ///
"""token.tok_name: tok_name is a dict mapping each token-type int back to its symbolic name (ENDMARKER=0, NAME=1)"""
import token

# tok_name is a dict[int, str].
assert isinstance(token.tok_name, dict), type(token.tok_name).__name__

# It maps each token-type int back to that constant's symbolic name.
assert token.tok_name[0] == "ENDMARKER", token.tok_name[0]
assert token.tok_name[1] == "NAME", token.tok_name[1]
assert token.tok_name[token.NUMBER] == "NUMBER", token.tok_name[token.NUMBER]

# The round-trip closes: name -> value -> name for each public constant.
for name in ["ENDMARKER", "NAME", "NUMBER", "OP", "STRING"]:
    value = getattr(token, name)
    assert token.tok_name[value] == name, (name, value, token.tok_name[value])

print("tok_name_maps_int_to_name OK")
"###);
    assert_output(&out, r###"tok_name_maps_int_to_name OK
"###);
}

/// Ported from `tests/cpython/behavior/std-libs/token/token_constants_are_ints.py`.
#[test]
fn test_gen_behavior_std_libs_token_token_constants_are_ints() {
    let out = jit_capture(r###"# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "token"
# dimension = "behavior"
# case = "token_constants_are_ints"
# subject = "token.NAME"
# kind = "semantic"
# xfail = ""
# mem_carveout = ""
# source = "Lib/token.py"
# status = "filled"
# ///
"""token.NAME: the public token-type constants NAME/NUMBER/OP/STRING/ENDMARKER are plain ints"""
import token

# Every public token-type constant is a plain int (not bool, not a subclass).
for name in ["NAME", "NUMBER", "OP", "STRING", "ENDMARKER"]:
    value = getattr(token, name)
    assert isinstance(value, int), (name, type(value).__name__)
    assert type(value) is int, (name, type(value).__name__)

print("token_constants_are_ints OK")
"###);
    assert_output(&out, r###"token_constants_are_ints OK
"###);
}
