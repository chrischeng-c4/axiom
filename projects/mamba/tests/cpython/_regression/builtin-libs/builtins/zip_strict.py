# /// script
# requires-python = ">=3.12"
# dependencies = []
# ///
# `zip(*iters, strict=True)` (PEP 618, Py3.10+) was unsupported. The
# `strict` kwarg was forwarded as a positional bool, which produced a
# Cranelift verifier failure rather than CPython's length-validation:
#
#   list(zip([1, 2, 3], "ab", strict=True))
#   # mamba → "error: codegen error: ... mismatched argument count"
#   # CPython → ValueError: zip() argument 2 is shorter than argument 1
#
# Fix:
#   - `IterKind::Zip` carries a `strict: bool` flag.
#   - New `mb_zip_strict(iterables_list, strict)` runtime entry; the
#     ast_to_hir lowerer routes `zip(*args, strict=...)` through it so
#     keyword args don't leak into the codegen path.
#   - `advance_iter` for Zip raises ValueError with CPython's exact
#     wording — "argument N is shorter/longer than argument 1" — when
#     strict mode detects a length mismatch.

# Equal-length: strict and non-strict agree.
print(list(zip([1, 2, 3], "abc")))                  # [(1,'a'),(2,'b'),(3,'c')]
print(list(zip([1, 2, 3], "abc", strict=True)))     # same
print(list(zip([1, 2, 3], "abc", strict=False)))    # same

# Non-strict tolerates length mismatch (truncates to shortest).
print(list(zip([1, 2, 3], "ab")))                   # [(1,'a'),(2,'b')]
print(list(zip([1, 2, 3], "ab", strict=False)))     # same

# Strict mode: shorter follower → "argument 2 is shorter".
try:
    list(zip([1, 2, 3], "ab", strict=True))
except ValueError as e:
    print("VE-shorter:", e)

# Strict mode: longer follower → "argument 2 is longer".
try:
    list(zip([1, 2], "abc", strict=True))
except ValueError as e:
    print("VE-longer:", e)

# Strict mode with three iterables, second runs out first.
try:
    list(zip("abcd", "ab", "abcd", strict=True))
except ValueError as e:
    print("VE-three-mid:", e)

# Strict mode with three iterables, first runs out first → reports
# the first peer that still has values.
try:
    list(zip("ab", "abcd", "ab", strict=True))
except ValueError as e:
    print("VE-three-first:", e)

# Two-iterable equal length still works in strict mode.
print(list(zip([10, 20, 30], [1, 2, 3], strict=True)))   # [(10,1),(20,2),(30,3)]

# strict=False is the default; explicit False matches the bare form.
print(list(zip([1, 2, 3, 4], [10, 20])))            # [(1,10),(2,20)]
print(list(zip([1, 2, 3, 4], [10, 20], strict=False)))  # same
