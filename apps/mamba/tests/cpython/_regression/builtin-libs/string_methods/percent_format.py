# /// script
# requires-python = ">=3.12"
# dependencies = []
# ///
# Regression: % string formatting. Previously the type checker rejected
# `"%d" % 42` as a type mismatch and the runtime had no implementation.

# Integer
print("%d" % 42)
print("%i" % 7)
print("%d %d" % (1, 2))
print("%5d" % 3)
print("%-5d|" % 3)
print("%05d" % 3)
print("%+d" % 5)
print("%+d" % -5)
print("% d" % 5)

# String
print("%s %s" % ("hello", "world"))
print("%10s|" % "hi")
print("%-10s|" % "hi")
print("%.3s" % "abcdef")

# Float
print("%f" % 3.14)
print("%.2f" % 3.14159)
print("%10.3f" % 3.14159)
print("%-8.2f|" % 3.14)

# Radix
print("%x" % 255)
print("%X" % 255)
print("%o" % 8)
print("%#x" % 255)
print("%#o" % 8)

# Char
print("%c" % 65)
print("%c" % 97)

# repr
print("%r" % "hello")

# Mixed
print("[%s] = %d (%#x)" % ("count", 42, 42))

# Single-arg via scalar (not tuple) still works
print("value: %d" % 100)
