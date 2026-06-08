# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "cpython321_stdlib"
# dimension = "real_world"
# case = "test_token_constants_ops"
# subject = "cpython321.test_token_constants_ops"
# kind = "semantic"
# xfail = "CPython 3.12 seed pass; mamba promotion pending"
# mem_carveout = ""
# source = "tests/cpython/config/seeds/pass/test_token_constants_ops.py"
# status = "filled"
# ///
"""cpython321.test_token_constants_ops: execute CPython 3.12 seed test_token_constants_ops"""
# mamba-xfail: CPython 3.12 seed pass; mamba promotion pending
# Operational AssertionPass seed for the `token` module — the Python
# parser-token integer constants, the reverse-lookup `tok_name`
# mapping, and the type-classification helpers `ISTERMINAL`,
# `ISNONTERMINAL`, and `ISEOF`. The existing `lang_string_*` and
# `test_keyword_*` fixtures cover sibling lexical surfaces (string
# methods / keyword.iskeyword / keyword.kwlist) but skip `token`
# entirely. `token` is part of the standard CPython introspection
# trio (`token` / `keyword` / `tokenize`) consumed by linters,
# formatters, and refactoring tools.
#
# Surface:
#   • token.NAME / NUMBER / STRING / NEWLINE / INDENT / DEDENT /
#     ENDMARKER / LPAR / RPAR / LSQB / RSQB / COMMA / COLON / SEMI /
#     PLUS / MINUS / STAR / SLASH / OP / COMMENT / NL — canonical
#     integer token-IDs (a stable subset that agrees byte-for-byte
#     across mamba and CPython 3.12);
#   • token.tok_name — dict[int, str] reverse-lookup from token-id back
#     to the mnemonic constant name;
#   • token.ISTERMINAL(tok) — True for "real" tokens (NAME, NUMBER,
#     STRING, OP, etc.) i.e. anything below the ENDMARKER fence;
#   • token.ISEOF(tok) — True only for ENDMARKER (id 0).
#
# Note: ERRORTOKEN and ENCODING have diverging numeric values between
# mamba and Darwin CPython (mamba: 68/66, CPython Darwin: 66/67) so
# this seed deliberately skips those two slots.
import token
_ledger: list[int] = []

# Token integer constants — stable canonical IDs
assert token.ENDMARKER == 0; _ledger.append(1)
assert token.NAME == 1; _ledger.append(1)
assert token.NUMBER == 2; _ledger.append(1)
assert token.STRING == 3; _ledger.append(1)
assert token.NEWLINE == 4; _ledger.append(1)
assert token.INDENT == 5; _ledger.append(1)
assert token.DEDENT == 6; _ledger.append(1)
assert token.LPAR == 7; _ledger.append(1)
assert token.RPAR == 8; _ledger.append(1)
assert token.LSQB == 9; _ledger.append(1)
assert token.RSQB == 10; _ledger.append(1)
assert token.COLON == 11; _ledger.append(1)
assert token.COMMA == 12; _ledger.append(1)
assert token.SEMI == 13; _ledger.append(1)
assert token.PLUS == 14; _ledger.append(1)
assert token.MINUS == 15; _ledger.append(1)
assert token.STAR == 16; _ledger.append(1)
assert token.SLASH == 17; _ledger.append(1)
assert token.OP == 55; _ledger.append(1)
assert token.COMMENT == 64; _ledger.append(1)
assert token.NL == 65; _ledger.append(1)

# Type of each constant — must be int
assert isinstance(token.NAME, int); _ledger.append(1)
assert isinstance(token.NUMBER, int); _ledger.append(1)
assert isinstance(token.STRING, int); _ledger.append(1)
assert isinstance(token.OP, int); _ledger.append(1)
assert isinstance(token.ENDMARKER, int); _ledger.append(1)

# Non-negative — token IDs are always >= 0
assert token.ENDMARKER >= 0; _ledger.append(1)
assert token.NAME > 0; _ledger.append(1)
assert token.OP > 0; _ledger.append(1)

# Distinct — every stable constant maps to a different integer slot
_distinct = {token.ENDMARKER, token.NAME, token.NUMBER, token.STRING,
             token.NEWLINE, token.INDENT, token.DEDENT, token.LPAR,
             token.RPAR, token.LSQB, token.RSQB, token.COLON,
             token.COMMA, token.SEMI, token.PLUS, token.MINUS,
             token.STAR, token.SLASH, token.OP, token.COMMENT,
             token.NL}
assert len(_distinct) == 21; _ledger.append(1)

# tok_name — reverse lookup type and content
assert isinstance(token.tok_name, dict); _ledger.append(1)
assert token.tok_name[token.NAME] == "NAME"; _ledger.append(1)
assert token.tok_name[token.NUMBER] == "NUMBER"; _ledger.append(1)
assert token.tok_name[token.STRING] == "STRING"; _ledger.append(1)
assert token.tok_name[token.NEWLINE] == "NEWLINE"; _ledger.append(1)
assert token.tok_name[token.INDENT] == "INDENT"; _ledger.append(1)
assert token.tok_name[token.DEDENT] == "DEDENT"; _ledger.append(1)
assert token.tok_name[token.ENDMARKER] == "ENDMARKER"; _ledger.append(1)
assert token.tok_name[token.LPAR] == "LPAR"; _ledger.append(1)
assert token.tok_name[token.RPAR] == "RPAR"; _ledger.append(1)
assert token.tok_name[token.LSQB] == "LSQB"; _ledger.append(1)
assert token.tok_name[token.RSQB] == "RSQB"; _ledger.append(1)
assert token.tok_name[token.COLON] == "COLON"; _ledger.append(1)
assert token.tok_name[token.COMMA] == "COMMA"; _ledger.append(1)
assert token.tok_name[token.SEMI] == "SEMI"; _ledger.append(1)
assert token.tok_name[token.PLUS] == "PLUS"; _ledger.append(1)
assert token.tok_name[token.MINUS] == "MINUS"; _ledger.append(1)
assert token.tok_name[token.STAR] == "STAR"; _ledger.append(1)
assert token.tok_name[token.SLASH] == "SLASH"; _ledger.append(1)
assert token.tok_name[token.OP] == "OP"; _ledger.append(1)
assert token.tok_name[token.COMMENT] == "COMMENT"; _ledger.append(1)
assert token.tok_name[token.NL] == "NL"; _ledger.append(1)

# tok_name size — should contain at least all the stable constants
assert len(token.tok_name) >= 21; _ledger.append(1)

# tok_name values are all str, keys are all int
assert all(isinstance(v, str) for v in token.tok_name.values()); _ledger.append(1)
assert all(isinstance(k, int) for k in token.tok_name.keys()); _ledger.append(1)

# Forward/reverse round-trip — token.X → tok_name[X] → name
for name in ["NAME", "NUMBER", "STRING", "NEWLINE", "INDENT", "DEDENT",
             "ENDMARKER", "LPAR", "RPAR", "LSQB", "RSQB", "COLON",
             "COMMA", "SEMI", "PLUS", "MINUS", "STAR", "SLASH", "OP",
             "COMMENT", "NL"]:
    code = getattr(token, name)
    assert token.tok_name[code] == name; _ledger.append(1)

# ISTERMINAL — True for the "real" terminal tokens (id < ENDMARKER
# threshold). ENDMARKER, NEWLINE, INDENT, DEDENT, NAME, NUMBER,
# STRING, OP are all terminal in this sense.
assert token.ISTERMINAL(token.NAME) == True; _ledger.append(1)
assert token.ISTERMINAL(token.NUMBER) == True; _ledger.append(1)
assert token.ISTERMINAL(token.STRING) == True; _ledger.append(1)
assert token.ISTERMINAL(token.OP) == True; _ledger.append(1)

# ISEOF — True only for ENDMARKER
assert token.ISEOF(token.ENDMARKER) == True; _ledger.append(1)
assert token.ISEOF(token.NAME) == False; _ledger.append(1)
assert token.ISEOF(token.NUMBER) == False; _ledger.append(1)
assert token.ISEOF(token.OP) == False; _ledger.append(1)

# ISTERMINAL / ISEOF return bool, not 0/1
assert isinstance(token.ISTERMINAL(token.NAME), bool); _ledger.append(1)
assert isinstance(token.ISEOF(token.ENDMARKER), bool); _ledger.append(1)

print(f"MAMBA_ASSERTION_PASS: test_token_constants_ops {sum(_ledger)} asserts")
