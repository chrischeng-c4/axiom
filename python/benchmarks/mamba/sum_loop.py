def f() -> int:
    total: int = 0
    i: int = 0
    while i < 10000000:
        total = total + i
        i = i + 1
    return total
