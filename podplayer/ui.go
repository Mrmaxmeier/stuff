package main

import (
	"fmt"
	"os"

	"github.com/gdamore/tcell"
)

func pickEpisode(podcasts []Podcast) *Episode {
	tcell.SetEncodingFallback(tcell.EncodingFallbackASCII)
	s, e := tcell.NewScreen()
	if e != nil {
		fmt.Fprintf(os.Stderr, "%v\n", e)
		os.Exit(1)
	}
	if e = s.Init(); e != nil {
		fmt.Fprintf(os.Stderr, "%v\n", e)
		os.Exit(1)
	}

	s.SetStyle(tcell.StyleDefault.
		Foreground(tcell.ColorWhite).
		Background(tcell.ColorBlack))
	s.Clear()

	quit := make(chan struct{})
	keyEvents := make(chan tcell.EventKey)
	go func() {
		for {
			select {
			case <-quit:
				return
			default:
			}
			ev := s.PollEvent()
			switch ev := ev.(type) {
			case *tcell.EventKey:
				switch ev.Key() {
				case tcell.KeyEscape, tcell.KeyCtrlC:
					close(quit)
					return
				case tcell.KeyCtrlL:
					s.Sync()
				default:
					keyEvents <- *ev
				}
			case *tcell.EventResize:
				s.Sync()
			}
		}
	}()

	pp := PodcastPicker{
		Podcasts: podcasts,
		Active:   true,
	}
	pp.Render(s)

	ep := EpisodePicker{
		PodcastPicker: &pp,
	}
	ep.Render(s)

	for {
		select {
		case <-quit:
			s.Fini()
			return nil
		case ev := <-keyEvents:
			switch ev.Key() {
			case tcell.KeyUp:
				if pp.Active {
					pp.ActiveIndex--
				} else {
					ep.ActiveIndex--
				}
			case tcell.KeyDown:
				if pp.Active {
					pp.ActiveIndex++
				} else {
					ep.ActiveIndex++
				}
			case tcell.KeyEnter:
				if ep.Active {
					episode := ep.Episode()
					s.Fini()
					close(quit)
					return &episode
				}
			case tcell.KeyLeft:
				pp.Active = true
				ep.Active = false
			case tcell.KeyRight:
				pp.Active = false
				ep.Active = true
			}
		}
		s.Clear()
		pp.Render(s)
		ep.Render(s)
		s.Sync()
	}
}
