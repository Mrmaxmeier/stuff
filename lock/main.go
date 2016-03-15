package main

import (
	"fmt"
	"net"
	"os"
	"os/exec"
	"time"

	"github.com/bobziuchkovski/writ"
)

const sockfile = "/var/run/lock.sock"

type Config struct {
	HelpFlag     bool   `flag:"help" description:"Display this help message and exit"`
	Verbosity    int    `flag:"v, verbose" description:"Display verbose output"`
	Progress     bool   `flag:"progress" description:"Progress"`
	Lock         bool   `flag:"lock" description:"Lock"`
	LockAfter    string `option:"lock-after" default:"3m"`
	SuspendAfter string `option:"suspend-after" default:"5m"`
}

func main() {
	config := &Config{}
	cmd := writ.New("lock", config)
	cmd.Help.Usage = "Usage: lock [OPTION]..."
	cmd.Help.Header = "A crappy locker."
	_, _, err := cmd.Decode(os.Args[1:])
	if err != nil || config.HelpFlag {
		fmt.Println(`
                          / '.   .' "
                  .---.  <    > <    >  .---.
                  |    \  \ - ~ ~ - /  /    |
      _____          ..-~             ~-..-~
     |     |   \~~~\.'                    './~~~/
    ---------   \__/                        \__/
   .'  O    \     /              /        \  "
  (_____,    '._.'               |         }  \/~~~/
   '----.          /       }     |        /    \__/
         '-.      |       /      |       /      '. ,~~|
             ~-.__|      /_ - ~ ^|      /- _      '..-'
                  |     /        |     /     ~-.     '-. _  _  _
                  |_____|        |_____|         ~ - . _ _ _ _ _>
		`)
		cmd.ExitHelp(err)
	}

	if config.Lock {
		notify("locking...")
	} else {
		getDur := func(s string) time.Duration {
			time, err := time.ParseDuration(s)
			if err != nil {
				panic(err)
			}
			return time
		}
		locker := I3Lock{}
		locker.Init()
		idletrigger := XIdleTrigger{
			lock:    getDur(config.LockAfter),
			suspend: getDur(config.SuspendAfter),
			locker:  &locker,
		}
		go listen(&locker)
		idletrigger.Run(config)
	}
}

func listen(locker Locker) {
	l, err := net.Listen("unix", sockfile)
	if err != nil {
		fmt.Println("listen error", err)
		return
	}
	for {
		fd, err := l.Accept()
		if err != nil {
			fmt.Println("accept error", err)
			return
		}

		go func() {
			fmt.Println("locking due to unixsock", fd)
			locker.Lock()
		}()
	}
}

func notify(msg string) {
	fmt.Println(msg)
	exec.Command("notify-send", msg).Run()
}

func suspend() {
	err := exec.Command("systemctl", "suspend").Run()
	if err != nil {
		notify(err.Error())
	}
}
