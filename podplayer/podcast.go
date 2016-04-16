package main

import "fmt"

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
}
