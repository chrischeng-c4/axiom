# #1697: bigint-arg recursion safety guard. Locks in today's correct
# behaviour for `Ty::Int` params that carry values exceeding INT48,
# so any future JIT-prologue Int-param unbox optimization (NOTES-NEXT
# §3 D2 plan for factorial_recursive / fib_recursive perf) cannot
# silently truncate a bigint arg into a wrapped 48-bit int.
#
# fact(21) and above overflow i64 (max ~9.2e18); fact(25) is well into
# bigint range. identity() threads a bigint value through 5 recursion
# frames — its `n: int` param is the would-be-unboxed slot.

def fact(n: int) -> int:
    if n <= 1:
        return 1
    return n * fact(n - 1)

def identity(n: int, depth: int) -> int:
    if depth == 0:
        return n
    return identity(n, depth - 1)

print(fact(20))
print(fact(21))
print(fact(25))

big = 1 << 60
print(identity(big, 5))
print(identity(-big, 5))
print(identity(0, 5))
