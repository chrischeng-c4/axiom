def f() -> int:
    limit: int = 1000000
    count: int = 0
    n: int = 2
    while n < limit:
        is_p: int = 1
        d: int = 2
        while d * d <= n:
            if n % d == 0:
                is_p = 0
                d = n
            d = d + 1
        count = count + is_p
        n = n + 1
    return count
