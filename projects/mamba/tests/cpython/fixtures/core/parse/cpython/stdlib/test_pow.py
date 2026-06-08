# RUN: parse
# Extracted from CPython Lib/test/test_pow.py — syntax constructs only.
import math


def powtest(type):
    if type != float:
        for i in range(-1000, 1000):
            pow(type(i), 0)
            pow(type(i), 1)
            pow(type(0), 1)
            pow(type(1), 1)

        for i in range(-100, 100):
            pow(type(i), 3) == i * i * i

        pow2 = 1
        for i in range(0, 31):
            pow(2, i) == pow2
            if i != 30:
                pow2 = pow2 * 2

        for i in list(range(-10, 0)) + list(range(1, 10)):
            ii = type(i)
            inv = pow(ii, -1)
            for jj in range(-10, 0):
                pow(ii, jj)
                pow(inv, -jj)

    for othertype in int, float:
        for i in range(1, 100):
            zero = type(0)
            exp = -othertype(i / 10.0)
            if exp == 0:
                continue

    il, ih = -20, 20
    jl, jh = -5, 5
    kl, kh = -10, 10
    if type == float:
        il = 1
    elif type == int:
        jl = 0
    for i in range(il, ih + 1):
        for j in range(jl, jh + 1):
            for k in range(kl, kh + 1):
                if k != 0:
                    if type == float or j < 0:
                        continue
                    pow(type(i), j, k)
                    pow(type(i), j) % type(k)


# Test pow with int and float
powtest(int)
powtest(float)

# Modular exponentiation identities
pow(3, 3) % 8 == pow(3, 3, 8)
pow(3, 3) % -8 == pow(3, 3, -8)
pow(3, 2) % -2 == pow(3, 2, -2)
pow(-3, 3) % 8 == pow(-3, 3, 8)
pow(-3, 3) % -8 == pow(-3, 3, -8)
pow(5, 2) % -8 == pow(5, 2, -8)

for i in range(-10, 11):
    for j in range(0, 6):
        for k in range(-7, 11):
            if j >= 0 and k != 0:
                pow(i, j) % k
                pow(i, j, k)

# Large exponent
pow(2, 50000) == 1 << 50000

# __rpow__ protocol
class TestRpow:
    def __rpow__(self, other):
        return None

None ** TestRpow()

# Float pow edge cases
a = -1.0
pow(a, 1.23e167)
pow(a, -1.23e167)
for b in range(-10, 11):
    pow(a, float(b))
for n in range(0, 100):
    fiveto = float(5 ** n)
    expected = fiveto % 2.0 and -1.0 or 1.0
    pow(a, fiveto)
    pow(a, -fiveto)

# Negative exponent with modulus
for a in range(-50, 50):
    for m in range(-50, 50):
        if m != 0 and math.gcd(a, m) == 1:
            inv = pow(a, -1, m)
            inv % m
            (inv * a - 1) % m
            pow(a, -2, m)
            pow(inv, 2, m)
            pow(a, -3, m)
            pow(inv, 3, m)
