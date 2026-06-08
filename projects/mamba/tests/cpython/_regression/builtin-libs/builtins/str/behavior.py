"""Behavior contract for builtins.str.

Each block tests one semantic rule. Expected values are what CPython
3.12 produces. mamba PASS = matches CPython exactly.

# type-regime: monomorphic
"""

# Rule 1: str() with no args returns ''
assert str() == "", f"str() = {str()!r}"

# Rule 2: str(int/float/bool)
assert str(42) == "42", f"str(42) = {str(42)!r}"
assert str(-3.5) == "-3.5", f"str(-3.5) = {str(-3.5)!r}"
assert str(True) == "True", f"str(True) = {str(True)!r}"
assert str(False) == "False", f"str(False) = {str(False)!r}"

# Rule 3: str(None)
assert str(None) == "None", f"str(None) = {str(None)!r}"

# Rule 4: upper / lower
assert "hello".upper() == "HELLO", f"'hello'.upper() = {'hello'.upper()!r}"
assert "WORLD".lower() == "world", f"'WORLD'.lower() = {'WORLD'.lower()!r}"

# Rule 5: strip / lstrip / rstrip
assert "  hi  ".strip() == "hi", f"strip = {'  hi  '.strip()!r}"
assert "  hi  ".lstrip() == "hi  ", f"lstrip = {'  hi  '.lstrip()!r}"
assert "  hi  ".rstrip() == "  hi", f"rstrip = {'  hi  '.rstrip()!r}"
assert "xxAxx".strip("x") == "A", f"strip('x') = {'xxAxx'.strip('x')!r}"

# Rule 6: split / join
parts = "a,b,c".split(",")
assert parts == ["a", "b", "c"], f"split = {parts!r}"
assert ",".join(["a", "b", "c"]) == "a,b,c", f"join = {','.join(['a','b','c'])!r}"
assert "a b c".split() == ["a", "b", "c"], "split() with no arg"

# Rule 7: replace
assert "hello".replace("l", "r") == "herro", f"replace = {'hello'.replace('l','r')!r}"
assert "aaa".replace("a", "b", 2) == "bba", f"replace count = {'aaa'.replace('a','b',2)!r}"

# Rule 8: find / index
assert "hello".find("ll") == 2, f"find = {'hello'.find('ll')!r}"
assert "hello".find("z") == -1, f"find miss = {'hello'.find('z')!r}"
assert "hello".index("ll") == 2, f"index = {'hello'.index('ll')!r}"
_raised = False
try:
    "hello".index("z")
except ValueError:
    _raised = True
assert _raised, "'hello'.index('z') did not raise ValueError"

# Rule 9: startswith / endswith
assert "hello".startswith("he"), "'hello'.startswith('he') failed"
assert "hello".endswith("lo"), "'hello'.endswith('lo') failed"
assert not "hello".startswith("lo"), "'hello'.startswith('lo') should be False"

# Rule 10: count
assert "hello".count("l") == 2, f"count = {'hello'.count('l')!r}"
assert "hello".count("z") == 0, f"count miss = {'hello'.count('z')!r}"

# Rule 11: encode / decode
b = "hello".encode("utf-8")
assert b == b"hello", f"encode = {b!r}"
assert b.decode("utf-8") == "hello", f"decode = {b.decode('utf-8')!r}"

# Rule 12: format
assert "{}+{}={}".format(1, 2, 3) == "1+2=3", f"format = {'{}+{}={}'.format(1,2,3)!r}"
assert "{name}".format(name="Alice") == "Alice", "format kwargs"

# Rule 13: f-string
x = 42
assert f"{x}" == "42", f"f-string = {f'{x}'!r}"

# Rule 14: slicing
s = "hello"
assert s[1:3] == "el", f"s[1:3] = {s[1:3]!r}"
assert s[::-1] == "olleh", f"s[::-1] = {s[::-1]!r}"

# Rule 15: in / not in
assert "ell" in "hello", "'ell' in 'hello' failed"
assert "xyz" not in "hello", "'xyz' not in 'hello' failed"

# Rule 16: multiplication
assert "ab" * 3 == "ababab", f"'ab'*3 = {'ab'*3!r}"

# Rule 17: len
assert len("hello") == 5, f"len('hello') = {len('hello')!r}"
assert len("") == 0, f"len('') = {len('')!r}"

print("behavior OK")
