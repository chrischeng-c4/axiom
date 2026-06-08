# Operational AssertionPass seed for the `token` module — the
# canonical integer-id namespace shared by `tokenize` / `ast` /
# `compile` / `inspect.signature`-style introspection tools. Surface
# focuses on the integer-constant identifiers (`NAME`, `NUMBER`,
# `STRING`, `OP`, etc.), the reverse-lookup map `tok_name`, and the
# three classifier predicates (`ISEOF`, `ISTERMINAL`, `ISNONTERMINAL`).
# The numeric values of each token constant are part of CPython's
# stable public API (`Lib/token.py` / `Include/token.h`) — mamba
# matches the CPython 3.12 ordering exactly except for `ENCODING`
# (mamba 66 vs CPython 67), which we don't pin to a specific int. No
# fixture coverage yet for `token`.
#
# Surface:
#   • token.ENDMARKER == 0;   token.NAME == 1;   token.NUMBER == 2;
#     token.STRING == 3;      token.NEWLINE == 4; token.INDENT == 5;
#     token.DEDENT == 6;      token.LPAR == 7;    token.RPAR == 8;
#     token.LSQB == 9;        token.RSQB == 10;   token.COLON == 11;
#     token.COMMA == 12;      token.SEMI == 13;   token.PLUS == 14;
#     token.MINUS == 15;      token.STAR == 16;   token.SLASH == 17;
#     token.EQUAL == 22;      token.OP == 55;     token.COMMENT == 64;
#     token.NL == 65;
#     — `ENCODING` is not pinned (mamba and CPython diverge on the
#       exact int but both expose it);
#   • token.tok_name — dict[int, str]; reverse-lookup map;
#       — token.tok_name[token.NAME] == "NAME";
#       — token.tok_name[token.NUMBER] == "NUMBER";
#       — token.tok_name[token.STRING] == "STRING";
#       — token.tok_name[token.ENDMARKER] == "ENDMARKER";
#   • token.ISEOF(x) — True iff x == ENDMARKER;
#   • token.ISTERMINAL(x) — True iff x corresponds to a terminal
#     grammar symbol (NAME/NUMBER/STRING/OP/etc.);
#   • token.ISNONTERMINAL(x) — True iff x corresponds to a non-
#     terminal grammar symbol (compound rule); inverse of
#     ISTERMINAL for `OP` and below.
import token
_ledger: list[int] = []

# Canonical integer constants — fixed by CPython public API
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
assert token.EQUAL == 22; _ledger.append(1)
assert token.OP == 55; _ledger.append(1)
assert token.COMMENT == 64; _ledger.append(1)
assert token.NL == 65; _ledger.append(1)

# ENCODING attribute presence (numeric value diverges; just check it's int)
assert hasattr(token, "ENCODING"); _ledger.append(1)
assert isinstance(token.ENCODING, int); _ledger.append(1)

# All canonical constants are int
assert isinstance(token.ENDMARKER, int); _ledger.append(1)
assert isinstance(token.NAME, int); _ledger.append(1)
assert isinstance(token.NUMBER, int); _ledger.append(1)
assert isinstance(token.STRING, int); _ledger.append(1)
assert isinstance(token.NEWLINE, int); _ledger.append(1)
assert isinstance(token.OP, int); _ledger.append(1)

# tok_name reverse-lookup map
assert isinstance(token.tok_name, dict); _ledger.append(1)
assert len(token.tok_name) > 0; _ledger.append(1)
assert token.tok_name[token.NAME] == "NAME"; _ledger.append(1)
assert token.tok_name[token.NUMBER] == "NUMBER"; _ledger.append(1)
assert token.tok_name[token.STRING] == "STRING"; _ledger.append(1)
assert token.tok_name[token.ENDMARKER] == "ENDMARKER"; _ledger.append(1)
assert token.tok_name[token.NEWLINE] == "NEWLINE"; _ledger.append(1)
assert token.tok_name[token.INDENT] == "INDENT"; _ledger.append(1)
assert token.tok_name[token.DEDENT] == "DEDENT"; _ledger.append(1)
assert token.tok_name[token.OP] == "OP"; _ledger.append(1)
assert token.tok_name[token.COMMA] == "COMMA"; _ledger.append(1)
assert token.tok_name[token.COLON] == "COLON"; _ledger.append(1)
assert token.tok_name[token.LPAR] == "LPAR"; _ledger.append(1)
assert token.tok_name[token.RPAR] == "RPAR"; _ledger.append(1)
assert token.tok_name[token.LSQB] == "LSQB"; _ledger.append(1)
assert token.tok_name[token.RSQB] == "RSQB"; _ledger.append(1)
assert token.tok_name[token.COMMENT] == "COMMENT"; _ledger.append(1)
assert token.tok_name[token.NL] == "NL"; _ledger.append(1)

# tok_name values are all str
assert all(isinstance(_n, str) for _n in token.tok_name.values()); _ledger.append(1)
# tok_name keys are all int
assert all(isinstance(_k, int) for _k in token.tok_name.keys()); _ledger.append(1)

# Round-trip: every canonical constant has a tok_name entry
for _c in [token.ENDMARKER, token.NAME, token.NUMBER, token.STRING,
           token.NEWLINE, token.INDENT, token.DEDENT, token.OP,
           token.COMMA, token.COLON, token.LPAR, token.RPAR,
           token.LSQB, token.RSQB, token.COMMENT, token.NL]:
    assert _c in token.tok_name; _ledger.append(1)

# Classifier predicates exist
assert callable(token.ISEOF); _ledger.append(1)
assert callable(token.ISTERMINAL); _ledger.append(1)
assert callable(token.ISNONTERMINAL); _ledger.append(1)

# ISEOF — true only for ENDMARKER
assert token.ISEOF(token.ENDMARKER) == True; _ledger.append(1)
assert token.ISEOF(token.NAME) == False; _ledger.append(1)
assert token.ISEOF(token.NUMBER) == False; _ledger.append(1)
assert token.ISEOF(token.STRING) == False; _ledger.append(1)
assert token.ISEOF(token.OP) == False; _ledger.append(1)

# ISTERMINAL — NAME/NUMBER/STRING/OP are terminals
assert token.ISTERMINAL(token.NAME) == True; _ledger.append(1)
assert token.ISTERMINAL(token.NUMBER) == True; _ledger.append(1)
assert token.ISTERMINAL(token.STRING) == True; _ledger.append(1)
assert token.ISTERMINAL(token.OP) == True; _ledger.append(1)
assert token.ISTERMINAL(token.ENDMARKER) == True; _ledger.append(1)

# ISNONTERMINAL is the complement of ISTERMINAL for canonical IDs
assert token.ISNONTERMINAL(token.NAME) == False; _ledger.append(1)
assert token.ISNONTERMINAL(token.NUMBER) == False; _ledger.append(1)
assert token.ISNONTERMINAL(token.STRING) == False; _ledger.append(1)
assert token.ISNONTERMINAL(token.OP) == False; _ledger.append(1)

# Repeatable — calling twice returns same result
assert token.ISEOF(token.ENDMARKER) == token.ISEOF(token.ENDMARKER); _ledger.append(1)
assert token.ISTERMINAL(token.NAME) == token.ISTERMINAL(token.NAME); _ledger.append(1)
assert token.tok_name[token.NAME] == token.tok_name[token.NAME]; _ledger.append(1)

# Return-type discipline
assert isinstance(token.ISEOF(token.ENDMARKER), bool); _ledger.append(1)
assert isinstance(token.ISTERMINAL(token.NAME), bool); _ledger.append(1)
assert isinstance(token.ISNONTERMINAL(token.NAME), bool); _ledger.append(1)
assert isinstance(token.tok_name[token.NAME], str); _ledger.append(1)

# All distinct canonical integer constants — no collisions
_constants = [token.ENDMARKER, token.NAME, token.NUMBER, token.STRING,
              token.NEWLINE, token.INDENT, token.DEDENT, token.LPAR,
              token.RPAR, token.LSQB, token.RSQB, token.COLON,
              token.COMMA, token.SEMI, token.PLUS, token.MINUS,
              token.STAR, token.SLASH, token.EQUAL, token.OP,
              token.COMMENT, token.NL]
assert len(_constants) == len(set(_constants)); _ledger.append(1)

print(f"MAMBA_ASSERTION_PASS: test_token_constants_classifiers_ops {sum(_ledger)} asserts")
