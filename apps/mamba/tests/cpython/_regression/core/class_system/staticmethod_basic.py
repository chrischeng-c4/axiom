# @staticmethod: no self/cls, call on class and instance

class MathUtils:
    @staticmethod
    def add(a, b):
        return a + b

    @staticmethod
    def is_even(n):
        return n % 2 == 0

# Call on class
print(MathUtils.add(3, 4))
print(MathUtils.is_even(10))
print(MathUtils.is_even(7))

# Call on instance
m = MathUtils()
print(m.add(10, 20))
print(m.is_even(4))
