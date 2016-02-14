package main

import (
	"fmt"
	"io"
	"strings"

	"github.com/fatih/color"
)

type Test struct {
	name   string
	passed bool
	failed bool
	output string
	err    string
}

func (t *Test) print(w io.Writer) {
	if t.output != "" {
		fmt.Fprint(w, color.YellowString(indentString(t.output)))
	}
	if t.passed {
		fmt.Fprintln(w, color.GreenString("\t✔"), t.name)
	} else if t.failed {
		fmt.Fprintln(w, color.RedString("\t✖"), t.name)
		fmt.Fprintln(w, color.RedString("\t"+strings.Repeat("-", len(t.name)+2)))
		fmt.Fprintln(w, color.MagentaString(indentString(t.err)))
	} else {
		fmt.Fprintln(w, color.MagentaString("\t."), t.name)
	}
}
