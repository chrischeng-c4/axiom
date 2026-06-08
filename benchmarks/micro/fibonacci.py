# Benchmark: fibonacci(35) — classic recursive Fibonacci.
# Measures: recursive function call overhead, integer arithmetic.

def fib(n: int) -> int:
    if n <= 1:
        return n
    return fib(n - 1) + fib(n - 2)

result: int = fib(35)
print(result)
