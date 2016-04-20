package main

import (
	"encoding/json"
	"fmt"
	"io"
	"net"
	"os"
	"os/exec"
	"path/filepath"
	"strconv"
	"strings"
	"sync"
	"time"
)

// MPV launches and controls mpv sessions.
type MPV struct {
	media      string
	socketPath string
	sendChan   chan interface{}
	doneCh     chan interface{}
	eventChan  map[string]chan interface{}
	ipcDone    sync.WaitGroup
}

// Launch starts mpv given an url to play.
func (mpv *MPV) Launch(media string) {
	fmt.Println("streaming", media)
	mpv.doneCh = make(chan interface{})
	mpv.media = media
	suffix := int(time.Now().UnixNano()) + os.Getpid()
	mpv.socketPath = filepath.Join("/tmp", strconv.Itoa(suffix))
	fmt.Println("socketPath:", mpv.socketPath)
	mpv.launchProcess()
	fmt.Println("launched process")
	mpv.connectToIPC()
}

// Wait waits for the mpv process to exit.
func (mpv *MPV) Wait() {
	mpv.ipcDone.Wait()
}

func (mpv *MPV) launchProcess() {
	cmd := exec.Command("mpv", mpv.media, "--input-unix-socket="+mpv.socketPath, "--force-window", "--volume=30")
	if false {
		cmd.Stdout = os.Stdout
		cmd.Stderr = os.Stderr
	}
	err := cmd.Start()
	if err != nil {
		panic(err)
	}
	go func() {
		err := cmd.Wait()
		if err != nil {
			panic(err)
		}
		close(mpv.doneCh)
	}()
}

func (mpv *MPV) connectToIPC() {
	for {
		time.Sleep(time.Millisecond * 150)
		fmt.Println("stat", mpv.socketPath)
		stat, err := os.Stat(mpv.socketPath)
		if err == nil {
			fmt.Println("success!", stat.Mode())
			break
		}
	}
	conn, err := net.Dial("unix", mpv.socketPath)
	if err != nil {
		panic(err)
	}

	mpv.ipcDone.Add(2)

	go func() {
		data := make([]byte, 1024)
		var n int
		for {
			n, err = conn.Read(data)
			fmt.Println("read", n, "bytes")
			if err == io.EOF || n == 0 {
				break
			} else if err != nil {
				panic(err)
			}
			for _, seperated := range strings.Split(string(data[:n-1]), "\n") {
				mpv.parseMessage(seperated)
			}
		}
		mpv.ipcDone.Done()
		mpv.ipcDone.Wait()
		_ = conn.Close()
	}()

	mpv.sendChan = make(chan interface{})

	go func() {
		var data interface{}
		for {
			select {
			case data = <-mpv.sendChan:
			case <-mpv.doneCh:
				mpv.ipcDone.Done()
				return
			}
			buf, err := json.Marshal(data)
			if err != nil {
				panic(err)
			}
			fmt.Println("sending", string(buf))
			buf = append(buf, '\n')
			n, err := conn.Write(buf)
			fmt.Println("sent", n, "bytes")
			if err != nil {
				panic(err)
			}
		}
	}()
}

func (mpv *MPV) parseMessage(str string) {
	fmt.Println("parse", str)
	var typ struct {
		Event string `json:"event"`
	}
	if err := json.Unmarshal([]byte(str), &typ); err != nil {
		panic(err)
	}
	fmt.Println(typ.Event)
}

func (mpv *MPV) observeValues() {
	fmt.Println("observing values")
	mpv.sendChan <- struct {
		Command []interface{} `json:"command"`
	}{
		[]interface{}{"observe_property", 1, "playback-time"},
	}
}

func (mpv *MPV) subscribeEvent(typ string) chan interface{} {
	evChan := make(chan interface{})
	mpv.eventChan[typ] = evChan
	return evChan
}

// Seek seeks to given absolute position.
func (mpv *MPV) Seek(seconds uint) {
	mpv.sendChan <- struct {
		Command []interface{} `json:"command"`
	}{
		[]interface{}{"seek", seconds, "absolute"},
	}
}

func (mpv *MPV) ReportPlayingStatus(cb func(playing bool, position float64)) {
	var position float64
	pauseChan := mpv.subscribeEvent("pause")
	unpauseChan := mpv.subscribeEvent("unpause")
	select {
	case <-pauseChan:
		cb(false, position)
	case <-unpauseChan:
		cb(true, position)
	case <-time.After(time.Second * 5):
		mpv.sendChan <- struct {
			Command []interface{} `json:"command"`
		}{
			[]interface{}{"seek", seconds, "absolute"},
		}
	}
}
