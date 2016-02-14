package main

import (
	"bufio"
	"fmt"
	"io"
	"os"
	"os/exec"
	"regexp"
	"strconv"
	"strings"
	"sync"
	"time"

	"github.com/fatih/color"
	"github.com/gosuri/uilive"
)

type Tester struct {
	tests       []*Test
	benchmarks  []*Benchmark
	status1ok   bool
	outChan     chan string
	testspassed bool
	cmdFinished bool
	errors      string
}

func (t *Tester) processLines() {
	var currentTest *Test
	for line := range t.outChan {
		line = line[:len(line)-1]
		if currentTest != nil {
			if strings.HasPrefix(line, "--- PASS") {
				currentTest.passed = true
				// TODO: parse time
				currentTest = nil
				continue
			}
			if strings.HasPrefix(line, "--- FAIL") {
				currentTest.failed = true
				t.status1ok = true
				for line, ok := <-t.outChan; ok && strings.HasPrefix(line, "\t"); line, ok = <-t.outChan {
					currentTest.err += line[1:len(line)-1] + "\n"
				}
				currentTest = nil
			}
			if currentTest != nil {
				currentTest.output += line + "\n"
				continue
			}
		}
		if strings.HasPrefix(line, "=== RUN") {
			currentTest = &Test{}
			currentTest.name = line[6+4:]
			t.tests = append(t.tests, currentTest)
			continue
		}

		if line == "PASS" {
			t.testspassed = true
			break
		}

		t.errors += line + "\n"
	}
	benchmarkRegex, _ := regexp.Compile("Benchmark(\\w+)-8[ \t]+([0-9]+)[ \t]+([0-9.]+) ns/op")
	for line := range t.outChan {
		line = line[:len(line)-1]
		if strings.HasPrefix(line, "ok") {
			// TODO: parse path, time
			break
		}
		if !strings.HasPrefix(line, "Benchmark") {
			t.errors += line + "\n"
			break
		}
		regexMatches := benchmarkRegex.FindAllStringSubmatch(line, -1)
		if len(regexMatches) != 1 || len(regexMatches[0]) != 4 {
			t.errors += "invalid benchmark line:" + "\n"
			t.errors += line + "\n"
			continue
		}
		b := Benchmark{}
		t.benchmarks = append(t.benchmarks, &b)
		regexResults := regexMatches[0][1:]
		//fmt.Printf("regex: %v\n", regexResults)
		b.name = regexResults[0]
		ops, err := strconv.ParseInt(regexResults[1], 10, 64)
		if err != nil {
			panic(err)
		}
		b.ops = ops

		dur, err := time.ParseDuration(regexResults[2] + "ns")
		if err != nil {
			panic(err)
		}
		b.opdur = dur
		//b.print()
	}
}

func (t *Tester) render(w io.Writer) {
	category := color.New(color.Bold).SprintFunc()
	fmt.Fprintln(w, category("Tests"), len(t.tests))
	for _, test := range t.tests {
		test.print(w)
	}
	if t.testspassed {
		fmt.Fprintln(w, color.CyanString("Tests Passed!"))
	}
	if len(t.benchmarks) > 0 {
		fmt.Fprintln(w, category("Benchmarks"), len(t.benchmarks))
		padding := 5
		for _, benchmark := range t.benchmarks {
			if len(benchmark.name) > padding {
				padding = len(benchmark.name)
			}
		}
		for _, benchmark := range t.benchmarks {
			benchmark.print(w, padding)
		}
	}
	if t.errors != "" {
		fmt.Fprintln(w, color.RedString(t.errors))
	}
}

func (t *Tester) renderBlocking(wg *sync.WaitGroup) {
	writer := uilive.New()
	writer.RefreshInterval = time.Millisecond * 25
	//writer.Start()

	for !t.cmdFinished {
		t.render(writer)
		writer.Wait()
	}
	writer.Flush()
	wg.Done()
}

func indentString(s string) (n string) {
	lines := strings.Split(s, "\n")
	for _, line := range lines {
		n += "\t" + line + "\n"
	}
	return n[:len(n)-2]
}

func main() {
	args := os.Args[1:]
	args = append([]string{"test", "-v"}, args...)
	fmt.Println("$ go", args)
	cmd := exec.Command("go", args...)
	cmdOut, _ := cmd.StdoutPipe()
	cmdErr, _ := cmd.StderrPipe()
	cmd.Start()
	tester := &Tester{
		outChan: make(chan string, 99),
	}
	readLinesToChan := func(r io.Reader, c chan string, wg *sync.WaitGroup) {
		reader := bufio.NewReader(r)
		for {
			line, err := reader.ReadString('\n')
			//fmt.Print("line", line)
			if err != nil {
				if err.Error() != "EOF" {
					fmt.Println("read err", err)
				}
				wg.Done()
				return
			}
			c <- line
		}
	}
	waitGroup := &sync.WaitGroup{}
	waitGroup.Add(2)
	go readLinesToChan(cmdOut, tester.outChan, waitGroup)
	go readLinesToChan(cmdErr, tester.outChan, waitGroup)
	go tester.processLines()
	renderWG := &sync.WaitGroup{}
	renderWG.Add(1)
	go tester.renderBlocking(renderWG)
	waitGroup.Wait()
	err := cmd.Wait()
	tester.cmdFinished = true
	if err != nil {
		if err.Error() == "exit status 1" && tester.status1ok {
			return
		}
		panic(err)
	}
	renderWG.Wait()
}
