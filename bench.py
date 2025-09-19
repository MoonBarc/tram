i = 0
for r in range(100000):
    i += 1
    i *= (i % 5) + 1

print(i)