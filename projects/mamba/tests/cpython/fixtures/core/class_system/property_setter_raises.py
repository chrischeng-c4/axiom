# @property + @setter with validation that raises

class Temperature:
    def __init__(self, celsius):
        self._c = celsius

    @property
    def celsius(self):
        return self._c

    @celsius.setter
    def celsius(self, value):
        if value < -273.15:
            raise ValueError("below absolute zero")
        self._c = value

    @property
    def fahrenheit(self):
        return self._c * 9/5 + 32

t = Temperature(25)
print(t.celsius)
print(t.fahrenheit)

t.celsius = 100
print(t.celsius)
print(t.fahrenheit)

try:
    t.celsius = -300
except ValueError as e:
    print("caught:", e)
