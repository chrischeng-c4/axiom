# /// script
# requires-python = ">=3.12"
# dependencies = []
# ///
"""builtins: documented exception paths (CPython 3.12 oracle)."""


# len() of non-sized raises TypeError.
try:
    len(42)  # type: ignore[arg-type]
    print("len_int: no_raise")
except TypeError as e:
    print("len_int:", type(e).__name__, str(e)[:60])


# iter() of non-iterable raises TypeError.
try:
    iter(42)  # type: ignore[call-overload]
    print("iter_int: no_raise")
except TypeError as e:
    print("iter_int:", type(e).__name__, str(e)[:60])


# next() of empty iterator raises StopIteration.
try:
    next(iter([]))
    print("next_empty: no_raise")
except StopIteration as e:
    print("next_empty:", type(e).__name__, str(e)[:40] or "(no msg)")


# next() with default returns it (no raise).
print("next_default:", next(iter([]), "fallback"))


# pow with negative int exp on int requires modulus or raises ValueError.
try:
    pow(2, -1)
    print("neg_pow: no_raise (returns 0.5)")
except ValueError as e:
    print("neg_pow:", type(e).__name__, str(e)[:60])


# divmod by zero raises ZeroDivisionError.
try:
    divmod(1, 0)
    print("divmod_zero: no_raise")
except ZeroDivisionError as e:
    print("divmod_zero:", type(e).__name__, str(e)[:60])


# abs of non-numeric raises TypeError.
try:
    abs("string")  # type: ignore[arg-type]
    print("abs_str: no_raise")
except TypeError as e:
    print("abs_str:", type(e).__name__, str(e)[:60])


# zip strict=True with mismatched lengths raises ValueError.
try:
    list(zip([1, 2], [1], strict=True))
    print("zip_strict_mismatch: no_raise")
except ValueError as e:
    print("zip_strict_mismatch:", type(e).__name__, str(e)[:60])


# eval of bad syntax raises SyntaxError.
try:
    eval("1 +")
    print("eval_bad: no_raise")
except SyntaxError as e:
    print("eval_bad:", type(e).__name__, str(e)[:60])


# exec of bad syntax raises SyntaxError.
try:
    exec("def 0bad():")
    print("exec_bad: no_raise")
except SyntaxError as e:
    print("exec_bad:", type(e).__name__, str(e)[:60])


# Open non-existent file raises FileNotFoundError.
try:
    open("/no/such/file_for_open_xyzzy")
    print("open_missing: no_raise")
except FileNotFoundError as e:
    print("open_missing:", type(e).__name__, str(e)[:60])


# hash() of an unhashable container raises TypeError.
try:
    hash([1, 2, 3])
    print("hash_list: no_raise")
except TypeError as e:
    print("hash_list:", type(e).__name__, str(e)[:60])


# round() on an object lacking __round__ raises TypeError.
class NoRound:
    pass


try:
    round(NoRound())  # type: ignore[arg-type]
    print("round_noround: no_raise")
except TypeError as e:
    print("round_noround:", type(e).__name__, str(e)[:60])


# sorted() rejects its iterable passed as a keyword.
try:
    sorted(iterable=[])  # type: ignore[call-overload]
    print("sorted_kwarg: no_raise")
except TypeError as e:
    print("sorted_kwarg:", type(e).__name__, str(e)[:60])


# sorted() rejects a positional key argument (key is keyword-only).
try:
    sorted([], None)  # type: ignore[arg-type]
    print("sorted_poskey: no_raise")
except TypeError as e:
    print("sorted_poskey:", type(e).__name__, str(e)[:60])


# divmod() with no arguments raises TypeError.
try:
    divmod()  # type: ignore[call-arg]
    print("divmod_noargs: no_raise")
except TypeError as e:
    print("divmod_noargs:", type(e).__name__, str(e)[:60])


# max() of an empty iterable with no default raises ValueError.
try:
    max([])
    print("max_empty: no_raise")
except ValueError as e:
    print("max_empty:", type(e).__name__, str(e)[:60])


# type() with non-tuple bases raises TypeError.
try:
    type("X", [], {})  # type: ignore[arg-type]
    print("type_badbases: no_raise")
except TypeError as e:
    print("type_badbases:", type(e).__name__, str(e)[:60])
