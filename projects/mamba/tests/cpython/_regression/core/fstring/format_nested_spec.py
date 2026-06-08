# Regression: nested `{...}` inside a format spec must resolve to the next
# positional arg before the spec is applied. Previously the parser cut
# the field at the first `}` so `"{:{}}".format("hi", 10)` parsed the
# field as ":{" and produced `"hi}"`.

# Width via nested spec
print("{:{}}".format("hi", 10))
print("{:>{}}".format("hi", 10))
print("{:<{}}".format("hi", 10))
print("{:^{}}".format("hi", 10))

# Width + type char
print("{:{}d}".format(42, 6))

# Precision via nested spec
print("{:.{}f}".format(3.14159, 3))

# Explicit positional index for both outer and inner
print("{0:{1}}".format("hello", 15))

# Multiple placeholders with nested widths mixed in
print("[{:{}}][{:{}}]".format("a", 5, "b", 5))
