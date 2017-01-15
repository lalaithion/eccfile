import sys
import random

from bitstring import BitArray

def lossy(data):
    bits = BitArray(data)
    for i in range(len(bits)):
        if random.randrange(1000) == 0:
            bits[i] = not bits[i]
    return bits.tobytes()

data = sys.stdin.buffer.read()
data = lossy(data)
sys.stdout.buffer.write(data)
