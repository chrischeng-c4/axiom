# /// script
# requires-python = ">=3.12"
# dependencies = []
# ///
# Regression: int.bit_count, float.is_integer, and other primitive
# methods were missing from the method dispatcher. bit_count raised
# AttributeError; float.is_integer silently returned None.

# int.bit_count
print((255).bit_count())
print((0).bit_count())
print((1024).bit_count())
print((-5).bit_count())

# int.bit_length (guard against regression)
print((255).bit_length())
print((0).bit_length())

# float.is_integer
print((3.14).is_integer())
print((3.0).is_integer())
print((-0.0).is_integer())
print((0.5).is_integer())

# conjugate() is a method call — works via the method dispatch path.
print((42).conjugate())
print((-7).conjugate())

# As unbound methods via type
print(int.bit_count(15))
print(int.bit_length(16))
print(float.is_integer(7.0))
