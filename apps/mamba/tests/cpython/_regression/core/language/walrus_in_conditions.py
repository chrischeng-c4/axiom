# Walrus operator in if/elif/while conditions

# Walrus in if condition
if (n := 10) > 5:
    print("n is", n)

# Walrus in elif chain
value = 42
if (x := value * 2) < 50:
    print("small", x)
elif (y := value + 10) > 40:
    print("mid", y)
else:
    print("big")

# Walrus in while loop condition
data = [3, 2, 1, 0]
while (val := data.pop()) > 0:
    print("got", val)
print("done", val)

# Walrus in comprehension filter
nums = [1, 2, 3, 4, 5]
squares = [sq for n in nums if (sq := n * n) > 4]
print(squares)
