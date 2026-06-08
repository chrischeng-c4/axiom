# tier: required
# category: numeric
# inclusion_reason: MVP int arithmetic baseline (10M-iter `int += int`)

total: int = 0
i: int = 0
while i < 10000000:
    total = total + i
    i = i + 1
print(total)
