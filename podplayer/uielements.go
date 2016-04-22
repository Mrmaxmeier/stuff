package main

import "github.com/gdamore/tcell"

func setString(screen tcell.Screen, x, y int, str string, style tcell.Style) {
	runes := []rune(str)
	for i := 0; i < len(runes); i++ {
		screen.SetCell(x+i, y, style, runes[i])
	}
}

func getStyle(active bool) tcell.Style {
	if active {
		return tcell.StyleDefault.Bold(true)
	}
	return tcell.StyleDefault
}

type EpisodePicker struct {
	Active        bool
	ActiveIndex   int
	PodcastPicker *PodcastPicker
}

func (lp *EpisodePicker) Render(screen tcell.Screen) {
	podcast := lp.PodcastPicker.Podcasts[lp.PodcastPicker.ActiveIndex]
	episodes := podcast.Episodes
	lp.ActiveIndex = (lp.ActiveIndex + len(episodes)) % len(episodes)
	width, _ := screen.Size()
	x := width / 2
	setString(screen, x, 0, "Episodes - "+podcast.Title, getStyle(lp.Active))
	for i, episode := range episodes {
		active := i == lp.ActiveIndex && lp.Active
		y := i + 1
		if active {
			screen.SetCell(x, y, getStyle(true), '>')
		} else {
			char := ' '
			switch episode.TempInfo.PlayingStatus {
			case InProgress:
				char = '.'
			case Finished:
				char = '-'
			}
			screen.SetCell(x, y, getStyle(false), char)
		}
		setString(screen, x+1, y, episode.Title, getStyle(active))
	}
}

func (lp *EpisodePicker) Episode() Episode {
	podcast := lp.PodcastPicker.Podcasts[lp.PodcastPicker.ActiveIndex]
	return podcast.Episodes[lp.ActiveIndex]
}

type PodcastPicker struct {
	Active      bool
	Podcasts    []Podcast
	ActiveIndex int
	IsRight     bool
}

func (lp *PodcastPicker) Render(screen tcell.Screen) {
	lp.ActiveIndex = (lp.ActiveIndex + len(lp.Podcasts)) % len(lp.Podcasts)
	setString(screen, 0, 0, "Podcasts", getStyle(lp.Active))
	for i, podcast := range lp.Podcasts {
		active := i == lp.ActiveIndex && lp.Active
		y := i + 1
		if active {
			screen.SetCell(0, y, getStyle(true), '>')
		} else {
			screen.SetCell(0, y, getStyle(false), ' ')
		}
		setString(screen, 1, y, podcast.Title, getStyle(active))
	}
}
