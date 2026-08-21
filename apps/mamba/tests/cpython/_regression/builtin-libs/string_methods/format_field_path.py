# /// script
# requires-python = ">=3.12"
# dependencies = []
# ///
# Format-string field-path: index (`[k]`) and attribute (`.x`).
#
# CPython's str.format / f-string field-name grammar is
# `name (\. name | \[ key \])*`. Mamba's `mb_str_format` /
# `mb_str_format_kwargs` only parsed the head — anything after a `.` or
# `[` was treated as part of a keyword name, so:
#
#   "{0[0]}".format([10, 20, 30])     # → literal "{0[0]}"
#   "{0[name]}".format({"name": ...}) # → literal "{0[name]}"
#   "{0.x}".format(C())               # → literal "{0.x}"
#
# Fix in `runtime/string_ops.rs`:
#   - Add `split_field_head(name)` that finds the first `.` or `[` and
#     splits the field name into head + path.
#   - Add `resolve_field_path(val, path)` that walks `.attr` (via
#     `class::mb_getattr`) and `[key]` (List / Tuple by int index, Dict
#     by str key — including numeric-string keys) segments.
#   - Wire both into `mb_str_format` (positional) and
#     `mb_str_format_kwargs` (keyword-aware) so the head resolves like
#     before and the path is applied on top. Failures fall back to the
#     existing literal-placeholder behaviour rather than panicking.

# List index.
print("{0[0]}".format([10, 20, 30]))             # 10
print("{0[2]}".format([10, 20, 30]))             # 30

# Tuple index.
print("{0[1]}".format((1, 2, 3)))                # 2

# Dict by string key.
d = {"name": "alice", "age": 30}
print("{0[name]}".format(d))                     # alice
print("{0[age]}".format(d))                      # 30

# Auto-positional with path.
print("{[0]}".format([7, 8, 9]))                 # 7
print("{[name]}".format(d))                      # alice

# Attribute access.
class C:
    def __init__(self, x, y):
        self.x = x
        self.y = y

c = C(11, 22)
print("{0.x}".format(c))                         # 11
print("{0.y}".format(c))                         # 22
print("{.x}".format(c))                          # 11   (auto-positional + .attr)

# Mixed with format-spec.
print("{0[0]:>5}".format([42, 99]))              # "   42"
print("{0.x:04d}".format(c))                     # 0011

# str.format kwargs route.
print("{point.x}, {point.y}".format(point=c))    # 11, 22
print("{d[name]}".format(d=d))                   # alice

# Multiple steps in one field — chained `.` (path traversal).
class Box:
    def __init__(self, inner): self.inner = inner

box = Box(C(1, 2))
print("{0.inner.x}".format(box))                 # 1
print("{0.inner.y}".format(box))                 # 2
