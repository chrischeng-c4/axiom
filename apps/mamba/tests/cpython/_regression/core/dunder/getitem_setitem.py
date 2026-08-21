# Dunder conformance: __getitem__, __setitem__, __len__, __contains__.
class MyList:
    def __init__(self):
        self.data = []

    def __getitem__(self, i):
        return self.data[i]

    def __setitem__(self, i, v):
        self.data[i] = v

    def __len__(self):
        return len(self.data)

    def __contains__(self, item):
        return item in self.data

ml = MyList()
ml.data = [10, 20, 30]
print(ml[0])
print(ml[1])
print(ml[2])
print(len(ml))
print(10 in ml)
print(99 in ml)
ml[1] = 99
print(ml[1])
