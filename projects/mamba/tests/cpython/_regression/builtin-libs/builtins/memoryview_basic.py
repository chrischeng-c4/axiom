# /// script
# requires-python = ">=3.12"
# dependencies = []
# ///
# memoryview(obj) — minimal byte-flat view (#1256 long-tail tracker,
# sub-priority 4). Stored as Instance(class_name="memoryview") with
# a `_buffer` field holding the bytes/bytearray. `<memory at 0x…>`
# repr is intentionally not exercised here since the address won't
# match CPython byte-for-byte; only behaviour and attribute surface
# are covered.

# bytes-backed view.
mv = memoryview(b"hello")
print(len(mv))                  # 5
print(mv.nbytes)                # 5
print(mv.format)                # B
print(mv.readonly)              # True
print(mv.itemsize)              # 1
print(mv.ndim)                  # 1

# tobytes copies the underlying buffer out as bytes.
print(mv.tobytes())             # b'hello'

# tolist returns the int byte values.
print(mv.tolist())              # [104, 101, 108, 108, 111]

# Empty buffer is well-formed.
mv2 = memoryview(b"")
print(len(mv2))                 # 0
print(mv2.nbytes)               # 0
print(mv2.tobytes())            # b''
print(mv2.tolist())             # []

# bytearray-backed view: tobytes returns a bytes copy.
mv3 = memoryview(bytearray(b"abc"))
print(len(mv3))                 # 3
print(mv3.nbytes)               # 3
print(mv3.tobytes())            # b'abc'
print(mv3.tolist())             # [97, 98, 99]

# release() is a no-op in this minimal implementation; CPython returns None.
print(mv.release())             # None
