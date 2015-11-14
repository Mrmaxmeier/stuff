#!/usr/bin/python3
import random
import time
import shutil
from collections import namedtuple
from itertools import product

EMPTY = 0
TREE = 1
HEATING = 2
BURNING = 3
STATES = {
	EMPTY: "EMPTY",
	TREE: "TREE",
	HEATING: "HEATING",
	BURNING: "BURNIGN"
}

TREE_CHANCE = .1
BURN_CHANCE = .25
DECAY_CHANCE = .5
FRAME_DELAY = 0.1

TREE_STR = '#' #"🎄"
EMPTY_STR = " "


GREEN_FMT = "\033[1;32m"
ORANGE_FMT = "\033[1;33m"
RED_FMT = "\033[1;31m"
RESET_FMT = "\033[0m"

nearby = list(product(*[[-1, 0, 1]] * 2))
nearby_3 = list(product(*[[-3, -2, -1, 0, 1, 2, 3]] * 2))
Size = namedtuple('size', ['x', 'y'])

def getTermSize():
	size = shutil.get_terminal_size((80, 20))
	return Size(x=size.columns // 2, y=size.lines-1)

def rnd(chance):
	return random.random() < chance

def step(state):
	will_heat = []
	def f(chk):
		def d(n):
			if size.x > x + n[0] >= 0 and size.y > y + n[1] >= 0:
				return state[y + n[1]][x + n[0]] == chk
		return d
	for y in range(size.y):
		for x in range(size.x):
			if state[y][x] == TREE:
				if any(map(f(BURNING), nearby)):
					will_heat.append((x, y))
	for x, y in will_heat:
		state[y][x] = HEATING
	no_fire = all([state[y][x] <= TREE for x, y in product(range(size.x), range(size.y))])
	state[size.y // 2][size.x // 2] = BURNING
	for y in range(size.y):
		for x in range(size.x):
			s = state[y][x]
			if s == EMPTY and rnd(TREE_CHANCE):
				if not any(map(f(BURNING), nearby_3)):
					s = TREE
			elif s == HEATING and rnd(BURN_CHANCE):
				s = BURNING
			elif s == BURNING and rnd(DECAY_CHANCE):
				s = EMPTY
			state[y][x] = s
	return state

def draw(state):
	current_reset = None
	for row in state:
		for s in row:
			rst, s = {
				EMPTY: (RESET_FMT, EMPTY_STR),
				TREE: (GREEN_FMT, TREE_STR),
				HEATING: (ORANGE_FMT, TREE_STR),
				BURNING: (RED_FMT, TREE_STR)
			}[s]
			if rst != current_reset:
				print(rst, end="")
				current_reset = rst
			print(s, end=" ")
		print()

size = getTermSize()
state = [[rnd(TREE_CHANCE) for x in range(size.x)] for y in range(size.y)]
try:
	while True:
		if size != getTermSize():
			size = getTermSize()
			state = [[rnd(TREE_CHANCE) for x in range(size.x)] for y in range(size.y)]
		state = step(state)
		draw(state)
		time.sleep(FRAME_DELAY)
except KeyboardInterrupt:
	print(RESET_FMT)
