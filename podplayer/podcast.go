package main

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

type Episode struct {
	UUID           string      `json:"uuid"`
	URL            string      `json:"url"`
	WebsiteURL     interface{} `json:"website_url"`
	Title          string      `json:"title"`
	Description    string      `json:"description"`
	Dd             string      `json:"dd"`
	DurationInSecs int         `json:"duration_in_secs"`
	FileType       string      `json:"file_type"`
	PublishedAt    string      `json:"published_at"`
	SizeInBytes    int         `json:"size_in_bytes"`
}
