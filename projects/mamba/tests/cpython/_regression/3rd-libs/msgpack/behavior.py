"""Behavior contract for third-party msgpack package.

Each block tests one semantic rule. Expected values are what CPython
3.12 produces. mamba PASS = matches CPython exactly.

# type-regime: monomorphic
"""

import msgpack  # type: ignore[import]

# Rule 1: packb/unpackb round-trip for basic types
_cases1 = [
    42,
    "hello",
    3.14,
    True,
    None,
    [1, 2, 3],
    {"key": "value"},
]
for _val in _cases1:
    _packed = msgpack.packb(_val, use_bin_type=True)
    assert isinstance(_packed, bytes), f"packed type = {type(_packed)!r}"
    _back = msgpack.unpackb(_packed, raw=False)
    assert _back == _val, f"round-trip failed for {_val!r}: got {_back!r}"

# Rule 2: bytes are preserved with use_bin_type
_b2 = b"\x00\x01\x02\x03"
_packed2 = msgpack.packb(_b2, use_bin_type=True)
_back2 = msgpack.unpackb(_packed2, raw=False)
assert _back2 == _b2, f"bytes round-trip = {_back2!r}"

# Rule 3: nested dict round-trip
_data3 = {"user": {"name": "Alice", "age": 30, "scores": [10, 20, 30]}}
_packed3 = msgpack.packb(_data3, use_bin_type=True)
_back3 = msgpack.unpackb(_packed3, raw=False)
assert _back3 == _data3, f"nested round-trip = {_back3!r}"

# Rule 4: Packer is stateful
_packer4 = msgpack.Packer(use_bin_type=True)
_packed4a = _packer4.pack(1)
_packed4b = _packer4.pack(2)
assert msgpack.unpackb(_packed4a, raw=False) == 1, "packer pack 1"
assert msgpack.unpackb(_packed4b, raw=False) == 2, "packer pack 2"

# Rule 5: Unpacker is streaming
_data5 = [msgpack.packb(i, use_bin_type=True) for i in range(5)]
_combined5 = b"".join(_data5)
_unpacker5 = msgpack.Unpacker(raw=False)
_unpacker5.feed(_combined5)
_results5 = list(_unpacker5)
assert _results5 == [0, 1, 2, 3, 4], f"streaming = {_results5!r}"

# Rule 6: Module attributes are identity-stable
_p_ref = msgpack.pack
_u_ref = msgpack.unpack
_pp_ref = msgpack.Packer
_up_ref = msgpack.Unpacker
for _ in range(5):
    assert msgpack.pack is _p_ref, "pack stable"
    assert msgpack.unpack is _u_ref, "unpack stable"
    assert msgpack.Packer is _pp_ref, "Packer stable"
    assert msgpack.Unpacker is _up_ref, "Unpacker stable"

print("behavior OK")
