def f() -> int:
    total: int = 0
    rep: int = 0
    while rep < 10000000:
        n: int = 20
        a: int = 0
        b: int = 1
        i: int = 0
        while i < n:
            temp: int = b
            b = a + b
            a = temp
            i = i + 1
        total = total + a
        rep = rep + 1
    return total
