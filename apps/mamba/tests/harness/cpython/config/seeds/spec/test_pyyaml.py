# test_pyyaml.py — #3463 axis-1 3p pyyaml AssertionPass seed.
#
# Mamba-authored seed exercising the pyyaml surface called out in the
# issue:
#   * safe_load — basic scalar/list/dict types
#   * dump — scalar/list/dict round-trip
#   * add_constructor — custom !tag support
#   * safe_load_all / dump_all — multi-document
#
# Contract placement: `spec/` — pins outcome Fail. Mamba pkgmgr (Phase
# 1.5 per #1262) cannot yet install pure-Python wheels like PyYAML, so
# `import yaml` fails on mamba today. Once mamba pkgmgr installs yaml
# cleanly and the seed flips to AssertionPass on mamba, drift detection
# prompts a `git mv spec/test_pyyaml.py pass/test_pyyaml.py`.
#
# Surface coverage (asserts run at module scope; no helper closures per
# the mamba top-level def() quirk in test_math.py):
#   1. Module identity + __version__ surface.
#   2. safe_load — int / float / str / bool / null scalar coercion.
#   3. safe_load — list / dict / nested-structure parsing.
#   4. dump — scalar / list / dict round-trips through safe_load.
#   5. add_constructor — custom !tag dispatch.
#   6. safe_load_all / dump_all — multi-document YAML round-trip.
#   7. YAMLError raised on malformed input.
#
# Contract with `cpython_lib_test_runner.rs`:
#   - Each `assert` runs at top level. Inverting any assert raises
#     AssertionError → non-zero exit → runner classifies as `Fail`.
#   - After every assertion has executed, the seed emits
#     `MAMBA_ASSERTION_PASS: test_pyyaml N asserts` to stdout.

import yaml

_ledger: list[int] = []

# 1. Module identity.
assert yaml.__name__ == "yaml", "yaml.__name__"
_ledger.append(1)
assert hasattr(yaml, "safe_load"), "yaml exposes safe_load"
_ledger.append(1)
assert hasattr(yaml, "dump"), "yaml exposes dump"
_ledger.append(1)
assert hasattr(yaml, "safe_load_all"), "yaml exposes safe_load_all"
_ledger.append(1)
assert hasattr(yaml, "dump_all"), "yaml exposes dump_all"
_ledger.append(1)


# 2. safe_load — basic scalar coercion.
assert yaml.safe_load("42") == 42, "safe_load coerces bare int"
_ledger.append(1)
assert yaml.safe_load("3.14") == 3.14, "safe_load coerces bare float"
_ledger.append(1)
assert yaml.safe_load("hello") == "hello", "safe_load yields bare string"
_ledger.append(1)
assert yaml.safe_load("true") is True, "safe_load coerces true → True"
_ledger.append(1)
assert yaml.safe_load("false") is False, "safe_load coerces false → False"
_ledger.append(1)
assert yaml.safe_load("null") is None, "safe_load coerces null → None"
_ledger.append(1)
assert yaml.safe_load("") is None, "safe_load on empty input yields None"
_ledger.append(1)


# 3. safe_load — list / dict / nested.
_lst = yaml.safe_load("- a\n- b\n- c\n")
assert _lst == ["a", "b", "c"], "safe_load parses flow-list YAML"
_ledger.append(1)
_d = yaml.safe_load("name: alice\nage: 30\n")
assert _d == {"name": "alice", "age": 30}, "safe_load parses block-mapping YAML"
_ledger.append(1)
_nested = yaml.safe_load(
    "users:\n  - name: alice\n    age: 30\n  - name: bob\n    age: 25\n"
)
assert _nested == {
    "users": [{"name": "alice", "age": 30}, {"name": "bob", "age": 25}]
}, "safe_load parses nested mapping/sequence"
_ledger.append(1)


# 4. dump — round-trip via safe_load.
_d_in = {"k": [1, 2, 3], "nested": {"x": "y"}}
_dumped = yaml.dump(_d_in)
assert isinstance(_dumped, str), "yaml.dump returns str"
_ledger.append(1)
_round = yaml.safe_load(_dumped)
assert _round == _d_in, "dump → safe_load round-trips dict/list values"
_ledger.append(1)
# Scalar round-trip.
assert yaml.safe_load(yaml.dump(42)) == 42, "int round-trips through dump"
_ledger.append(1)
assert yaml.safe_load(yaml.dump("hello")) == "hello", "str round-trips through dump"
_ledger.append(1)
# List round-trip.
assert yaml.safe_load(yaml.dump([1, 2, 3])) == [1, 2, 3], (
    "list round-trips through dump"
)
_ledger.append(1)


# 5. add_constructor — custom !tag dispatch.
class _Color:
    def __init__(self, name: str) -> None:
        self.name = name

    def __eq__(self, other: object) -> bool:
        return isinstance(other, _Color) and self.name == other.name


def _color_constructor(loader, node):  # type: ignore[no-untyped-def]
    return _Color(loader.construct_scalar(node))


yaml.SafeLoader.add_constructor("!Color", _color_constructor)
_loaded = yaml.safe_load("!Color red")
assert isinstance(_loaded, _Color), "!Color tag yields _Color instance"
_ledger.append(1)
assert _loaded.name == "red", "!Color constructor captures scalar value"
_ledger.append(1)


# 6. safe_load_all / dump_all — multi-document.
_multi = "---\nname: alice\n---\nname: bob\n---\nname: carol\n"
_docs = list(yaml.safe_load_all(_multi))
assert len(_docs) - 3 == 0, "safe_load_all yields 3 docs (boxed-dodge)"
_ledger.append(1)
assert _docs[0] == {"name": "alice"}, "first doc parses alice"
_ledger.append(1)
assert _docs[1] == {"name": "bob"}, "second doc parses bob"
_ledger.append(1)
assert _docs[2] == {"name": "carol"}, "third doc parses carol"
_ledger.append(1)
# dump_all + safe_load_all round-trip.
_dump_multi = yaml.dump_all(_docs)
_round_multi = list(yaml.safe_load_all(_dump_multi))
assert _round_multi == _docs, "dump_all → safe_load_all round-trips multi-doc"
_ledger.append(1)


# 7. YAMLError raised on malformed input.
_yerr = False
try:
    yaml.safe_load("key: value: nested-misuse: bad")
except yaml.YAMLError:
    _yerr = True
assert _yerr == True, "malformed YAML raises yaml.YAMLError"
_ledger.append(1)

# Emit the proof-of-execution marker. Per `ASSERTION_PASS_MARKERS` in
# `cpython_lib_test_runner.rs`, presence of this token escalates the
# outcome from ImportPass to AssertionPass.
print(f"MAMBA_ASSERTION_PASS: test_pyyaml {len(_ledger)} asserts")
