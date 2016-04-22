package main

import (
	"fmt"
	"time"
)

func parseDate(s string) (time.Time, error) {
	return time.Parse("2006-01-02 15:04:05", s)
}

type Podcast struct {
	UUID         string    `json:"uuid"`
	URL          string    `json:"url"`
	Title        string    `json:"title"`
	Description  string    `json:"description"`
	ThumbnailURL string    `json:"thumbnail_url"`
	Category     string    `json:"category"`
	MediaType    string    `json:"media_type"`
	Language     string    `json:"language"`
	Author       string    `json:"author"`
	Episodes     []Episode `json:"episodes"`

	TempInfo UserPodcastChange `json:"temporary_info"`
}

func (podcast *Podcast) mergeWith(other Podcast) {
	fmt.Printf("merging:\n%+v\nwith\n%+v\n", podcast, other)
	if podcast.UUID != other.UUID {
		panic("invalid merge")
	}
	podcast.URL = other.URL
	podcast.Title = other.Title
	podcast.ThumbnailURL = other.ThumbnailURL
	podcast.Category = other.Category
	podcast.MediaType = other.MediaType
	podcast.Language = other.Language
	podcast.Author = other.Author
	podcast.TempInfo = other.TempInfo

	episodeMap := make(map[string]Episode)
	for _, episode := range podcast.Episodes {
		episodeMap[episode.UUID] = episode
	}
	for _, episode := range other.Episodes {
		episodeMap[episode.UUID] = episode
	}
	var episodes []Episode
	for _, episode := range episodeMap {
		episodes = append(episodes, episode)
	}
	podcast.Episodes = episodes
}

type Episode struct {
	UUID            string `json:"uuid"`
	URL             string `json:"url"`
	WebsiteURL      string `json:"website_url"`
	Title           string `json:"title"`
	Description     string `json:"description"`
	FullDescription string `json:"dd"`
	DurationInSecs  int    `json:"duration_in_secs"`
	FileType        string `json:"file_type"`
	PublishedAt     string `json:"published_at"`
	SizeInBytes     int    `json:"size_in_bytes"`

	TempInfo UserEpisodeChange `json:"temporary_info"`

	podcast Podcast
}

type Episodes []Episode

func (s Episodes) Len() int {
	return len(s)
}
func (s Episodes) Swap(i, j int) {
	s[i], s[j] = s[j], s[i]
}
func (s Episodes) Less(i, j int) bool {
	ti, _ := parseDate(s[i].PublishedAt)
	tj, _ := parseDate(s[j].PublishedAt)
	return ti.After(tj)
}
