# /// script
# requires-python = ">=3.12"
# dependencies = []
# ///
# mamba-xfail: eval() defers parsing past mamba's strict-typing gate and
# silently returns None for cross-type ops where CPython raises TypeError
# (see project_mamba_eval_silent_none_cross_type memory). All ten cases
# below print a label-prefixed line that includes the exception class
# name; mamba currently prints "no_raise:" for each, which surfaces as
# MISSING_RAISE in the runner.
"""Cross-type operator ops via eval() raise TypeError under CPython 3.12."""


def probe(label: str, expr: str) -> None:
    try:
        result = eval(expr)
        print(f"{label}: no_raise: {result!r}")
    except TypeError as e:
        print(f"{label}: TypeError: {str(e)[:40]}")


probe("int_plus_str",   "1 + 'a'")
probe("str_plus_int",   "'a' + 1")
probe("str_times_float", "'a' * 2.5")
probe("list_plus_tuple", "[1] + (2,)")
probe("tuple_plus_list", "(1,) + [2]")
probe("list_times_float", "[1] * 2.0")
probe("bool_plus_str",  "True + 'a'")
probe("dict_plus_dict", "{1: 'a'} + {2: 'b'}")
probe("neg_str",        "-'a'")
probe("invert_str",     "~'a'")
