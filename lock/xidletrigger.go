package main

import (
	"fmt"
	"time"

	"github.com/BurntSushi/xgb"
	"github.com/BurntSushi/xgb/screensaver"
	"github.com/BurntSushi/xgb/xproto"
	"github.com/BurntSushi/xgbutil"
	"github.com/gosuri/uilive"
)

type XIdleTrigger struct {
	locker      Locker
	lock        time.Duration
	suspend     time.Duration
	latestEvent time.Time
	rootWindow  xproto.Drawable
	X           *xgb.Conn
}

func (s *XIdleTrigger) Init() error {
	if s.X != nil {
		return nil
	}
	s.latestEvent = time.Now()

	X, err := xgb.NewConn()
	if err != nil {
		return err
	}
	s.X = X
	err = screensaver.Init(X)
	if err != nil {
		return err
	}
	xproto.Setup(X)
	xgbutilconn, err := xgbutil.NewConn()
	if err != nil {
		return err
	}
	s.rootWindow = xproto.Drawable(xgbutilconn.RootWin())
	return nil
}

func (s *XIdleTrigger) timeSinceUserInput() (time.Duration, error) {
	queryInfo, err := screensaver.QueryInfo(s.X, s.rootWindow).Reply()
	if err != nil {
		return 0, err
	}
	return time.Millisecond * time.Duration(queryInfo.MsSinceUserInput), nil
}

func (s *XIdleTrigger) Run(config *Config) {
	s.Init()

	if config.Progress {
		go func() {
			writer := uilive.New()
			writer.Start()
			defer writer.Stop()

			ticker := time.NewTicker(time.Millisecond * 100)
			for range ticker.C {
				queryInfo, _ := screensaver.QueryInfo(s.X, s.rootWindow).Reply()
				sinceUserInput := time.Millisecond * time.Duration(queryInfo.MsSinceUserInput)
				untilServer := time.Millisecond * time.Duration(queryInfo.MsUntilServer)
				//fmt.Println("raw", queryInfo.MsSinceUserInput, "\t", sinceUserInput)
				//fmt.Println("idle", sinceUserInput, "\tlock in", s.lock-sinceUserInput, "\tuntil server", untilServer)
				fmt.Fprintf(writer, "idle %s\tlock in %s\tsuspend in %s\tuntil server %s\n", sinceUserInput, s.lock-sinceUserInput, s.suspend-sinceUserInput, untilServer)
			}
		}()
	}

	ticker := time.NewTicker(time.Second)
	for range ticker.C {
		sinceUserInput, err := s.timeSinceUserInput()
		if err != nil {
			panic(err)
		}
		if sinceUserInput > s.lock {
			s.locker.Lock()
		}
		if sinceUserInput > s.suspend {
			Suspend()
		}
	}
}
