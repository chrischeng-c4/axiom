# Benchmark: Fannkuch-redux (n=7).
# Measures: list manipulation, in-place reversal, integer arithmetic.

def fannkuch(n: int) -> int:
    perm: list = list(range(n))
    perm1: list = list(range(n))
    count: list = [0] * n
    max_flips: int = 0
    checksum: int = 0
    r: int = n
    sign: int = 1

    while True:
        while r != 1:
            count[r - 1] = r
            r -= 1

        flips: int = 0
        if perm1[0] != 0:
            perm = list(perm1)
            k: int = perm[0]
            while k != 0:
                # Reverse perm[0..k+1]
                i: int = 0
                j: int = k
                while i < j:
                    tmp: int = perm[i]
                    perm[i] = perm[j]
                    perm[j] = tmp
                    i += 1
                    j -= 1
                k = perm[0]
                flips += 1
            if flips > max_flips:
                max_flips = flips
        checksum += sign * flips

        if sign == 1:
            perm1[0], perm1[1] = perm1[1], perm1[0]
            sign = -1
        else:
            perm1[1], perm1[2] = perm1[2], perm1[1]
            sign = 1
            while r < n:
                if count[r] > 1:
                    count[r] -= 1
                    break
                r += 1
                if r == n:
                    return max_flips
            count[r] = r + 1

    return max_flips


result: int = fannkuch(7)
print(result)
