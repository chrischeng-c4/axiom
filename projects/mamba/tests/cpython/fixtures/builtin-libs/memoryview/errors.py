# /// script
# requires-python = ">=3.12"
# dependencies = []
# ///
"""memoryview: documented exception paths (CPython 3.12 oracle)."""


# memoryview of non-buffer raises TypeError.
try:
    memoryview("string")  # type: ignore[arg-type]
    print("str_view: no_raise")
except TypeError as e:
    print("str_view:", type(e).__name__, str(e)[:60])


# Indexing OOR raises IndexError.
mv = memoryview(b"hello")
try:
    mv[10]
    print("oor: no_raise")
except IndexError as e:
    print("oor:", type(e).__name__, str(e)[:60])


# Releasing then accessing raises ValueError.
mv.release()
try:
    mv[0]
    print("after_release: no_raise")
except ValueError as e:
    print("after_release:", type(e).__name__, str(e)[:60])


# Bad cast format raises ValueError.
buf = bytearray(8)
mv2 = memoryview(buf)
try:
    mv2.cast("no_such_format")
    print("bad_cast: no_raise")
except ValueError as e:
    print("bad_cast:", type(e).__name__, str(e)[:60])
finally:
    mv2.release()


# Assigning to immutable memoryview (over bytes) raises TypeError.
buf2 = b"hello"
mv3 = memoryview(buf2)
try:
    mv3[0] = ord("H")  # type: ignore[index]
    print("set_immutable: no_raise")
except TypeError as e:
    print("set_immutable:", type(e).__name__, str(e)[:60])


# Wrong-length assignment over a writable view raises ValueError.
buf3 = bytearray(b"hello")
mv4 = memoryview(buf3)
try:
    mv4[:5] = b"ab"  # too short
    print("wrong_len: no_raise")
except ValueError as e:
    print("wrong_len:", type(e).__name__, str(e)[:60])


# Happy: writable view modifies underlying buffer.
mv4[0] = ord("H")
print("after_write:", buf3)
mv4.release()


# Hashing a writable view raises ValueError (only read-only views hash).
try:
    hash(memoryview(bytearray(b"abc")))
    print("hash_mutable: no_raise")
except ValueError as e:
    print("hash_mutable:", type(e).__name__, str(e)[:60])


# memoryview is not copyable: copy.copy raises TypeError.
import copy
try:
    copy.copy(memoryview(b"abc"))
    print("copy: no_raise")
except TypeError as e:
    print("copy:", type(e).__name__, str(e)[:60])


# memoryview is not picklable: pickle.dumps raises TypeError.
import pickle
try:
    pickle.dumps(memoryview(b"abc"))
    print("pickle: no_raise")
except TypeError as e:
    print("pickle:", type(e).__name__, str(e)[:60])
