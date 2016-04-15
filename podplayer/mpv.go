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
	"sync"
	"time"
)

// MPV launches and controls mpv sessions.
type MPV struct {
	media      string
	socketPath string
	sendChan   chan interface{}
	doneCh     chan interface{}
}

// Launch starts mpv given an url to play.
func (mpv *MPV) Launch(media string) {
	mpv.doneCh = make(chan interface{})
	mpv.media = media
	suffix := int(time.Now().UnixNano()) + os.Getpid()
	mpv.socketPath = filepath.Join("/tmp", strconv.Itoa(suffix))
	fmt.Println("socketPath:", mpv.socketPath)
	mpv.launchProcess()
	fmt.Println("launched process")
	var ipcDone sync.WaitGroup
	mpv.connectToIPC(&ipcDone)

	fmt.Println("waiting 5s")
	time.Sleep(time.Second * 5)
	//mpv.observeValues()
	fmt.Println("waiting on ipc sync.WaitGroup")
	ipcDone.Wait()
	fmt.Println("done")
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

func (mpv *MPV) connectToIPC(wg *sync.WaitGroup) {
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

	wg.Add(2)

	go func() {
		data := make([]byte, 1024)
		var n int
		for {
			n, err = conn.Read(data)
			fmt.Println("read", n, "bytes")
			if err == io.EOF || n == 0 {
				break
			}
			fmt.Println(string(data[:n-1]))
			if err != nil {
				panic(err)
			}
		}
		wg.Done()
		fmt.Println("readRoutine waiting for sendRoutine")
		wg.Wait()
		fmt.Println("readRoutine closing and exiting")
		_ = conn.Close()
	}()

	mpv.sendChan = make(chan interface{})

	go func() {
		var data interface{}
		for {
			select {
			case data = <-mpv.sendChan:
			case <-mpv.doneCh:
				fmt.Println("sendRoutine got doneCh signal")
				wg.Done()
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

func (mpv *MPV) observeValues() {
	fmt.Println("observing values")
	mpv.sendChan <- struct {
		Command []interface{} `json:"command"`
	}{
		[]interface{}{"observe_property", 1, "playback-time"},
	}
}
