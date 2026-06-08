# tier: required
# category: numeric
# inclusion_reason: MVP `range()` iteration baseline (10M-iter range-driven sum)

total: int = 0
for i in range(10000000):
    total = total + i
print(total)
