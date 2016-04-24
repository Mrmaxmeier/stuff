package main

import (
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

func blockAndDrain(c chan interface{}) {
	<-c
	for {
		select {
		case <-c:
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
			blockAndDrain(s.lock)
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
	case <-time.NewTimer(time.Millisecond * 125).C:
	}
}
