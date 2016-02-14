package main

import (
	"fmt"
	"io"
	"strings"
	"time"
)

type Benchmark struct {
	name  string
	opdur time.Duration
	ops   int64
}

func (b *Benchmark) print(w io.Writer, padding int) {
	padding -= len(b.name)
	padding += 4
	if padding < 2 {
		padding = 2
	}
	fmt.Fprintf(w, "\t%s:%s", b.name, strings.Repeat(" ", padding))
	fmt.Fprintf(w, "%s/op\n", b.opdur)
}
