from __future__ import annotations


def scale_decimal(value: int, divisor: int) -> str:
    if value < 0 or divisor < 0:
        raise ValueError("scale_decimal expects non-negative value and divisor")

    places = 0
    remaining = divisor
    while remaining > 1 and remaining % 10 == 0:
        remaining //= 10
        places += 1

    if divisor <= 1 or remaining != 1:
        return str(value // max(divisor, 1))

    return f"{value // divisor}.{value % divisor:0{places}d}"
