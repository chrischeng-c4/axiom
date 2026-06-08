# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "struct"
# dimension = "real_world"
# case = "parse_wire_header"
# subject = "struct.unpack"
# kind = "semantic"
# xfail = ""
# mem_carveout = ""
# source = ""
# status = "filled"
# ///
"""struct.unpack: decode a fixed binary wire-protocol header ('<IBBHQ': magic, version, flags, length, seq) end-to-end: encode then decode round-trips the record and the LE/BE endianness contract holds; struct.error resolves as an attribute"""
import struct

# Wire format: 4-byte magic (LE int) + 1-byte version + 1-byte flags
# + 2-byte payload length (LE u16) + 8-byte sequence (LE u64) = 16 bytes.
HEADER_FMT = "<IBBHQ"
HEADER_SIZE = struct.calcsize(HEADER_FMT)
assert HEADER_SIZE == 16, f"unexpected header size: {HEADER_SIZE}"


def encode(magic, version, flags, length, seq):
    return struct.pack(HEADER_FMT, magic, version, flags, length, seq)


def decode(buf):
    if len(buf) < HEADER_SIZE:
        raise ValueError(f"buffer too short: {len(buf)} < {HEADER_SIZE}")
    return struct.unpack(HEADER_FMT, buf[:HEADER_SIZE])


# Round-trip a realistic record.
encoded = encode(0xDEADBEEF, 2, 0b10000001, 4096, 1234567890)
assert len(encoded) == HEADER_SIZE
magic, version, flags, length, seq = decode(encoded)
assert magic == 0xDEADBEEF, f"magic {magic:#x}"
assert version == 2
assert flags == 0b10000001
assert length == 4096
assert seq == 1234567890

# Endianness contract: little-endian writes the LSB first, big-endian the MSB.
assert struct.pack("<I", 0x12345678) == b"\x78\x56\x34\x12", "LE order"
assert struct.pack(">I", 0x12345678) == b"\x12\x34\x56\x78", "BE order"

# struct.error must resolve as an attribute (callers use it in except/assertRaises).
assert struct.error is not None

print("parse_wire_header OK")
