from pwn import *
import sys

with open(sys.argv[1], "rb") as f:
    data = f.read()

fake_header_len = 64-3
header = data[:fake_header_len]
data = data[fake_header_len:]

# fix header
data = b"\x1bLua" + data

header_len = 12
print(hexdump(data[:header_len]))


def unmask_string(data):
    l = u32(data[:4])
    s = data[4:][:l]
    if len(s):
        s = xor(s, s[-1])
    return data[:4] + s, data[4+l:]


def skip(data, l):
    return data[:l], data[l:]


def skipvec(data, elemsize):
    l = u32(data[:4])
    return data[:4 + l*elemsize], data[4+l*elemsize:]


def loadfunction(data):
    res, data = unmask_string(data)
    x, data = skip(data, 4*2 + 4*1)
    res += x

    # code
    codelen = u32(data[:4])
    res += data[:4]
    data = data[4:]
    code = data[:codelen*4]
    res += code
    data = data[codelen*4:]

    # constants
    constlen = u32(data[:4])
    res += data[:4]
    data = data[4:]

    print("consts", constlen)
    for i in range(constlen):
        kind = data[:1]
        data = data[1:]
        res += kind
        print(i, kind)
        if kind == b"\x00":
            pass  # nil
        elif kind == b"\x01":
            # boolean, not mangled?
            res += data[:1]
            data = data[1:]
        elif kind == b"\x03":
            # int, not mangled? bamboozled double?
            print("thisisanint")
            res += data[:8]
            print(data[:8])
            data = data[8:]
        elif kind == b"\x04":
            # string: xor'd with something but null-terminated
            x, data = unmask_string(data)
            print("string:", x[4:])
            res += x
        elif kind == b"\t":
            # not in the reference implementation. looks like an int?
            res += data[:4]
            data = data[4:]
        elif kind == b"@":
            # not in the reference implementation. looks nil or something?
            pass
        else:
            print(hexdump(data[:20]))
            assert False, "unknown const"

    funcs = u32(data[:4])
    res += data[:4]
    data = data[4:]
    for _ in range(funcs):
        print("recursing")
        print(hexdump(data[:32]))
        x, data = loadfunction(data)
        res += x

    # debug

    lineinfo, data = skipvec(data, 4)
    res += lineinfo

    locvars = u32(data[:4])
    res += data[:4]
    data = data[4:]

    for _ in range(locvars):
        x, data = unmask_string(data)
        res += x
        res += data[:8]
        data = data[8:]

    upvalues = u32(data[:4])
    res += data[:4]
    data = data[4:]

    for _ in range(upvalues):
        x, data = unmask_string(data)
        res += x

    return res, data


new_data = data[:header_len]
data = data[header_len:]

while data:
    new, rest = loadfunction(data)
    new_data += new
    data = rest
    assert data == b""

with open(sys.argv[1] + ".out", "wb") as f:
    f.write(new_data)
