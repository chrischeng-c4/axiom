# Operational AssertionPass seed for the keyword-module lookup
# surface. Surface: `keyword.iskeyword(s)` returns True for every
# reserved Python 3.12 word (control flow `if`/`else`/`elif`/`for`/
# `while`/`break`/`continue`; declarations `def`/`class`/`lambda`;
# scoping `global`/`nonlocal`; exception flow `try`/`except`/
# `finally`/`raise`; module flow `import`/`from`/`as`; constants
# `None`/`True`/`False`; operators `and`/`or`/`not`/`in`/`is`;
# coroutine `async`/`await`; misc `pass`/`return`/`yield`/`with`/
# `assert`/`del`) and False for ordinary identifiers, builtin
# names like `print`, dunders like `__init__`, the empty string,
# and the soft keywords `match`/`case`; `keyword.kwlist` is a list
# containing every hard keyword and excluding identifiers; and
# `keyword.softkwlist` is a list containing `match` and `case`.
import keyword
_ledger: list[int] = []

# Hard keywords — full Python 3.12 reserved-word inventory
assert keyword.iskeyword("if") == True; _ledger.append(1)
assert keyword.iskeyword("else") == True; _ledger.append(1)
assert keyword.iskeyword("elif") == True; _ledger.append(1)
assert keyword.iskeyword("for") == True; _ledger.append(1)
assert keyword.iskeyword("while") == True; _ledger.append(1)
assert keyword.iskeyword("def") == True; _ledger.append(1)
assert keyword.iskeyword("class") == True; _ledger.append(1)
assert keyword.iskeyword("return") == True; _ledger.append(1)
assert keyword.iskeyword("import") == True; _ledger.append(1)
assert keyword.iskeyword("from") == True; _ledger.append(1)
assert keyword.iskeyword("as") == True; _ledger.append(1)
assert keyword.iskeyword("pass") == True; _ledger.append(1)
assert keyword.iskeyword("None") == True; _ledger.append(1)
assert keyword.iskeyword("True") == True; _ledger.append(1)
assert keyword.iskeyword("False") == True; _ledger.append(1)
assert keyword.iskeyword("and") == True; _ledger.append(1)
assert keyword.iskeyword("or") == True; _ledger.append(1)
assert keyword.iskeyword("not") == True; _ledger.append(1)
assert keyword.iskeyword("in") == True; _ledger.append(1)
assert keyword.iskeyword("is") == True; _ledger.append(1)
assert keyword.iskeyword("lambda") == True; _ledger.append(1)
assert keyword.iskeyword("global") == True; _ledger.append(1)
assert keyword.iskeyword("nonlocal") == True; _ledger.append(1)
assert keyword.iskeyword("try") == True; _ledger.append(1)
assert keyword.iskeyword("except") == True; _ledger.append(1)
assert keyword.iskeyword("finally") == True; _ledger.append(1)
assert keyword.iskeyword("raise") == True; _ledger.append(1)
assert keyword.iskeyword("yield") == True; _ledger.append(1)
assert keyword.iskeyword("with") == True; _ledger.append(1)
assert keyword.iskeyword("assert") == True; _ledger.append(1)
assert keyword.iskeyword("del") == True; _ledger.append(1)
assert keyword.iskeyword("break") == True; _ledger.append(1)
assert keyword.iskeyword("continue") == True; _ledger.append(1)
assert keyword.iskeyword("async") == True; _ledger.append(1)
assert keyword.iskeyword("await") == True; _ledger.append(1)

# Negative — identifiers, builtins, dunders, empty string, soft kws
assert keyword.iskeyword("foo") == False; _ledger.append(1)
assert keyword.iskeyword("x") == False; _ledger.append(1)
assert keyword.iskeyword("") == False; _ledger.append(1)
assert keyword.iskeyword("print") == False; _ledger.append(1)
assert keyword.iskeyword("self") == False; _ledger.append(1)
assert keyword.iskeyword("__init__") == False; _ledger.append(1)
assert keyword.iskeyword("match") == False; _ledger.append(1)
assert keyword.iskeyword("case") == False; _ledger.append(1)

# kwlist — full list, must contain hard kws and exclude identifiers
assert isinstance(keyword.kwlist, list); _ledger.append(1)
assert "if" in keyword.kwlist; _ledger.append(1)
assert "def" in keyword.kwlist; _ledger.append(1)
assert "foo" not in keyword.kwlist; _ledger.append(1)

# softkwlist — soft keywords introduced in 3.10 (match, case)
assert isinstance(keyword.softkwlist, list); _ledger.append(1)
assert "match" in keyword.softkwlist; _ledger.append(1)
assert "case" in keyword.softkwlist; _ledger.append(1)

print(f"MAMBA_ASSERTION_PASS: test_keyword_iskeyword_ops {sum(_ledger)} asserts")
