# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "pickle"
# dimension = "behavior"
# case = "primitive_roundtrip_preserves_type"
# subject = "pickle.loads"
# kind = "semantic"
# xfail = "pickle shim has no bytes serialization branch; bytes serialize to the 'N' sentinel (src/runtime/stdlib/pickle_mod.rs:220)"
# mem_carveout = ""
# source = "Lib/test/test_pickle.py"
# status = "filled"
# ///
"""pickle.loads: every primitive (int/float/bool/None/str including unicode/bytes including empty) round-trips through dumps+loads equal to itself and with its exact type preserved"""
import pickle

primitives = [
    42, -1, 0, 3.14, -2.5, 1e100,
    True, False, None,
    "hello", "", "unicode: 中文",
    b"bytes", b"",
]
for v in primitives:
    rt = pickle.loads(pickle.dumps(v))
    assert rt == v, f"prim round-trip {type(v).__name__}: {v!r}"
    assert type(rt) == type(v), f"prim type preserved for {v!r}: {type(rt)!r}"

print("primitive_roundtrip_preserves_type OK")
