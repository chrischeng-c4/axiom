# Operational AssertionPass seed for the value contract of the
# `argparse` / `getopt` / `shlex` / `keyword` / `token` /
# `tokenize` / `ast` seven-pack pinned to atomic 218:
# `argparse` (the documented partial module-level class
# identifier hasattr surface — `ArgumentParser`), `getopt`
# (the documented partial module-level helper / class
# identifier hasattr surface — `getopt` / `gnu_getopt` /
# `GetoptError` + the documented
# `getopt.getopt(["-a", "5", "--foo", "bar", "remain"],
# "a:", ["foo="]) == ([("-a", "5"), ("--foo", "bar")],
# ["remain"])` short / long option-parse value contract),
# `shlex` (the documented partial module-level helper
# identifier hasattr surface — `split` / `join` / `quote`
# + the documented `shlex.split("a b c") == ["a", "b", "c"]`
# / `shlex.split('a "b c" d') == ["a", "b c", "d"]` /
# `shlex.quote("a b") == "'a b'"` / `shlex.quote("safe") ==
# "safe"` / `shlex.join(["a", "b c"]) == "a 'b c'"`
# shell-quote / split value contract), `keyword` (the
# documented full module-level helper identifier hasattr
# surface — `iskeyword` / `issoftkeyword` / `kwlist` /
# `softkwlist` + the documented
# `keyword.iskeyword("if") == True` /
# `keyword.iskeyword("foo") == False` /
# `keyword.iskeyword("def") == True` /
# `"while" in keyword.kwlist` /
# `"def" in keyword.kwlist` /
# `len(keyword.kwlist) > 30` keyword-classification value
# contract), `token` (the documented full module-level
# sentinel identifier hasattr surface — `NAME` / `NUMBER`
# / `STRING` / `OP` / `NEWLINE` / `INDENT` / `DEDENT` /
# `ENDMARKER` / `tok_name` / `ISTERMINAL`), `tokenize`
# (the documented full module-level helper identifier
# hasattr surface — `tokenize` / `untokenize` /
# `generate_tokens` / `detect_encoding`), and `ast` (the
# documented partial module-level helper / class
# identifier hasattr surface — `parse` / `dump` /
# `literal_eval` / `walk` / `Module` / `Expression` /
# `Name` / `Load` / `Store` / `Constant` / `BinOp` /
# `Add` / `NodeVisitor` / `NodeTransformer` /
# `get_docstring` / `fix_missing_locations` /
# `increment_lineno` + the documented
# `ast.literal_eval("42") == 42` /
# `ast.literal_eval("3.14") == 3.14` /
# `ast.literal_eval('"hello"') == "hello"` /
# `ast.literal_eval("True") == True` /
# `ast.literal_eval("None") is None` scalar literal-eval
# value contract).
#
# Behavioral edges that DIVERGE on mamba
# (hasattr(argparse, "Namespace") / "Action" / "FileType"
# / "ArgumentError" / "ArgumentTypeError" /
# "HelpFormatter" / "RawDescriptionHelpFormatter" /
# "RawTextHelpFormatter" / "ArgumentDefaultsHelpFormatter"
# / "MetavarTypeHelpFormatter" / "REMAINDER" / "OPTIONAL"
# / "ZERO_OR_MORE" / "ONE_OR_MORE" / "SUPPRESS" / "PARSER"
# / "BooleanOptionalAction" all False on mamba +
# type(argparse.ArgumentParser()).__name__ ==
# "ArgumentParser" collapses to "dict" on mamba,
# hasattr(getopt, "error") False on mamba + hasattr(shlex,
# "shlex") False on mamba, hasattr(ast, "iter_fields") /
# "iter_child_nodes" all False on mamba +
# ast.literal_eval("[1, 2, 3]") == [1, 2, 3] /
# ast.literal_eval('{"a": 1}') == {"a": 1} /
# ast.literal_eval("(1, 2)") == (1, 2) all collapse to
# None on mamba + len(ast.parse("x = 1 + 2").body) == 1
# collapses to 0 on mamba) are covered in the matching
# spec fixture `lang_argparse_ast_silent`.
import argparse
import getopt
import shlex
import keyword
import token
import tokenize
import ast


_ledger: list[int] = []

# 1) argparse — partial module hasattr surface
#    (17 attrs + type(ArgumentParser()).__name__ DIVERGE
#    on mamba — moved to spec)
assert hasattr(argparse, "ArgumentParser") == True; _ledger.append(1)

# 2) getopt — partial module hasattr surface
#    (getopt.error DIVERGE on mamba — moved to spec)
assert hasattr(getopt, "getopt") == True; _ledger.append(1)
assert hasattr(getopt, "gnu_getopt") == True; _ledger.append(1)
assert hasattr(getopt, "GetoptError") == True; _ledger.append(1)

# 3) getopt — short / long option-parse value contract
_opts, _args_pos = getopt.getopt(
    ["-a", "5", "--foo", "bar", "remain"], "a:", ["foo="]
)
assert _opts == [("-a", "5"), ("--foo", "bar")]; _ledger.append(1)
assert _args_pos == ["remain"]; _ledger.append(1)

# 4) shlex — partial module hasattr surface
#    (shlex.shlex DIVERGE on mamba — moved to spec)
assert hasattr(shlex, "split") == True; _ledger.append(1)
assert hasattr(shlex, "join") == True; _ledger.append(1)
assert hasattr(shlex, "quote") == True; _ledger.append(1)

# 5) shlex — shell-quote / split value contract
assert shlex.split("a b c") == ["a", "b", "c"]; _ledger.append(1)
assert shlex.split('a "b c" d') == ["a", "b c", "d"]; _ledger.append(1)
assert shlex.quote("a b") == "'a b'"; _ledger.append(1)
assert shlex.quote("safe") == "safe"; _ledger.append(1)
assert shlex.join(["a", "b c"]) == "a 'b c'"; _ledger.append(1)

# 6) keyword — full module hasattr surface
assert hasattr(keyword, "iskeyword") == True; _ledger.append(1)
assert hasattr(keyword, "issoftkeyword") == True; _ledger.append(1)
assert hasattr(keyword, "kwlist") == True; _ledger.append(1)
assert hasattr(keyword, "softkwlist") == True; _ledger.append(1)

# 7) keyword — keyword-classification value contract
assert keyword.iskeyword("if") == True; _ledger.append(1)
assert keyword.iskeyword("foo") == False; _ledger.append(1)
assert keyword.iskeyword("def") == True; _ledger.append(1)
assert "while" in keyword.kwlist; _ledger.append(1)
assert "def" in keyword.kwlist; _ledger.append(1)
assert len(keyword.kwlist) > 30; _ledger.append(1)

# 8) token — full module hasattr surface
assert hasattr(token, "NAME") == True; _ledger.append(1)
assert hasattr(token, "NUMBER") == True; _ledger.append(1)
assert hasattr(token, "STRING") == True; _ledger.append(1)
assert hasattr(token, "OP") == True; _ledger.append(1)
assert hasattr(token, "NEWLINE") == True; _ledger.append(1)
assert hasattr(token, "INDENT") == True; _ledger.append(1)
assert hasattr(token, "DEDENT") == True; _ledger.append(1)
assert hasattr(token, "ENDMARKER") == True; _ledger.append(1)
assert hasattr(token, "tok_name") == True; _ledger.append(1)
assert hasattr(token, "ISTERMINAL") == True; _ledger.append(1)

# 9) tokenize — full module hasattr surface
assert hasattr(tokenize, "tokenize") == True; _ledger.append(1)
assert hasattr(tokenize, "untokenize") == True; _ledger.append(1)
assert hasattr(tokenize, "generate_tokens") == True; _ledger.append(1)
assert hasattr(tokenize, "detect_encoding") == True; _ledger.append(1)

# 10) ast — partial module hasattr surface
#     (iter_fields / iter_child_nodes DIVERGE on mamba —
#     moved to spec)
assert hasattr(ast, "parse") == True; _ledger.append(1)
assert hasattr(ast, "dump") == True; _ledger.append(1)
assert hasattr(ast, "literal_eval") == True; _ledger.append(1)
assert hasattr(ast, "walk") == True; _ledger.append(1)
assert hasattr(ast, "Module") == True; _ledger.append(1)
assert hasattr(ast, "Expression") == True; _ledger.append(1)
assert hasattr(ast, "Name") == True; _ledger.append(1)
assert hasattr(ast, "Load") == True; _ledger.append(1)
assert hasattr(ast, "Store") == True; _ledger.append(1)
assert hasattr(ast, "Constant") == True; _ledger.append(1)
assert hasattr(ast, "BinOp") == True; _ledger.append(1)
assert hasattr(ast, "Add") == True; _ledger.append(1)
assert hasattr(ast, "NodeVisitor") == True; _ledger.append(1)
assert hasattr(ast, "NodeTransformer") == True; _ledger.append(1)
assert hasattr(ast, "get_docstring") == True; _ledger.append(1)
assert hasattr(ast, "fix_missing_locations") == True; _ledger.append(1)
assert hasattr(ast, "increment_lineno") == True; _ledger.append(1)

# 11) ast — scalar literal-eval value contract
#     (container literal_eval [/{/( DIVERGE on mamba —
#     moved to spec)
assert ast.literal_eval("42") == 42; _ledger.append(1)
assert ast.literal_eval("3.14") == 3.14; _ledger.append(1)
assert ast.literal_eval('"hello"') == "hello"; _ledger.append(1)
assert ast.literal_eval("True") == True; _ledger.append(1)
assert ast.literal_eval("None") is None; _ledger.append(1)

# NB: hasattr(argparse, "Namespace") / "Action" /
# "FileType" / "ArgumentError" / "ArgumentTypeError" /
# "HelpFormatter" / "RawDescriptionHelpFormatter" /
# "RawTextHelpFormatter" /
# "ArgumentDefaultsHelpFormatter" /
# "MetavarTypeHelpFormatter" / "REMAINDER" / "OPTIONAL"
# / "ZERO_OR_MORE" / "ONE_OR_MORE" / "SUPPRESS" /
# "PARSER" / "BooleanOptionalAction" all False on mamba
# + type(argparse.ArgumentParser()).__name__ ==
# "ArgumentParser" collapses to "dict" on mamba,
# hasattr(getopt, "error") False on mamba +
# hasattr(shlex, "shlex") False on mamba, hasattr(ast,
# "iter_fields") / "iter_child_nodes" all False on
# mamba + ast.literal_eval("[1, 2, 3]") == [1, 2, 3]
# / ast.literal_eval('{"a": 1}') == {"a": 1} /
# ast.literal_eval("(1, 2)") == (1, 2) all collapse to
# None on mamba + len(ast.parse("x = 1 + 2").body) ==
# 1 collapses to 0 on mamba — all DIVERGE on mamba —
# moved to the divergence-spec fixture.

print(f"MAMBA_ASSERTION_PASS: test_getopt_shlex_keyword_token_tokenize_ast_value_ops {sum(_ledger)} asserts")
