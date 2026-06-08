# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "nntplib"
# dimension = "behavior"
# case = "misc_tests__test_decode_header"
# subject = "cpython.test_nntplib.MiscTests.test_decode_header"
# kind = "semantic"
# xfail = "auto-ported CPython test; mamba promotion pending"
# mem_carveout = ""
# source = "Lib/test/test_nntplib.py"
# status = "filled"
# ///
# mamba-xfail: auto-ported CPython test; mamba promotion pending
# Auto-ported from CPython 3.12 test_nntplib.py::MiscTests::test_decode_header
"""Auto-ported test: MiscTests::test_decode_header (CPython 3.12 oracle)."""


import warnings

with warnings.catch_warnings():
    warnings.simplefilter("ignore", DeprecationWarning)
    import nntplib


def gives(raw, expected):
    assert nntplib.decode_header(raw) == expected, raw


gives("", "")
gives("a plain header", "a plain header")
gives(" with extra  spaces ", " with extra  spaces ")
gives("=?ISO-8859-15?Q?D=E9buter_en_Python?=", "Débuter en Python")
gives(
    "=?utf-8?q?Re=3A_=5Bsqlite=5D_probl=C3=A8me_avec_ORDER_BY_sur_des_cha?="
    " =?utf-8?q?=C3=AEnes_de_caract=C3=A8res_accentu=C3=A9es?=",
    "Re: [sqlite] problème avec ORDER BY sur des chaînes de caractères accentuées",
)
gives("Re: =?UTF-8?B?cHJvYmzDqG1lIGRlIG1hdHJpY2U=?=", "Re: problème de matrice")
gives("Re: Message d'erreur incompréhensible (par moi)", "Re: Message d'erreur incompréhensible (par moi)")

print("MiscTests::test_decode_header: ok")
