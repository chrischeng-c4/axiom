# Promoted from the upstream unittest port to an executable AssertionPass seed.
# Surface: ast — module surface that mamba actually services today:
#   * ast.parse(src) returning a Module-typed object
#   * ast.literal_eval on scalar literals (int / float / bool / None / str)
#   * ast.dump returning a string
#   * Major AST node-type symbols exposed (Module, BinOp, Add, Sub, Mult,
#     Name, Constant, Call, Load, Store, Expression)
#   * ast.PyCF_ALLOW_TOP_LEVEL_AWAIT exposed with the canonical CPython value
#     (8192) for top-level await compilation
# Intentionally NOT exercised on mamba today (tracked separately):
#   * ast.literal_eval on container literals (tuple/list/dict returns None)
#   * ast.unparse — returns '<unparsed>' literal stub
#   * ast.AST base class — missing on mamba (use isinstance via node-type name)
#   * ast.NodeVisitor / NodeTransformer — lambda stubs
#   * ast.walk traversal contents — only checked non-empty here
import ast

_ledger: list[int] = []

# (1) ast.parse returns a Module-typed object on a simple expression
_t1 = ast.parse("1 + 2")
assert type(_t1).__name__ == "Module", (
    f"ast.parse returns a Module-typed object, got {type(_t1).__name__!r}"
)
_ledger.append(1)

# (2) ast.parse on a function-def yields a Module-typed object
_t2 = ast.parse("def f(): pass")
assert type(_t2).__name__ == "Module", (
    f"ast.parse('def f(): pass') returns Module, got {type(_t2).__name__!r}"
)
_ledger.append(1)

# (3) ast.literal_eval on an int literal returns the int
assert ast.literal_eval("42") - 42 == 0, (
    f"ast.literal_eval('42') == 42, got {ast.literal_eval('42')!r}"
)
_ledger.append(1)

# (4) ast.literal_eval on a negative int literal
assert ast.literal_eval("-7") - (-7) == 0, (
    f"ast.literal_eval('-7') == -7, got {ast.literal_eval('-7')!r}"
)
_ledger.append(1)

# (5) ast.literal_eval on a float literal
_f = ast.literal_eval("3.14")
assert _f == 3.14, f"ast.literal_eval('3.14') == 3.14, got {_f!r}"
_ledger.append(1)

# (6) ast.literal_eval on True / False / None literals
assert ast.literal_eval("True") == True, (
    f"ast.literal_eval('True') is True, got {ast.literal_eval('True')!r}"
)
_ledger.append(1)
assert ast.literal_eval("False") == False, (
    f"ast.literal_eval('False') is False, got {ast.literal_eval('False')!r}"
)
_ledger.append(1)
assert ast.literal_eval("None") is None, (
    f"ast.literal_eval('None') is None, got {ast.literal_eval('None')!r}"
)
_ledger.append(1)

# (7) ast.literal_eval on a single-quoted string literal
_s = ast.literal_eval("'hi'")
assert _s == "hi", f"ast.literal_eval(\"'hi'\") == 'hi', got {_s!r}"
_ledger.append(1)

# (8) ast.dump on a parsed Module returns a non-empty string
_d = ast.dump(_t1)
assert isinstance(_d, str), f"ast.dump returns str, got {type(_d).__name__!r}"
_ledger.append(1)
assert len(_d) > 0, f"ast.dump returns non-empty string, got {_d!r}"
_ledger.append(1)

# (9) Major AST node-type symbols are exposed
for _name in ("Module", "Expression", "BinOp", "Add", "Sub", "Mult",
              "Name", "Constant", "Call", "Load", "Store"):
    assert hasattr(ast, _name), f"ast.{_name} symbol is exposed"
_ledger.append(1)

# (10) ast.PyCF_ALLOW_TOP_LEVEL_AWAIT == 8192 (canonical CPython value)
assert ast.PyCF_ALLOW_TOP_LEVEL_AWAIT - 8192 == 0, (
    f"ast.PyCF_ALLOW_TOP_LEVEL_AWAIT == 8192, got {ast.PyCF_ALLOW_TOP_LEVEL_AWAIT!r}"
)
_ledger.append(1)

# (11) ast.parse symbol + ast.literal_eval symbol exposed
assert hasattr(ast, "parse"), "ast.parse symbol is exposed"
_ledger.append(1)
assert hasattr(ast, "literal_eval"), "ast.literal_eval symbol is exposed"
_ledger.append(1)

# (12) ast.walk yields at least one node on a parsed module
_walked = list(ast.walk(_t1))
assert len(_walked) >= 1, f"ast.walk yields ≥1 node, got {len(_walked)!r}"
_ledger.append(1)

print(f"MAMBA_ASSERTION_PASS: test_ast {sum(_ledger)} asserts")
