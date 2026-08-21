# Operational AssertionPass divergence-spec fixture for the silent
# value-contract divergences of `ast.literal_eval('[1,2,3]') ==
# [1, 2, 3]` (the documented "literal_eval of a list-literal source
# yields the list value" — mamba returns None — list literal not
# decoded), `ast.literal_eval('{1: 2}') == {1: 2}` (the documented
# "literal_eval of a dict-literal source yields the dict value" —
# mamba returns None — dict literal not decoded), `hasattr(ast, '
# iter_fields')` (the documented "ast exposes the iter_fields
# helper" — mamba returns False), `hasattr(ast, 'iter_child_nodes')`
# (the documented "ast exposes the iter_child_nodes helper" — mamba
# returns False), `hasattr(dis, 'Bytecode')` (the documented "dis
# exposes the Bytecode wrapper class" — mamba returns False), `math.
# tau == 6.283185307179586` (the documented "math.tau is the float
# 2*pi" — mamba returns 4618760256179416344 — i64 bit pattern of the
# double instead of the float), `math.pi == 3.141592653589793` (the
# documented "math.pi is the float pi" — mamba returns
# 4614256656552045848 — i64 bit pattern of the double), `math.e ==
# 2.718281828459045` (the documented "math.e is the float e" — mamba
# returns 4613303445314885481 — i64 bit pattern of the double),
# `unicodedata.name('A') == 'LATIN CAPITAL LETTER A'` (the
# documented "unicodedata.name returns the canonical Unicode name" —
# mamba returns 'UNICODE CHAR 0041' — stubbed hex-code fallback), and
# `hasattr(gettext, 'gettext')` (the documented "gettext exposes the
# gettext function" — mamba returns False — module body resolves to
# None).
# Ten-pack pinned to atomic 304.
#
# Behavioral edges that CONFORM on mamba (ast — hasattr parse/dump/
# literal_eval/walk/NodeVisitor/NodeTransformer/Module/Expression/
# Constant/Name/Call/Assign/FunctionDef/unparse/fix_missing_locations
# + literal_eval int/str + ast.parse Module. dis — hasattr dis/
# Instruction/get_instructions/opmap/opname/HAVE_ARGUMENT. tokenize —
# hasattr tokenize/untokenize/generate_tokens/TokenInfo. keyword —
# hasattr iskeyword/kwlist/issoftkeyword/softkwlist + value contracts
# + len 35. token — hasattr NAME/NUMBER/STRING/OP/ENDMARKER/tok_name.
# array — typecodes constant. reprlib — hasattr Repr/repr/recursive_
# repr + repr truncation. shutil — hasattr copy/copy2/copytree/rmtree
# /move/disk_usage/which/get_terminal_size. unicodedata — hasattr
# name/normalize/category/decimal/unidata_version/bidirectional +
# category 'Lu' + normalize identity) are covered in the matching
# pass fixture `test_ast_dis_keyword_unicodedata_value_ops`.
import ast
import dis
import math
import unicodedata
import gettext


_ledger: list[int] = []

# 1) ast.literal_eval('[1,2,3]') == [1, 2, 3] — list-literal decode
#    (mamba: returns None — list literal not decoded)
assert ast.literal_eval("[1,2,3]") == [1, 2, 3]; _ledger.append(1)

# 2) ast.literal_eval('{1: 2}') == {1: 2} — dict-literal decode
#    (mamba: returns None — dict literal not decoded)
assert ast.literal_eval("{1: 2}") == {1: 2}; _ledger.append(1)

# 3) hasattr(ast, 'iter_fields') — iter_fields helper
#    (mamba: returns False)
assert hasattr(ast, "iter_fields") == True; _ledger.append(1)

# 4) hasattr(ast, 'iter_child_nodes') — iter_child_nodes helper
#    (mamba: returns False)
assert hasattr(ast, "iter_child_nodes") == True; _ledger.append(1)

# 5) hasattr(dis, 'Bytecode') — Bytecode wrapper class
#    (mamba: returns False)
assert hasattr(dis, "Bytecode") == True; _ledger.append(1)

# 6) math.tau == 6.283185307179586 — float 2*pi
#    (mamba: returns 4618760256179416344 — i64 bit pattern of the double)
assert math.tau == 6.283185307179586; _ledger.append(1)

# 7) math.pi == 3.141592653589793 — float pi
#    (mamba: returns 4614256656552045848 — i64 bit pattern of the double)
assert math.pi == 3.141592653589793; _ledger.append(1)

# 8) math.e == 2.718281828459045 — float e
#    (mamba: returns 4613303445314885481 — i64 bit pattern of the double)
assert math.e == 2.718281828459045; _ledger.append(1)

# 9) unicodedata.name('A') == 'LATIN CAPITAL LETTER A' — canonical Unicode name
#    (mamba: returns 'UNICODE CHAR 0041' — stubbed hex-code fallback)
assert unicodedata.name("A") == "LATIN CAPITAL LETTER A"; _ledger.append(1)

# 10) hasattr(gettext, 'gettext') — gettext function
#     (mamba: returns False — module body resolves to None)
assert hasattr(gettext, "gettext") == True; _ledger.append(1)

print(f"MAMBA_ASSERTION_PASS: lang_ast_math_unicodedata_silent {sum(_ledger)} asserts")
