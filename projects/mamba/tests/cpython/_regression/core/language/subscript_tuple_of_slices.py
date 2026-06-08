# Tuple-of-slices in subscripts (#1670 parser, #1672 runtime).
#
# CPython grammar permits a comma-separated list of slice or expr
# elements inside `[...]`, building an implicit tuple. Mamba's parser
# previously accepted only `m[a, b, c]` (tuple of exprs, #1606) and
# `m[a:b]` (single slice); the mixed form `m[a:b, c:d]` was rejected
# with `expected ], got ,` (parser gap closed in #1670).
#
# The runtime path was also broken for 3-element tuple subscripts on
# user-defined classes — the slice fast-path in mb_obj_getitem
# defaulted unknown object types to mb_list_slice_full and silently
# returned `[]` instead of invoking the dunder `__getitem__`. Fixed
# in #1672 — the fast-path now applies only to the four built-in
# container types (list / tuple / str / bytes); everything else falls
# through to the dunder dispatch.

class M:
    def __getitem__(self, key):
        return key

m = M()

# Two slices.
print(m[1:2, 3:4])

# Slice + expr.
print(m[1:2, 7])

# Expr + slice.
print(m[7, 1:2])

# Two slices with steps.
print(m[1:2:3, 4:5:6])

# Three-element tuple subscript — exercises #1672 (slice fast-path no
# longer swallows the dunder dispatch).
print(m[1, 2, 3])

# Three-element mixed slice + expr — also #1672.
print(m[1:2, 3, 4:5])

# Single slice on custom class — formerly returned `[]` because the
# slice fast-path defaulted to `mb_list_slice_full` on the Instance.
# Now dispatches to `__getitem__` with a 3-tuple key.
print(m[1:2:3])
