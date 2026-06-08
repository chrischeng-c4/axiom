"""Behavior contract for third-party jmespath package.

Each block tests one semantic rule. Expected values are what CPython
3.12 produces. mamba PASS = matches CPython exactly.

# type-regime: monomorphic
"""

import jmespath  # type: ignore[import]
import jmespath.exceptions  # type: ignore[import]

# Rule 1: search finds nested key
_data1 = {"foo": {"bar": "baz"}}
assert jmespath.search("foo.bar", _data1) == "baz", \
    f"nested = {jmespath.search('foo.bar', _data1)!r}"

# Rule 2: search with array index
_data2 = {"items": [{"name": "a"}, {"name": "b"}, {"name": "c"}]}
assert jmespath.search("items[1].name", _data2) == "b", \
    f"array = {jmespath.search('items[1].name', _data2)!r}"

# Rule 3: search with wildcard
_data3 = {"users": [{"id": 1}, {"id": 2}, {"id": 3}]}
_r3 = jmespath.search("users[*].id", _data3)
assert _r3 == [1, 2, 3], f"wildcard = {_r3!r}"

# Rule 4: search returns None for missing key
_data4 = {"a": 1}
assert jmespath.search("b.c", _data4) is None, \
    f"missing = {jmespath.search('b.c', _data4)!r}"

# Rule 5: compile caches and re-uses expression
_expr5 = jmespath.compile("foo.bar")
_r5a = _expr5.search({"foo": {"bar": 42}})
_r5b = _expr5.search({"foo": {"bar": 99}})
assert _r5a == 42, f"expr result a = {_r5a!r}"
assert _r5b == 99, f"expr result b = {_r5b!r}"

# Rule 6: ParseError raised for invalid expression
_raised6 = False
try:
    jmespath.compile("!!!invalid!!!")
except jmespath.exceptions.ParseError:
    _raised6 = True
assert _raised6, "ParseError on invalid expression"

# Rule 7: Module attributes are identity-stable
_s_ref = jmespath.search
_c_ref = jmespath.compile
_o_ref = jmespath.Options
_v_ref = jmespath.__version__
for _ in range(5):
    assert jmespath.search is _s_ref, "search stable"
    assert jmespath.compile is _c_ref, "compile stable"
    assert jmespath.Options is _o_ref, "Options stable"
    assert jmespath.__version__ is _v_ref, "__version__ stable"

print("behavior OK")
