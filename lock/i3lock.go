package main

import (
	"fmt"
	"os/exec"
	"time"
)

func signalAll(c chan interface{}) {
	for {
		select {
		case c <- nil:
		default:
			return
		}
	}
}

type I3Lock struct {
	lock   chan interface{}
	unlock chan interface{}
}

func (s *I3Lock) Init() {
	s.lock = make(chan interface{})
	s.unlock = make(chan interface{})
	go func() {
		for {
			<-s.lock
			s.run()
			signalAll(s.unlock)
		}
	}()
}

func (s *I3Lock) run() {
	preExec := time.Now()
	err := exec.Command("i3lock", "-n").Run()
	if err != nil {
		notify(err.Error())
		panic(err)
	}
	notify("locked for " + time.Since(preExec).String())
}

func (s *I3Lock) Lock() {
	select {
	case s.lock <- nil:
		fmt.Println("locked")
	case <-time.NewTimer(time.Millisecond * 500).C:
		notify("locker already active")
	}
	<-s.unlock
}
