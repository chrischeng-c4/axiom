# /// script
# requires-python = ">=3.12"
# dependencies = []
# ///
# float conversion / coercion patterns broad

# int to float
print(float(5))
print(float(0))
print(float(-3))
print(float(100))

# str to float
print(float("3.14"))
print(float("2.5"))
print(float("0"))
print(float("-1.5"))
print(float("10"))

# bool to float
print(float(True))
print(float(False))

# float to int (truncation)
print(int(3.7))
print(int(3.2))
print(int(-3.7))
print(int(-3.2))
print(int(0.5))
print(int(-0.5))

# float to str
print(str(3.14))
print(str(-2.5))

# arithmetic mixing int/float
print(1 + 2.0)
print(2.0 + 1)
print(3 * 2.5)
print(10.0 / 4)
print(10 / 4.0)
print(7.5 - 2)
print(7 - 2.5)

# float division yields float
print(6.0 / 2)
print(6 / 4)

# floor div mixing
print(7.0 // 2)
print(7 // 2.0)
print(-7.0 // 2)

# modulo float
print(7.5 % 2)
print(-7.5 % 2)

# abs on float
print(abs(3.14))
print(abs(-3.14))
print(abs(0.0))
print(abs(-0.0))

# round int args
print(round(3.14, 1))
print(round(3.14159, 3))
print(round(3.7))
print(round(3.3))

# negative float
print(-3.14)
print(--3.14)
print(-(-3.14))

# unary plus
print(+3.14)
print(+(-3.14))

# comparison int vs float
print(5 == 5.0)
print(5 < 5.5)
print(5.5 > 5)
print(5.0 <= 5)
print(5 >= 5.0)

# zero equivalence
print(0 == 0.0)
print(0.0 == 0)

# PEP 515 — digit-separator underscores in float() string parsing.
print(float("1_000.5"))
print(float("2_500e-3"))
print(float("-1_2.3_4"))
# Invalid placements (adjacent to '.', 'e', sign) raise ValueError.
for bad in ["1_.5", "1._5", "1e_3", "1_e3"]:
    try:
        float(bad)
        print("expected error:", bad)
    except ValueError:
        print("rejected:", bad)
