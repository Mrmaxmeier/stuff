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

const (
	reqTime int = iota + 1
)

// MPV launches and controls mpv sessions.
type MPV struct {
	media      string
	socketPath string
	sendChan   chan interface{}
	doneCh     chan interface{}
	eventChan  map[string]chan []byte
	reqChan    map[int]chan []byte
	ipcDone    sync.WaitGroup
}

// Launch starts mpv given an url to play.
func (mpv *MPV) Launch(media string) {
	fmt.Println("streaming", media)
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
	byt := []byte(str)
	fmt.Println("parse", str)
	var typ struct {
		Event     string `json:"event"`
		RequestID int    `json:"request_id"`
	}
	if err := json.Unmarshal(byt, &typ); err != nil {
		panic(err)
	}
	if typ.Event != "" {
		fmt.Println("event", typ.Event)
		select {
		case mpv.eventChan[typ.Event] <- byt:
		default:
			fmt.Println("event chan full")
		}
	} else if typ.RequestID > 0 {
		fmt.Println("reqid", typ.RequestID)
		select {
		case mpv.reqChan[typ.RequestID] <- byt:
		default:
			fmt.Println("req chan full")
		}
	} else {
		fmt.Println(str)
		panic("invalid message")
	}
}

func (mpv *MPV) subscribeEvent(typ string) chan []byte {
	evChan := make(chan []byte, 5)
	mpv.eventChan[typ] = evChan
	return evChan
}

func (mpv *MPV) subscribeData(typ int) chan []byte {
	c := make(chan []byte, 5)
	mpv.reqChan[typ] = c
	return c
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
	var playing = true
	pauseChan := mpv.subscribeEvent("pause")
	unpauseChan := mpv.subscribeEvent("unpause")
	playbackTimeChan := mpv.subscribeData(reqTime)

	for {
		select {
		case <-pauseChan:
			playing = false
			cb(false, position)
		case <-unpauseChan:
			playing = true
		case dat := <-playbackTimeChan:
			var d struct {
				Seconds float64 `json:"data"`
				Error   string  `json:"error"`
			}
			if err := json.Unmarshal(dat, &d); err != nil {
				panic(err)
			}
			position = d.Seconds
		case <-time.After(time.Second * 5):
			mpv.sendChan <- struct {
				Command   []interface{} `json:"command"`
				RequestID int           `json:"request_id"`
			}{
				[]interface{}{"get_property", "playback-time"},
				reqTime,
			}
		}
		cb(playing, position)
	}
}

func NewMPV() MPV {
	return MPV{
		sendChan:  make(chan interface{}),
		doneCh:    make(chan interface{}),
		eventChan: make(map[string]chan []byte),
		reqChan:   make(map[int]chan []byte),
	}
}
