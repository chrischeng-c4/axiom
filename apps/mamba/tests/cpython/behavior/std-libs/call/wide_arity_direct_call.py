# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "call"
# dimension = "behavior"
# case = "wide_arity_direct_call"
# subject = "function call: literal positional arity beyond the JIT dispatch ceiling (direct call, no spread)"
# kind = "semantic"
# xfail = ""
# mem_carveout = ""
# source = "mamba issue #1950"
# status = "filled"
# ///
"""Direct literal-positional calls (no *args/**kwargs spread at the call
site) at arities that cross the old fixed dynamic-dispatch ceiling (16
params, #1754). These calls compile through the static known-callee path
(compile-time reorder, no ceiling) rather than the dynamic dispatcher, but
#1950's acceptance criteria explicitly cover direct calls alongside
*args-unpacked ones, so this fixture pins the invariant permanently. 16 is
the prior dynamic-dispatch boundary (must keep working); 17/18/32/64 match
the wide_arity_unpack_call.py / wide_arity_kwargs_mixed_call.py coverage in
the extcall bucket."""


def g16(a1, a2, a3, a4, a5, a6, a7, a8, a9, a10, a11, a12, a13, a14, a15, a16):
    return (a16 - a1, a1, a16)


assert g16(1, 2, 3, 4, 5, 6, 7, 8, 9, 10, 11, 12, 13, 14, 15, 16) == (15, 1, 16), "arity-16 direct positional call"


def g17(a1, a2, a3, a4, a5, a6, a7, a8, a9, a10, a11, a12, a13, a14, a15, a16, a17):
    return (a17 - a1, a1, a17)


assert g17(1, 2, 3, 4, 5, 6, 7, 8, 9, 10, 11, 12, 13, 14, 15, 16, 17) == (16, 1, 17), "arity-17 direct positional call"


def g18(a1, a2, a3, a4, a5, a6, a7, a8, a9, a10, a11, a12, a13, a14, a15, a16, a17, a18):
    return (a18 - a1, a1, a18)


assert g18(1, 2, 3, 4, 5, 6, 7, 8, 9, 10, 11, 12, 13, 14, 15, 16, 17, 18) == (17, 1, 18), "arity-18 direct positional call"


def g32(a1, a2, a3, a4, a5, a6, a7, a8, a9, a10, a11, a12, a13, a14, a15, a16, a17, a18, a19, a20, a21, a22, a23, a24, a25, a26, a27, a28, a29, a30, a31, a32):
    return (a32 - a1, a1, a32)


assert g32(1, 2, 3, 4, 5, 6, 7, 8, 9, 10, 11, 12, 13, 14, 15, 16, 17, 18, 19, 20, 21, 22, 23, 24, 25, 26, 27, 28, 29, 30, 31, 32) == (31, 1, 32), "arity-32 direct positional call"


def g64(a1, a2, a3, a4, a5, a6, a7, a8, a9, a10, a11, a12, a13, a14, a15, a16, a17, a18, a19, a20, a21, a22, a23, a24, a25, a26, a27, a28, a29, a30, a31, a32, a33, a34, a35, a36, a37, a38, a39, a40, a41, a42, a43, a44, a45, a46, a47, a48, a49, a50, a51, a52, a53, a54, a55, a56, a57, a58, a59, a60, a61, a62, a63, a64):
    return (a64 - a1, a1, a64)


assert g64(1, 2, 3, 4, 5, 6, 7, 8, 9, 10, 11, 12, 13, 14, 15, 16, 17, 18, 19, 20, 21, 22, 23, 24, 25, 26, 27, 28, 29, 30, 31, 32, 33, 34, 35, 36, 37, 38, 39, 40, 41, 42, 43, 44, 45, 46, 47, 48, 49, 50, 51, 52, 53, 54, 55, 56, 57, 58, 59, 60, 61, 62, 63, 64) == (63, 1, 64), "arity-64 direct positional call"

print("wide_arity_direct_call OK")
