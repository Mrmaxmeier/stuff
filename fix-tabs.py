#!/bin/python3
from pathlib import Path
from pprint import pprint

import click

stats = dict(tabs=0, spaces=0, trailing_whitespace=0, lines_processed=0)
is_whitespace = lambda c: c in ' \t'

def first_non_whitespace(line, tab_size):
    offset = 0
    for i, c in enumerate(line):
        if c == ' ':
            offset += 1
        elif c == '\t':
            offset = (offset + tab_size) // tab_size * tab_size
        else:
            return (i, offset)

assert first_non_whitespace('  \t  hue', tab_size=4) == (5, 6)
assert '  \t  hue'[5:] == 'hue'
assert first_non_whitespace('  \t  ', tab_size=4) == None

def process(indent_type, tab_size, lines):
    global stats
    for line in lines:
        stats['lines_processed'] += 1
        if line != line.rstrip('\t '):
            stats['trailing_whitespace'] += 1
        line = line.rstrip('\t ')
        # only trailing whitespace:
        # yield line
        # continue
        fnw = first_non_whitespace(line, tab_size)
        if fnw is None:
            yield ''
            continue

        fnwi, fnwo = fnw
        line = line[fnwi:]
        if indent_type == 'spaces':
            line = ' ' * fnwo + line
        else:
            line = '\t' * (fnwo // tab_size) + ' ' * (fnwo % tab_size) + line
        yield line
    yield ''

def reindent_file(indent_type, tab_size, path):
    global stats

    print(path)
    with open(path, 'r') as f:
        data = f.read()

    stats['tabs'] += data.count("\t")
    stats['spaces'] += data.count(" ")

    lines = data.splitlines()
    processed = process(indent_type, tab_size, lines)
    with open(path, 'w') as f:
        f.writelines('\n'.join(processed))


@click.command()
@click.option('--indent-type', type=click.Choice(['tab', 'spaces']))
@click.option('--tab-size', default=8)
@click.argument('files', nargs=-1)
def reindent_files(indent_type, tab_size, files):
    """Fix various whitespace problems."""
    if indent_type is None:
        print("invalid indent-type")
        return
    for path in files:
        reindent_file(indent_type, tab_size, path)
    stats['files_processed'] = len(files)
    pprint(stats)

if __name__ == '__main__':
    reindent_files()
