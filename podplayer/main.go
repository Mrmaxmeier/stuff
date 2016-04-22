package main

import (
	"fmt"

	"github.com/bgentry/speakeasy"
)

func main() {
	client, isNew, err := LoadClientFromDisk()

	if err != nil {
		panic(err)
	}

	if isNew {
		fmt.Println("could'nt find data.json; creating new profile")
	}

	if client.Email == "" {
		client.Email, err = speakeasy.Ask("email> ")
		if err != nil {
			panic(err)
		}
	}

	if client.Password == "" {
		client.Password, err = speakeasy.Ask("pass> ")
		if err != nil {
			panic(err)
		}
	}

	if e := client.Login(); e != nil {
		panic(e)
	}
	if e := client.SaveToDisk(); e != nil {
		panic(e)
	}

	if e := client.Sync(); e != nil {
		panic(e)
	}
	if e := client.SaveToDisk(); e != nil {
		panic(e)
	}

	podList := make([]Podcast, 0, len(client.Podcasts))

	for _, podcast := range client.Podcasts {
		podList = append(podList, podcast)
	}

	episode := pickEpisode(podList)
	if episode == nil {
		return
	}

	fmt.Println(episode.Title)
	mpv := NewMPV()
	mpv.Launch(episode.URL)

	if episode.TempInfo.PlayingStatus == InProgress {
		fmt.Println("seeking to", episode.TempInfo.PlayedUpTo)
		mpv.Seek(uint(episode.TempInfo.PlayedUpTo))
	}

	playingStatus := make(chan float64)
	quitChan := make(chan interface{})

	go client.ReportPlayingStatus(episode, playingStatus, quitChan)

	go mpv.ReportPlayingStatus(func(playing bool, position float64) {
		fmt.Println("statusCB", playing, position)
		playingStatus <- position
	})
	mpv.Wait()
	close(quitChan)
}
