# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "struct"
# dimension = "security"
# case = "parse_integer_overflow"
# subject = "struct.pack"
# kind = "semantic"
# xfail = "struct shim models no bounds/size checks; out-of-range writes are silently clipped and wrong-size buffers accepted (WI #3929; repo-memory project_mamba_runtime_correctness_gaps_2026_05_13)"
# mem_carveout = ""
# source = ""
# status = "filled"
# ///
"""struct.pack: adversarial hardening: out-of-range pack values (i/B/I/b/H/Q over/under bounds), wrong-size unpack buffers, and malformed/hostile calcsize formats (unknown char, dangling count, embedded NUL, stray marker) must each raise struct.error rather than silently truncate; the boundary values themselves round-trip cleanly"""
import struct


def must_raise(label, fn, *args, **kwargs):
    """Out-of-range / malformed input must raise struct.error, never silently truncate."""
    try:
        fn(*args, **kwargs)
    except struct.error:
        return
    raise AssertionError(f"{label}: expected struct.error, got silent success")


# --- pack: value out of range for the declared format code ---
must_raise("pack_i_too_big", struct.pack, "i", 2 ** 31)
must_raise("pack_i_too_small", struct.pack, "i", -(2 ** 31) - 1)
must_raise("pack_B_256", struct.pack, "B", 256)
must_raise("pack_B_negative", struct.pack, "B", -1)
must_raise("pack_I_negative", struct.pack, "I", -1)
must_raise("pack_b_128", struct.pack, "b", 128)
must_raise("pack_H_overflow", struct.pack, "H", 65536)
must_raise("pack_Q_negative", struct.pack, "Q", -1)

# --- unpack: a buffer of the wrong size must be rejected, not zero-padded ---
must_raise("unpack_short", struct.unpack, "i", b"\x00\x00\x00")
must_raise("unpack_long", struct.unpack, "i", b"\x00\x00\x00\x00\x00")
must_raise("unpack_pair_short", struct.unpack, "ii", b"\x00" * 7)
must_raise("unpack_empty", struct.unpack, "i", b"")

# --- calcsize: malformed / hostile format strings must be rejected ---
must_raise("calcsize_bad_char", struct.calcsize, "z")
must_raise("calcsize_dangling_count", struct.calcsize, "4")
must_raise("calcsize_embedded_nul", struct.calcsize, "2\x00i")
must_raise("calcsize_bad_marker", struct.calcsize, "i@i")

# --- the boundary values themselves must round-trip cleanly (no off-by-one) ---
assert struct.unpack("i", struct.pack("i", 2 ** 31 - 1)) == (2 ** 31 - 1,), "i max"
assert struct.unpack("i", struct.pack("i", -(2 ** 31))) == (-(2 ** 31),), "i min"
assert struct.unpack("B", struct.pack("B", 255)) == (255,), "B max"
assert struct.unpack("B", struct.pack("B", 0)) == (0,), "B min"
assert struct.unpack("H", struct.pack("H", 65535)) == (65535,), "H max"

print("parse_integer_overflow OK")
