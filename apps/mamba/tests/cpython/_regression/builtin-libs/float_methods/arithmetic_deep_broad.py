# /// script
# requires-python = ">=3.12"
# dependencies = []
# ///
# float arithmetic deep broad

# basic arithmetic
print(1.0 + 2.0)
print(1.5 - 0.5)
print(2.0 * 3.5)
print(10.0 / 4.0)
print(10.0 / 3.0)
print(10.0 // 3.0)
print(10.0 % 3.0)

# mixed int/float (Python promotes)
print(1 + 2.0)
print(3.0 + 4)
print(1.5 * 2)
print(5 / 2)  # always float in Py3

# float special values
import math
print(math.pi)
print(math.e)
print(math.inf)

# comparison
print(1.0 == 1)
print(1.5 > 1)
print(1.5 < 2)
print(0.1 + 0.2 > 0.3)  # classic floating

# float from str
print(float("3.14"))
print(float("-2.5"))
print(float("1e3"))
print(float("1.5e-2"))
print(float("0"))

# float from int
print(float(42))
print(float(-5))
print(float(0))

# int from float (truncation)
print(int(3.9))
print(int(-3.9))
print(int(0.5))

# abs
print(abs(1.5))
print(abs(-2.5))
print(abs(0.0))
print(abs(-0.0))

# min/max
print(min(1.5, 2.5))
print(max(1.5, 2.5))
print(min([1.5, 2.5, 0.5]))

# math funcs
print(math.floor(3.7))
print(math.ceil(3.2))
print(math.floor(-3.2))
print(math.ceil(-3.7))

# sum
print(sum([1.5, 2.5, 3.5]))

# sorted
print(sorted([3.14, 1.5, 2.7, 0.5]))
print(sorted([1.1, 2.2, 3.3], reverse=True))

# float in conditions
x = 1.5
if x > 1.0:
    print("big")
if x < 2.0:
    print("small")

# negative floats
print(-1.5)
print(-0.5)
print(-(-1.5))

# float division by float
print(1.0 / 2.0)
print(3.0 / 4.0)
print(7.5 / 2.5)

# power (int exp avoids pow bug)
print(2.0 ** 10)
