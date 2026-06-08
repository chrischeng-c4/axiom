# Operational AssertionPass divergence-spec fixture for the silent
# value-contract divergences of `hasattr(difflib, 'Differ')` (the
# documented "difflib exposes the Differ class" — mamba returns
# False), `hasattr(difflib, 'ndiff')` (the documented "difflib
# exposes the ndiff helper" — mamba returns False), `hasattr(difflib,
# 'context_diff')` (the documented "difflib exposes the context_diff
# helper" — mamba returns False), `hasattr(difflib, 'HtmlDiff')`
# (the documented "difflib exposes the HtmlDiff class" — mamba
# returns False), `hasattr(difflib, 'IS_LINE_JUNK')` (the documented
# "difflib exposes the IS_LINE_JUNK predicate" — mamba returns
# False), `type(difflib.SequenceMatcher).__name__` (the documented
# "SequenceMatcher metatype is 'type'" — mamba returns 'function' —
# SequenceMatcher is a constructor-as-function), `list(difflib.
# unified_diff(['a\\n'], ['b\\n'], n=0))` (the documented "unified_
# diff yields ['--- \\n', '+++ \\n', ...]" — mamba returns [] —
# generator emits no records), `hasattr(shlex, 'shlex')` (the
# documented "shlex exposes the shlex lexer class" — mamba returns
# False), `ast.literal_eval('[1,2,3]')` (the documented "literal_eval
# evaluates '[1,2,3]' to [1,2,3]" — mamba returns None), and `ast.
# dump(ast.parse('1'))[:8]` (the documented "ast.dump renders
# 'Module(body=...)' format" — mamba returns 'Module()' — empty body
# rendering).
# Ten-pack pinned to atomic 280.
#
# Behavioral edges that CONFORM on mamba (difflib — hasattr
# SequenceMatcher/unified_diff/get_close_matches + get_close_matches
# returns matching list. shlex — hasattr split/join/quote + split
# value contracts. tokenize — hasattr tokenize/untokenize/generate_
# tokens/open/TokenInfo/TokenError/NAME/NUMBER/STRING/OP/NEWLINE/
# INDENT/DEDENT/ENDMARKER/COMMENT/ENCODING. ast — hasattr parse/
# literal_eval/dump/unparse/walk/NodeVisitor/NodeTransformer/Module/
# Expression/Name/Load/Store/Constant/BinOp/Add/Call/FunctionDef/
# ClassDef/If/For/While) are covered in the matching pass fixture
# `test_difflib_shlex_tokenize_ast_value_ops`.
import difflib
import shlex
import ast


_ledger: list[int] = []

# 1) hasattr(difflib, 'Differ') — Differ class
#    (mamba: returns False)
assert hasattr(difflib, "Differ") == True; _ledger.append(1)

# 2) hasattr(difflib, 'ndiff') — ndiff helper
#    (mamba: returns False)
assert hasattr(difflib, "ndiff") == True; _ledger.append(1)

# 3) hasattr(difflib, 'context_diff') — context_diff helper
#    (mamba: returns False)
assert hasattr(difflib, "context_diff") == True; _ledger.append(1)

# 4) hasattr(difflib, 'HtmlDiff') — HtmlDiff class
#    (mamba: returns False)
assert hasattr(difflib, "HtmlDiff") == True; _ledger.append(1)

# 5) hasattr(difflib, 'IS_LINE_JUNK') — junk predicate
#    (mamba: returns False)
assert hasattr(difflib, "IS_LINE_JUNK") == True; _ledger.append(1)

# 6) type(difflib.SequenceMatcher).__name__ == 'type' — SequenceMatcher metatype
#    (mamba: returns 'function' — constructor-as-function)
assert type(difflib.SequenceMatcher).__name__ == "type"; _ledger.append(1)

# 7) list(difflib.unified_diff(['a\n'], ['b\n'], n=0)) != [] — emits records
#    (mamba: returns [] — generator emits nothing)
assert list(difflib.unified_diff(["a\n"], ["b\n"], n=0)) != []; _ledger.append(1)

# 8) hasattr(shlex, 'shlex') — shlex lexer class
#    (mamba: returns False)
assert hasattr(shlex, "shlex") == True; _ledger.append(1)

# 9) ast.literal_eval('[1,2,3]') == [1, 2, 3] — literal list eval
#    (mamba: returns None)
assert ast.literal_eval("[1,2,3]") == [1, 2, 3]; _ledger.append(1)

# 10) ast.dump(ast.parse('1'))[:8] == 'Module(b' — dump format
#     (mamba: returns 'Module()' — empty body rendering)
assert ast.dump(ast.parse("1"))[:8] == "Module(b"; _ledger.append(1)

print(f"MAMBA_ASSERTION_PASS: lang_difflib_shlex_tokenize_ast_silent {sum(_ledger)} asserts")
