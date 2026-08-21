# Atomic 304 pass conformance — ast module (hasattr parse/dump/
# literal_eval/walk/NodeVisitor/NodeTransformer/Module/Expression/
# Constant/Name/Call/Assign/FunctionDef/unparse/fix_missing_locations
# + literal_eval int + literal_eval str + ast.parse returns Module) +
# dis module (hasattr dis/Instruction/get_instructions/opmap/opname/
# HAVE_ARGUMENT) + tokenize module (hasattr tokenize/untokenize/
# generate_tokens/TokenInfo) + keyword module (hasattr iskeyword/
# kwlist/issoftkeyword/softkwlist + iskeyword True/False predicates +
# kwlist type/len + 'if' membership) + token module (hasattr NAME/
# NUMBER/STRING/OP/ENDMARKER/tok_name) + array module (typecodes
# value contract) + reprlib module (hasattr Repr/repr/recursive_repr
# + repr truncation) + shutil module (hasattr copy/copy2/copytree/
# rmtree/move/disk_usage/which/get_terminal_size) + unicodedata
# module (hasattr name/normalize/category/decimal/unidata_version/
# bidirectional + category 'Lu' + normalize NFC identity).
# All asserts match between CPython 3.12 and mamba.
import ast
import dis
import tokenize
import keyword
import token
import array
import reprlib
import shutil
import unicodedata


_ledger: list[int] = []

# 1) ast — hasattr core surface (conformant subset)
assert hasattr(ast, "parse") == True; _ledger.append(1)
assert hasattr(ast, "dump") == True; _ledger.append(1)
assert hasattr(ast, "literal_eval") == True; _ledger.append(1)
assert hasattr(ast, "walk") == True; _ledger.append(1)
assert hasattr(ast, "NodeVisitor") == True; _ledger.append(1)
assert hasattr(ast, "NodeTransformer") == True; _ledger.append(1)
assert hasattr(ast, "Module") == True; _ledger.append(1)
assert hasattr(ast, "Expression") == True; _ledger.append(1)
assert hasattr(ast, "Constant") == True; _ledger.append(1)
assert hasattr(ast, "Name") == True; _ledger.append(1)
assert hasattr(ast, "Call") == True; _ledger.append(1)
assert hasattr(ast, "Assign") == True; _ledger.append(1)
assert hasattr(ast, "FunctionDef") == True; _ledger.append(1)
assert hasattr(ast, "unparse") == True; _ledger.append(1)
assert hasattr(ast, "fix_missing_locations") == True; _ledger.append(1)

# 2) ast — value contracts (conformant subset)
assert ast.literal_eval("42") == 42; _ledger.append(1)
assert ast.literal_eval('"hi"') == "hi"; _ledger.append(1)
assert type(ast.parse("1+1")).__name__ == "Module"; _ledger.append(1)

# 3) dis — hasattr core surface (conformant subset)
assert hasattr(dis, "dis") == True; _ledger.append(1)
assert hasattr(dis, "Instruction") == True; _ledger.append(1)
assert hasattr(dis, "get_instructions") == True; _ledger.append(1)
assert hasattr(dis, "opmap") == True; _ledger.append(1)
assert hasattr(dis, "opname") == True; _ledger.append(1)
assert hasattr(dis, "HAVE_ARGUMENT") == True; _ledger.append(1)

# 4) tokenize — hasattr core surface
assert hasattr(tokenize, "tokenize") == True; _ledger.append(1)
assert hasattr(tokenize, "untokenize") == True; _ledger.append(1)
assert hasattr(tokenize, "generate_tokens") == True; _ledger.append(1)
assert hasattr(tokenize, "TokenInfo") == True; _ledger.append(1)

# 5) keyword — hasattr core surface
assert hasattr(keyword, "iskeyword") == True; _ledger.append(1)
assert hasattr(keyword, "kwlist") == True; _ledger.append(1)
assert hasattr(keyword, "issoftkeyword") == True; _ledger.append(1)
assert hasattr(keyword, "softkwlist") == True; _ledger.append(1)

# 6) keyword — value contracts
assert keyword.iskeyword("if") == True; _ledger.append(1)
assert keyword.iskeyword("foo") == False; _ledger.append(1)
assert keyword.iskeyword("class") == True; _ledger.append(1)
assert type(keyword.kwlist).__name__ == "list"; _ledger.append(1)
assert len(keyword.kwlist) == 35; _ledger.append(1)
assert ("if" in keyword.kwlist) == True; _ledger.append(1)

# 7) token — hasattr core surface
assert hasattr(token, "NAME") == True; _ledger.append(1)
assert hasattr(token, "NUMBER") == True; _ledger.append(1)
assert hasattr(token, "STRING") == True; _ledger.append(1)
assert hasattr(token, "OP") == True; _ledger.append(1)
assert hasattr(token, "ENDMARKER") == True; _ledger.append(1)
assert hasattr(token, "tok_name") == True; _ledger.append(1)

# 8) array — typecodes value contract
assert array.typecodes == "bBuhHiIlLqQfd"; _ledger.append(1)
assert type(array.typecodes).__name__ == "str"; _ledger.append(1)
assert ("i" in array.typecodes) == True; _ledger.append(1)

# 9) reprlib — hasattr + value contracts
assert hasattr(reprlib, "Repr") == True; _ledger.append(1)
assert hasattr(reprlib, "repr") == True; _ledger.append(1)
assert hasattr(reprlib, "recursive_repr") == True; _ledger.append(1)
assert reprlib.repr([1] * 1000) == "[1, 1, 1, 1, 1, 1, ...]"; _ledger.append(1)

# 10) shutil — hasattr core surface
assert hasattr(shutil, "copy") == True; _ledger.append(1)
assert hasattr(shutil, "copy2") == True; _ledger.append(1)
assert hasattr(shutil, "copytree") == True; _ledger.append(1)
assert hasattr(shutil, "rmtree") == True; _ledger.append(1)
assert hasattr(shutil, "move") == True; _ledger.append(1)
assert hasattr(shutil, "disk_usage") == True; _ledger.append(1)
assert hasattr(shutil, "which") == True; _ledger.append(1)
assert hasattr(shutil, "get_terminal_size") == True; _ledger.append(1)

# 11) unicodedata — hasattr + conformant value contracts
assert hasattr(unicodedata, "name") == True; _ledger.append(1)
assert hasattr(unicodedata, "normalize") == True; _ledger.append(1)
assert hasattr(unicodedata, "category") == True; _ledger.append(1)
assert hasattr(unicodedata, "decimal") == True; _ledger.append(1)
assert hasattr(unicodedata, "unidata_version") == True; _ledger.append(1)
assert hasattr(unicodedata, "bidirectional") == True; _ledger.append(1)
assert unicodedata.category("A") == "Lu"; _ledger.append(1)
assert unicodedata.normalize("NFC", "A") == "A"; _ledger.append(1)

print(f"MAMBA_ASSERTION_PASS: test_ast_dis_keyword_unicodedata_value_ops {sum(_ledger)} asserts")
