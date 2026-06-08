# String edge cases: empty string, removeprefix/removesuffix, partition, splitlines, swapcase, slicing
print(''.upper())
print(''.split(','))
# removeprefix / removesuffix
print('TestCase'.removeprefix('Test'))
print('file.txt'.removesuffix('.txt'))
# partition / rpartition
print('a-b-c'.partition('-'))
print('a-b-c'.rpartition('-'))
# splitlines
print('a\nb\nc'.splitlines())
# swapcase
print('hElLo'.swapcase())
# String slicing
s = 'abcdef'
print(s[1:4])
print(s[::2])
# Reverse slicing (negative step)
print(s[::-1])
print(s[-2::-1])
print(s[4:1:-1])
print(s[::-2])
