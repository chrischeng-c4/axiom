# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "struct"
# dimension = "behavior"
# case = "mixed_record_all_byteorders"
# subject = "struct.pack"
# kind = "semantic"
# xfail = ""
# mem_carveout = ""
# source = ""
# status = "filled"
# ///
"""struct.pack: a heterogeneous 'cbHid?' record round-trips identically under every byte-order prefix ('', '@', '<', '>', '=', '!')"""
import struct

# Round-trip a heterogeneous record under every byte-order prefix.
for prefix in ("", "@", "<", ">", "=", "!"):
    fmt = prefix + "cbHid?"
    packed = struct.pack(fmt, b"a", 1, 255, 65535, 3.5, True)
    c, b, h, i, d, flag = struct.unpack(fmt, packed)
    assert c == b"a" and b == 1 and h == 255, f"int round-trip ({prefix})"
    assert i == 65535 and d == 3.5 and flag is True, f"mixed round-trip ({prefix})"

print("mixed_record_all_byteorders OK")
