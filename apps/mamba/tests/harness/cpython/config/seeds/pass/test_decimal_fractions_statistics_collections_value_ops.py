# Atomic 295 pass conformance — decimal module (hasattr Decimal) +
# fractions module (hasattr Fraction + Fraction(1,2).numerator==1/.
# denominator==2) + statistics module (hasattr mean/median/mode/
# stdev/variance/pstdev/pvariance/median_low/median_high/median_
# grouped/multimode/quantiles/harmonic_mean/geometric_mean/fmean/
# StatisticsError/NormalDist + median([1,3,5])==3 + mode([1,1,2])
# ==1) + collections module (hasattr deque/Counter/defaultdict/
# OrderedDict/namedtuple/ChainMap/UserDict/UserList/UserString +
# Counter('hello')['l']==2 + Counter('hello')['o']==1 +
# Counter('hello')['h']==1).
# All asserts match between CPython 3.12 and mamba.
import decimal
import fractions
import statistics
import collections


_ledger: list[int] = []

# 1) decimal — hasattr core surface (conformant subset)
assert hasattr(decimal, "Decimal") == True; _ledger.append(1)

# 2) fractions — hasattr core surface (conformant subset)
assert hasattr(fractions, "Fraction") == True; _ledger.append(1)

# 3) fractions — value contracts (integer-returning attributes)
assert fractions.Fraction(1, 2).numerator == 1; _ledger.append(1)
assert fractions.Fraction(1, 2).denominator == 2; _ledger.append(1)
assert fractions.Fraction(3, 4).numerator == 3; _ledger.append(1)
assert fractions.Fraction(3, 4).denominator == 4; _ledger.append(1)

# 4) statistics — hasattr core surface
assert hasattr(statistics, "mean") == True; _ledger.append(1)
assert hasattr(statistics, "median") == True; _ledger.append(1)
assert hasattr(statistics, "mode") == True; _ledger.append(1)
assert hasattr(statistics, "stdev") == True; _ledger.append(1)
assert hasattr(statistics, "variance") == True; _ledger.append(1)
assert hasattr(statistics, "pstdev") == True; _ledger.append(1)
assert hasattr(statistics, "pvariance") == True; _ledger.append(1)
assert hasattr(statistics, "median_low") == True; _ledger.append(1)
assert hasattr(statistics, "median_high") == True; _ledger.append(1)
assert hasattr(statistics, "median_grouped") == True; _ledger.append(1)
assert hasattr(statistics, "multimode") == True; _ledger.append(1)
assert hasattr(statistics, "quantiles") == True; _ledger.append(1)
assert hasattr(statistics, "harmonic_mean") == True; _ledger.append(1)
assert hasattr(statistics, "geometric_mean") == True; _ledger.append(1)
assert hasattr(statistics, "fmean") == True; _ledger.append(1)
assert hasattr(statistics, "StatisticsError") == True; _ledger.append(1)
assert hasattr(statistics, "NormalDist") == True; _ledger.append(1)

# 5) statistics — integer-returning value contracts
assert statistics.median([1, 3, 5]) == 3; _ledger.append(1)
assert statistics.mode([1, 1, 2]) == 1; _ledger.append(1)
assert statistics.mode([3, 3, 3, 5]) == 3; _ledger.append(1)

# 6) collections — hasattr core surface
assert hasattr(collections, "deque") == True; _ledger.append(1)
assert hasattr(collections, "Counter") == True; _ledger.append(1)
assert hasattr(collections, "defaultdict") == True; _ledger.append(1)
assert hasattr(collections, "OrderedDict") == True; _ledger.append(1)
assert hasattr(collections, "namedtuple") == True; _ledger.append(1)
assert hasattr(collections, "ChainMap") == True; _ledger.append(1)
assert hasattr(collections, "UserDict") == True; _ledger.append(1)
assert hasattr(collections, "UserList") == True; _ledger.append(1)
assert hasattr(collections, "UserString") == True; _ledger.append(1)

# 7) collections — Counter element-frequency value contracts
assert collections.Counter("hello")["l"] == 2; _ledger.append(1)
assert collections.Counter("hello")["o"] == 1; _ledger.append(1)
assert collections.Counter("hello")["h"] == 1; _ledger.append(1)
assert collections.Counter("hello")["e"] == 1; _ledger.append(1)
assert collections.Counter("hello")["z"] == 0; _ledger.append(1)

print(f"MAMBA_ASSERTION_PASS: test_decimal_fractions_statistics_collections_value_ops {sum(_ledger)} asserts")
