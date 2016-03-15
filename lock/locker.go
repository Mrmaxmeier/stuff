package main

import (
	"bufio"
	"errors"
	"fmt"
	"log"
	"os"
	"sync"
	"time"

	"github.com/BurntSushi/xgb"
	"github.com/BurntSushi/xgb/xproto"

	"github.com/msteinert/pam"
)

type Locker interface {
	Init()
	Lock()
}

type CustomLocker struct {
	retries      int
	lockedSince  time.Time
	unlockSignal sync.Cond
	X            *xgb.Conn
	window       xproto.Window
}

func (l *CustomLocker) openFullscreenWindow() {
	var err error
	l.window, err = xproto.NewWindowId(l.X)
	if err != nil {
		panic(err)
	}
	screen := xproto.Setup(l.X).DefaultScreen(l.X)
	xproto.CreateWindow(l.X, screen.RootDepth, l.window, screen.Root,
		0, 0, screen.WidthInPixels, screen.HeightInPixels, 0,
		xproto.WindowClassInputOutput, screen.RootVisual,
		xproto.CwBackPixel|xproto.CwEventMask,
		[]uint32{
			0xffffffff,
			xproto.EventMaskExposure |
				xproto.EventMaskKeyPress |
				xproto.EventMaskKeyRelease |
				xproto.EventMaskVisibilityChange |
				xproto.EventMaskStructureNotify,
		},
	)

	xproto.MapWindow(l.X, l.window)
}

func (l *CustomLocker) grabPointer() {
	for tries := 0; tries < 10000; tries++ {
		// TODO: magic values
		reply, err := xproto.GrabPointer(l.X, false, l.window, 0x0, 0x0, 0x0, l.window, xproto.CursorNone, xproto.TimeCurrentTime).Reply()
		fmt.Println("grabbing pointer...", reply.Status, err)
		if reply.Status == 0 {
			fmt.Println("pointer grabbed")
			return
		}
		time.Sleep(time.Microsecond * 50)
	}
	fmt.Println("failed to grab pointer")
}

func (l *CustomLocker) releaseInput() {
	for tries := 0; tries < 10000; tries++ {
		err := xproto.UngrabPointerChecked(l.X, xproto.TimeCurrentTime).Check()
		if err == nil {
			break
		}
		fmt.Println(err)
		time.Sleep(time.Microsecond * 50)
	}
}

func (l CustomLocker) Lock() {
	l.openFullscreenWindow()
	l.grabPointer()
	go func() {
		for {
			ev, xerr := l.X.WaitForEvent()
			if ev == nil && xerr == nil {
				fmt.Println("Both event and error are nil. Exiting...")
				return
			}

			if ev != nil {
				fmt.Printf("Event: %s\n", ev)
			}
			if xerr != nil {
				fmt.Printf("Error: %s\n", xerr)
			}
		}
	}()
	time.Sleep(time.Second * 10)
	l.releaseInput()
}

func (l *CustomLocker) tryAuth(password string) {
	t, err := pam.StartFunc("", "", func(s pam.Style, msg string) (string, error) {
		switch s {
		case pam.PromptEchoOff:
			fmt.Println(msg)
			return password, nil
		case pam.PromptEchoOn:
			fmt.Print(msg + " ")
			input, err := bufio.NewReader(os.Stdin).ReadString('\n')
			if err != nil {
				return "", err
			}
			return input[:len(input)-1], nil
		case pam.ErrorMsg:
			log.Print(msg)
			return "", nil
		case pam.TextInfo:
			fmt.Println(msg)
			return "", nil
		}
		return "", errors.New("Unrecognized message style")
	})
	if err != nil {
		log.Fatalf("Start: %s", err.Error())
	}
	err = t.Authenticate(0)
	if err != nil {
		log.Fatalf("Authenticate: %s", err.Error())
	}
	fmt.Println("Authentication succeeded!")
}

var customLocker CustomLocker

func init() {
	X, err := xgb.NewConn()
	if err != nil {
		fmt.Println(err)
		panic(err)
	}
	customLocker.X = X
}
