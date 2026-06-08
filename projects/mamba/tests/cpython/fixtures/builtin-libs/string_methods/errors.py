# /// script
# requires-python = ">=3.12"
# dependencies = []
# ///
"""str methods: documented exception paths (CPython 3.12 oracle)."""


# index of missing substring raises ValueError.
try:
    "abc".index("xyz")
    print("missing_idx: no_raise")
except ValueError as e:
    print("missing_idx:", type(e).__name__, str(e)[:60])


# encode with bad encoding raises LookupError.
try:
    "hi".encode("no_such_codec")
    print("bad_codec: no_raise")
except LookupError as e:
    print("bad_codec:", type(e).__name__, str(e)[:60])


# encode strict for non-ASCII to ascii raises UnicodeEncodeError.
try:
    "☃".encode("ascii")
    print("ascii_snowman: no_raise")
except UnicodeEncodeError as e:
    print("ascii_snowman:", type(e).__name__, str(e)[:60])


# format with bad spec raises ValueError.
try:
    "{:Q}".format(1)
    print("bad_spec: no_raise")
except ValueError as e:
    print("bad_spec:", type(e).__name__, str(e)[:60])


# format with missing field raises KeyError or IndexError.
try:
    "{missing}".format()
    print("missing_field: no_raise")
except KeyError as e:
    print("missing_field:", type(e).__name__, str(e)[:60])


# split with empty separator raises ValueError.
try:
    "abc".split("")
    print("empty_sep: no_raise")
except ValueError as e:
    print("empty_sep:", type(e).__name__, str(e)[:60])


# str + int raises TypeError.
try:
    "a" + 1  # type: ignore[operator]
    print("str_plus_int: no_raise")
except TypeError as e:
    print("str_plus_int:", type(e).__name__, str(e)[:60])


# str * str raises TypeError.
try:
    "a" * "b"  # type: ignore[operator]
    print("str_times_str: no_raise")
except TypeError as e:
    print("str_times_str:", type(e).__name__, str(e)[:60])


# Indexing OOR raises IndexError.
try:
    "abc"[10]
    print("oor: no_raise")
except IndexError as e:
    print("oor:", type(e).__name__, str(e)[:60])


# rindex of missing substring raises ValueError (like index).
try:
    "abc".rindex("z")
    print("rindex_miss: no_raise")
except ValueError as e:
    print("rindex_miss:", type(e).__name__, str(e)[:60])


# join with a non-str element raises TypeError.
try:
    ",".join(["a", 1])  # type: ignore[list-item]
    print("join_nonstr: no_raise")
except TypeError as e:
    print("join_nonstr:", type(e).__name__, str(e)[:60])


# format_map with a non-mapping positional raises TypeError.
try:
    "{a}".format_map([])  # type: ignore[arg-type]
    print("format_map_nonmap: no_raise")
except TypeError as e:
    print("format_map_nonmap:", type(e).__name__, str(e)[:60])


# format_map with a missing key raises KeyError.
try:
    "{a}".format_map({})
    print("format_map_missing: no_raise")
except KeyError as e:
    print("format_map_missing:", type(e).__name__, str(e)[:60])


# expandtabs that would overflow the result size raises OverflowError.
import sys
try:
    "t\tt".expandtabs(sys.maxsize)
    print("expandtabs_overflow: no_raise")
except OverflowError as e:
    print("expandtabs_overflow:", type(e).__name__, str(e)[:60])


# A multi-character fill char for center/ljust/rjust raises TypeError.
try:
    "x".center(5, "ab")
    print("multichar_fill: no_raise")
except TypeError as e:
    print("multichar_fill:", type(e).__name__, str(e)[:60])


# startswith with a non-str / non-tuple prefix raises TypeError.
try:
    "abc".startswith(1)  # type: ignore[arg-type]
    print("startswith_int: no_raise")
except TypeError as e:
    print("startswith_int:", type(e).__name__, str(e)[:60])
