# /// script
# requires-python = ">=3.12"
# dependencies = []
# ///
# center, ljust, rjust, zfill, count — padding/count methods

# center
print("hi".center(10))
print("hi".center(10, "*"))
print("hi".center(3))
print("h".center(4))
print("hi".center(5))
print("hi".center(1))
print("even".center(10, "-"))

# ljust / rjust
print("hi".ljust(5))
print("hi".ljust(5, "."))
print("hi".rjust(5))
print("hi".rjust(5, "0"))

# zfill
print("42".zfill(5))
print("-42".zfill(5))
print("+42".zfill(5))
print("abc".zfill(5))
print("abc".zfill(2))

# count — basic
print("abracadabra".count("a"))
print("abracadabra".count("ab"))
print("aaa".count("aa"))
print("xyz".count("a"))

# count with start/end
print("abracadabra".count("a", 3))
print("abracadabra".count("a", 0, 5))
print("abracadabra".count("a", 5, 8))

# count empty string edge
print("abc".count(""))
