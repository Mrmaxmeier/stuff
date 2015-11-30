package main

import "fmt"

import "math"
import "github.com/nsf/termbox-go"
import "time"

func abs(a int) int {
	if a < 0 {
		return -a
	}
	return a
}

func lerp(from, to float64, step float64) float64 {
	return from + step*(to-from)
}

type Field struct {
	pos   Position
	state State
}

type Position struct {
	q, r, s int
}

type Ant struct {
	position Position
	rotation int
}

type State struct {
	color    termbox.Attribute
	rotation int
	next     *State
	char     rune
}

func (a *Ant) turn(i int) {
	a.rotation = a.rotation + i
	a.rotation = (6 + (a.rotation % 6)) % 6
}

func (a *Ant) nextPos() Position {
	q := a.position.q
	r := a.position.r
	s := a.position.s
	switch a.rotation {
	case 0:
		return Position{q, r - 1, s + 1}
	case 1:
		return Position{q + 1, r - 1, s}
	case 2:
		return Position{q + 1, r, s - 1}
	case 3:
		return Position{q, r + 1, s - 1}
	case 4:
		return Position{q - 1, r + 1, s}
	case 5:
		return Position{q - 1, r, s + 1}
	}
	panic("invalid rotation")
}

func (a *Ant) fieldState() State {
	for _, f := range active {
		if a.position == f.pos {
			return f.state
		}
	}
	return initialState
}

func (a *Ant) flip() {
	for i := 0; i < len(active); i++ {
		if a.position == active[i].pos {
			active[i].state = *active[i].state.next
			return
		}
	}
	f := Field{a.position, *initialState.next}
	active = append(active, f)
}

func (a *Ant) tick() {
	a.turn(a.fieldState().rotation)
	a.flip()
	a.position = a.nextPos()
}

func lerpPos() {
	w, h := termbox.Size()
	prefX, prefY := 0, 0
	if ax, ay := ant.position.toPixel(); abs(ax) > w/2 || abs(ay) > h/2 {
		prefX = -ax
		prefY = -ay
	}
	xMod = lerp(xMod, float64(prefX), lerpSpeed)
	yMod = lerp(yMod, float64(prefY), lerpSpeed)
}

func (p *Position) toPixel() (int, int) {
	x := int(size * float64(3.) / float64(2.) * float64(p.q))
	y := int(size * math.Sqrt(3.) * float64(p.r+p.q/2))
	return x, y
}

func (f *Field) drawHex() {
	w, h := termbox.Size()
	x, y := f.pos.toPixel()

	x += w / 2
	y += h / 2

	x += int(xMod)
	y += int(yMod)

	color := f.state.color
	c := f.state.char
	size := 3
	for dx := -size; dx < size; dx++ {
		for dy := -size; dy < size; dy++ {
			if abs(dx)+abs(dy) < size {
				termbox.SetCell(x+dx, y+dy, c, termbox.ColorDefault, color)
			}
		}
	}
}

func draw() {
	tick++
	termbox.Clear(termbox.ColorDefault, termbox.ColorDefault)
	for _, f := range active {
		f.drawHex()
	}
	char := map[int]rune{0: '-', 1: '\\', 2: '/'}[tick%3]
	termbox.SetCell(0, 0, char, termbox.ColorDefault, termbox.ColorDefault)

	termbox.Sync()
}

var tick = 0

var colors = []termbox.Attribute{}
var size = float64(1)
var initialState State
var active = []Field{}
var ant Ant
var xMod = float64(0)
var yMod = float64(0)
var lerpSpeed = float64(0.01)
var speedMod = 10

func main() {
	g := 0
	r := 0
	for r = 0; r <= 5; r++ {
		colors = append(colors, termbox.Attribute(6+g*6+r*36))
	}
	r = 5
	for g = 0; g <= 5; g++ {
		colors = append(colors, termbox.Attribute(6+g*6+r*36))
	}
	fmt.Println(colors)

	if true {
		// Rule: L2,N,N,L1,L2,L1. (L2=2Pi/3, L1=Pi/3, N=+0 (no turn)).
		s6 := State{colors[5], 1, nil, '6'}
		s5 := State{colors[4], 2, &s6, '5'}
		s4 := State{colors[3], 1, &s5, '4'}
		s3 := State{colors[2], 0, &s4, '3'}
		s2 := State{colors[1], 0, &s3, '2'}
		s1 := State{termbox.ColorBlack, 2, &s2, '1'}
		s6.next = &s1
		initialState = s1
	} else {
		initialState = State{termbox.ColorBlack, 1, nil, '0'}
		s2 := State{termbox.ColorRed, -1, &initialState, '1'}
		initialState.next = &s2
	}

	ant = Ant{}

	err := termbox.Init()
	if err != nil {
		panic(err)
	}
	defer termbox.Close()

	termbox.SetOutputMode(termbox.Output216)

	eventQueue := make(chan termbox.Event)
	go func() {
		for {
			eventQueue <- termbox.PollEvent()
		}
	}()
	draw()

loop:
	for {
		select {
		case ev := <-eventQueue:
			if ev.Type == termbox.EventKey && (ev.Key == termbox.KeyEsc || ev.Key == termbox.KeyCtrlC) {
				break loop
			}
		default:
			draw()
			for i := 0; i < speedMod; i++ {
				ant.tick()
			}
			lerpPos()
			time.Sleep(75 * time.Millisecond)
		}
	}
}
