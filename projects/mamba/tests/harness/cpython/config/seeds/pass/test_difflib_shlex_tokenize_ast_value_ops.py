# Atomic 280 pass conformance — difflib module (hasattr
# SequenceMatcher/unified_diff/get_close_matches + get_close_matches
# returns matching list) + shlex module (hasattr split/join/quote +
# split 'a b c' == ['a', 'b', 'c'] + split 'a "b c" d' == ['a',
# 'b c', 'd']) + tokenize module (hasattr tokenize/untokenize/
# generate_tokens/open/TokenInfo/TokenError/NAME/NUMBER/STRING/OP/
# NEWLINE/INDENT/DEDENT/ENDMARKER/COMMENT/ENCODING) + ast module
# (hasattr parse/literal_eval/dump/unparse/walk/NodeVisitor/
# NodeTransformer/Module/Expression/Name/Load/Store/Constant/BinOp/
# Add/Call/FunctionDef/ClassDef/If/For/While).
# All asserts match between CPython 3.12 and mamba.
import difflib
import shlex
import tokenize
import ast


_ledger: list[int] = []

# 1) difflib — hasattr core surface
assert hasattr(difflib, "SequenceMatcher") == True; _ledger.append(1)
assert hasattr(difflib, "unified_diff") == True; _ledger.append(1)
assert hasattr(difflib, "get_close_matches") == True; _ledger.append(1)

# 2) difflib — get_close_matches behavioral
assert difflib.get_close_matches("apple", ["ape", "apricot", "banana"]) == ["ape"]; _ledger.append(1)

# 3) shlex — hasattr helper surface
assert hasattr(shlex, "split") == True; _ledger.append(1)
assert hasattr(shlex, "join") == True; _ledger.append(1)
assert hasattr(shlex, "quote") == True; _ledger.append(1)

# 4) shlex — split value contracts
assert shlex.split("a b c") == ["a", "b", "c"]; _ledger.append(1)
assert shlex.split('a "b c" d') == ["a", "b c", "d"]; _ledger.append(1)

# 5) tokenize — hasattr function surface
assert hasattr(tokenize, "tokenize") == True; _ledger.append(1)
assert hasattr(tokenize, "untokenize") == True; _ledger.append(1)
assert hasattr(tokenize, "generate_tokens") == True; _ledger.append(1)
assert hasattr(tokenize, "open") == True; _ledger.append(1)
assert hasattr(tokenize, "TokenInfo") == True; _ledger.append(1)
assert hasattr(tokenize, "TokenError") == True; _ledger.append(1)

# 6) tokenize — hasattr token-type constants
assert hasattr(tokenize, "NAME") == True; _ledger.append(1)
assert hasattr(tokenize, "NUMBER") == True; _ledger.append(1)
assert hasattr(tokenize, "STRING") == True; _ledger.append(1)
assert hasattr(tokenize, "OP") == True; _ledger.append(1)
assert hasattr(tokenize, "NEWLINE") == True; _ledger.append(1)
assert hasattr(tokenize, "INDENT") == True; _ledger.append(1)
assert hasattr(tokenize, "DEDENT") == True; _ledger.append(1)
assert hasattr(tokenize, "ENDMARKER") == True; _ledger.append(1)
assert hasattr(tokenize, "COMMENT") == True; _ledger.append(1)
assert hasattr(tokenize, "ENCODING") == True; _ledger.append(1)

# 7) ast — hasattr top-level api
assert hasattr(ast, "parse") == True; _ledger.append(1)
assert hasattr(ast, "literal_eval") == True; _ledger.append(1)
assert hasattr(ast, "dump") == True; _ledger.append(1)
assert hasattr(ast, "unparse") == True; _ledger.append(1)
assert hasattr(ast, "walk") == True; _ledger.append(1)
assert hasattr(ast, "NodeVisitor") == True; _ledger.append(1)
assert hasattr(ast, "NodeTransformer") == True; _ledger.append(1)

# 8) ast — hasattr node-class surface (module/expression containers)
assert hasattr(ast, "Module") == True; _ledger.append(1)
assert hasattr(ast, "Expression") == True; _ledger.append(1)
assert hasattr(ast, "Name") == True; _ledger.append(1)
assert hasattr(ast, "Load") == True; _ledger.append(1)
assert hasattr(ast, "Store") == True; _ledger.append(1)
assert hasattr(ast, "Constant") == True; _ledger.append(1)

# 9) ast — hasattr expression/statement nodes
assert hasattr(ast, "BinOp") == True; _ledger.append(1)
assert hasattr(ast, "Add") == True; _ledger.append(1)
assert hasattr(ast, "Call") == True; _ledger.append(1)

# 10) ast — hasattr top-level statement nodes
assert hasattr(ast, "FunctionDef") == True; _ledger.append(1)
assert hasattr(ast, "ClassDef") == True; _ledger.append(1)
assert hasattr(ast, "If") == True; _ledger.append(1)
assert hasattr(ast, "For") == True; _ledger.append(1)
assert hasattr(ast, "While") == True; _ledger.append(1)

print(f"MAMBA_ASSERTION_PASS: test_difflib_shlex_tokenize_ast_value_ops {sum(_ledger)} asserts")
