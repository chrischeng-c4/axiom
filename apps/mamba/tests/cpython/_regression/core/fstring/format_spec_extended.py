# Regression: format spec was missing sign, thousands separator, and the
# b/o/x/X integer radix types — .format() silently fell back to the raw
# decimal value when any of these appeared in the spec.

# Sign
print("{:+d}".format(5))
print("{:+d}".format(-5))
print("{: d}".format(5))
print("{: d}".format(-5))
print("{:-d}".format(5))

# Thousands separator
print("{:,}".format(1234567))
print("{:,}".format(-1234567))
print("{:,d}".format(1000))
print("{:,.2f}".format(1234567.89))

# Radix types
print("{:b}".format(5))
print("{:b}".format(255))
print("{:o}".format(8))
print("{:o}".format(64))
print("{:x}".format(255))
print("{:X}".format(255))

# Alternate form (# flag)
print("{:#b}".format(5))
print("{:#o}".format(8))
print("{:#x}".format(255))
print("{:#X}".format(255))

# Zero-pad keeps sign on the outside
print("{:+05d}".format(3))
print("{:+05d}".format(-3))
print("{: 05d}".format(3))

# Existing features continue to work
print("{:5d}".format(3))
print("{:>10}".format("hi"))
print("{:.3f}".format(3.14159))
