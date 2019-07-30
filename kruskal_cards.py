import random
from collections import Counter
from math import ceil

CARD_TYPES = ['🂡', '🂱', '🃁', '🃑']
DECK = [chr(ord(c) + i + (1 if i > 10 else 0)) for i in range(13) for c in CARD_TYPES]
# DECK *= 2
ROW_SIZE = 9

def card_value(c):
    if ord(c) % 16 > 10:
        return 5
    return ord(c) % 16

def print_deck(deck, end=None):
    for y in range(ceil(len(deck) / ROW_SIZE)):
        print(" ".join(deck[ROW_SIZE*y:ROW_SIZE*(y+1)]))
        if end and ROW_SIZE*y <= end < ROW_SIZE * (y+1):
            print("  " * (end - ROW_SIZE * y) + "^")

instance = DECK[:]
random.shuffle(instance)
endcount = Counter()
for ci, _ in enumerate(instance[:ROW_SIZE]):
    while ci + card_value(instance[ci]) < len(instance):
        ci += card_value(instance[ci])
    endcount[ci] += 1
# print(endcount)
(selected_end, hits) = endcount.most_common()[0]
print_deck(instance, end=selected_end)
