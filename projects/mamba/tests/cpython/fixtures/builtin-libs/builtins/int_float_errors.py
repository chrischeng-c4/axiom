# /// script
# requires-python = ">=3.12"
# dependencies = []
# ///
# int()/float() raise ValueError on invalid strings

# int — bad strings
try:
    int("abc")
except ValueError as e:
    print("int abc:", type(e).__name__)

try:
    int("12.3")
except ValueError as e:
    print("int 12.3:", type(e).__name__)

try:
    int("")
except ValueError as e:
    print("int empty:", type(e).__name__)

# int — good strings
print(int("42"))
print(int("  -7  "))
print(int("+100"))

# int with base
print(int("ff", 16))
print(int("0xff", 16))
print(int("101", 2))
print(int("777", 8))

try:
    int("xyz", 16)
except ValueError as e:
    print("int xyz16:", type(e).__name__)

# float — bad strings
try:
    float("not-a-num")
except ValueError as e:
    print("float bad:", type(e).__name__)

try:
    float("")
except ValueError as e:
    print("float empty:", type(e).__name__)

# float — good strings
print(float("3.14"))
print(float("  -2.5  "))
print(float("1e3"))
print(float("inf") > 1e308)
print(float("-inf") < -1e308)
