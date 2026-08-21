# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "extcall"
# dimension = "behavior"
# case = "wide_arity_unpack_call"
# subject = "function call: *args-unpacked positional arity beyond the JIT dispatch ceiling"
# kind = "semantic"
# xfail = ""
# mem_carveout = ""
# source = "mamba issue #1950"
# status = "filled"
# ///
"""*args-unpacked calls at arities that cross the old fixed dynamic-dispatch
ceiling (16 params, #1754). CPython has no such ceiling; mamba's JIT frame
binder previously hard-errored above it (#1950). 16 is the prior boundary
(must keep working); 17/18/32/64 previously raised
"JIT call frame arity N exceeds the supported dispatch ceiling"."""


def g16(a1, a2, a3, a4, a5, a6, a7, a8, a9, a10, a11, a12, a13, a14, a15, a16):
    return (a16 - a1, a1, a16)


assert g16(*range(1, 17)) == (15, 1, 16), "arity-16 *args-unpacked call"


def g17(a1, a2, a3, a4, a5, a6, a7, a8, a9, a10, a11, a12, a13, a14, a15, a16, a17):
    return (a17 - a1, a1, a17)


assert g17(*range(1, 18)) == (16, 1, 17), "arity-17 *args-unpacked call"


def g18(a1, a2, a3, a4, a5, a6, a7, a8, a9, a10, a11, a12, a13, a14, a15, a16, a17, a18):
    return (a18 - a1, a1, a18)


assert g18(*range(1, 19)) == (17, 1, 18), "arity-18 *args-unpacked call"


def g32(a1, a2, a3, a4, a5, a6, a7, a8, a9, a10, a11, a12, a13, a14, a15, a16, a17, a18, a19, a20, a21, a22, a23, a24, a25, a26, a27, a28, a29, a30, a31, a32):
    return (a32 - a1, a1, a32)


assert g32(*range(1, 33)) == (31, 1, 32), "arity-32 *args-unpacked call"


def g64(a1, a2, a3, a4, a5, a6, a7, a8, a9, a10, a11, a12, a13, a14, a15, a16, a17, a18, a19, a20, a21, a22, a23, a24, a25, a26, a27, a28, a29, a30, a31, a32, a33, a34, a35, a36, a37, a38, a39, a40, a41, a42, a43, a44, a45, a46, a47, a48, a49, a50, a51, a52, a53, a54, a55, a56, a57, a58, a59, a60, a61, a62, a63, a64):
    return (a64 - a1, a1, a64)


assert g64(*range(1, 65)) == (63, 1, 64), "arity-64 *args-unpacked call"

print("wide_arity_unpack_call OK")
