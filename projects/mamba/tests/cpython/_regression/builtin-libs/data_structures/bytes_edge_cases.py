# Bytes/bytearray edge cases: construction, length, bytearray mutation
b1 = b'hello'
print(len(b1))
# bytearray mutable ops
ba = bytearray(b'abc')
ba[0] = 65
print(ba)
ba = bytearray(b'abc')
ba.append(100)
ba.extend(b'ef')
print(ba)
# bytearray reverse
ba = bytearray(b'abc')
ba.reverse()
print(ba)
