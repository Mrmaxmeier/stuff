package main

import (
	"os/exec"
	"time"
)

var suspendChan chan interface{}

func Suspend() {
	select {
	case suspendChan <- nil:
	case <-time.NewTimer(time.Millisecond * 250).C:
	}
}

func suspendRoutine() {
	for {
		blockAndDrain(suspendChan)
		notify("suspending")
		preExec := time.Now()
		err := exec.Command("systemctl", "suspend").Run()
		if err != nil {
			notify(err.Error())
		}
		time.Sleep(time.Second * 5) // TODO: magic resume detection
		notify("suspended for " + time.Since(preExec).String())
		time.Sleep(time.Second)
	}
}

func init() {
	suspendChan = make(chan interface{}, 2)
}
