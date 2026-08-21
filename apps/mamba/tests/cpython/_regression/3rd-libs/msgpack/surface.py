"""Surface contract for third-party msgpack package.

# type-regime: monomorphic

Probes: msgpack.pack, msgpack.unpack, msgpack.Packer,
msgpack.Unpacker, msgpack.packb, msgpack.unpackb.
CPython 3.12 is the oracle.
"""

import msgpack  # type: ignore[import]

# Core API
assert hasattr(msgpack, "pack"), "pack"
assert hasattr(msgpack, "unpack"), "unpack"
assert hasattr(msgpack, "packb"), "packb"
assert hasattr(msgpack, "unpackb"), "unpackb"
assert hasattr(msgpack, "Packer"), "Packer"
assert hasattr(msgpack, "Unpacker"), "Unpacker"
assert hasattr(msgpack, "PackException"), "PackException"
assert hasattr(msgpack, "UnpackException"), "UnpackException"
assert hasattr(msgpack, "version"), "version"

# version is a tuple
assert isinstance(msgpack.version, tuple), \
    f"version type = {type(msgpack.version)!r}"
assert len(msgpack.version) >= 2, f"version len = {len(msgpack.version)}"

# Callables
assert callable(msgpack.pack), "pack callable"
assert callable(msgpack.unpack), "unpack callable"
assert callable(msgpack.packb), "packb callable"
assert callable(msgpack.unpackb), "unpackb callable"
assert callable(msgpack.Packer), "Packer callable"
assert callable(msgpack.Unpacker), "Unpacker callable"

# packb/unpackb round-trip
_data = {"name": "Alice", "score": 42, "tags": ["a", "b"]}
_packed = msgpack.packb(_data, use_bin_type=True)
assert isinstance(_packed, bytes), f"packed type = {type(_packed)!r}"
_unpacked = msgpack.unpackb(_packed, raw=False)
assert _unpacked == _data, f"round-trip = {_unpacked!r}"

# Packer construction
_packer = msgpack.Packer(use_bin_type=True)
assert hasattr(_packer, "pack"), "packer.pack"
assert callable(_packer.pack), "packer.pack callable"

# Module attributes stable
_p_ref = msgpack.pack
assert msgpack.pack is _p_ref, "pack stable"
_u_ref = msgpack.unpack
assert msgpack.unpack is _u_ref, "unpack stable"
_pp_ref = msgpack.Packer
assert msgpack.Packer is _pp_ref, "Packer stable"
_up_ref = msgpack.Unpacker
assert msgpack.Unpacker is _up_ref, "Unpacker stable"

print("surface OK")
