import sys
import argparse

from bitstring import BitArray, BitStream

def multiply(input, n):
    bits = BitArray(input)
    final = BitStream([])
    for i in bits:
        final.insert([i]*n)
    return final.tobytes()

def multiply_decode(input, n):
    bits = BitArray(input)
    final = BitStream([])
    for i in range(0,len(bits),n):
        ones = 0
        for j in range(0,n):
            ones += bits[i+j]
        if ones > n//2:
            final.insert([1])
        else:
            final.insert([0])
    return final.tobytes()

def main(function):
    data = sys.stdin.buffer.read()
    data = function(data)
    sys.stdout.buffer.write(data)

if __name__ == "__main__":
    parser = argparse.ArgumentParser()
    parser.add_argument('-d', '--decode', action='store_true')
    parser.add_argument('-e', '--encode', action='store_true')
    parser.add_argument('-m', '--multiply', default=3)
    args = parser.parse_args()
    
    if args.decode and args.encode:
        raise ValueError('You can\'t both encode and decode the input')
    elif args.decode:
        function = lambda x: multiply_decode(x, int(args.multiply))
    else:
        function = lambda x: multiply(x, int(args.multiply))
    
    main(function)
