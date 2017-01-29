#!/usr/bin/python3

import click
TAB_SIZE = 8
MAGIC_NUMBER = 140

RESTYLE_CHARS = "{;}"

def restyle(line):
    #print(line)
    if not any([line.endswith(c) for c in RESTYLE_CHARS]):
        return line
    sizes = {"\t": TAB_SIZE}
    visible_line_width = sum([sizes.get(c, 1) for c in line])
    line_width = len(line)
    move_width = 0
    for c in line[::-1]:
        if c in RESTYLE_CHARS:
            move_width += 1
        else:
            break
    filler = " " * (MAGIC_NUMBER - (visible_line_width - move_width))
    # print(line_width, move_width, filler)
    return line[:line_width - move_width] + filler + line[-move_width:]

def reflow(line):
    before = ""
    if any([line.startswith(c) for c in RESTYLE_CHARS]):
        before += line[0]
        line = line[1:]
    if len(before) > 0:
        return before + "\n" + line
    return line

def restyle_lines(lines, move_lines=True):
    data = ""
    for line in lines:
        while line.endswith("\n") or line.endswith("\t") or line.endswith(" "):
            line = line[:-1]
        line = reflow(restyle(line))
        data += line + "\n"

    if not move_lines:
        return data.split("\n")

    l = []
    for line in data.split("\n"):
        chars = [" ", "\t"] + list(RESTYLE_CHARS)
        if len(line) == sum([line.count(c) for c in chars]) and any([c in line for c in RESTYLE_CHARS]):
            l[-1] += line.replace(" ", "").replace("\t", "")
        else:
            l.append(line)
    l = restyle_lines(l, move_lines=False)
    return "\n".join(l)

@click.command()
@click.argument('input', type=click.File('r'))
@click.option('--width', default=MAGIC_NUMBER)
@click.option('--tabsize', default=TAB_SIZE)
def main(input, width, tabsize):
    global TAB_SIZE, MAGIC_NUMBER
    TAB_SIZE = tabsize
    MAGIC_NUMBER = width
    print(restyle_lines(input.readlines()))

if __name__ == '__main__':
    main()
