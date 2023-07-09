package official

import (
	"bytes"
	"context"
	"encoding/json"
	"errors"
	"fmt"
	"icfpc2023/backend/internal/httputil"
	"net/http"
	"net/url"
)

type submissionJSON struct {
	ID          string          `json:"_id"`
	ProblemID   int             `json:"problem_id"`
	SubmittedAt string          `json:"submitted_at"`
	Score       json.RawMessage `json:"score"`
}

type Submission struct {
	ID          string
	ProblemID   int
	SubmittedAt string
	Done        bool
	Score       int64
	Error       string
}

var _ json.Marshaler = &Submission{}
var _ json.Unmarshaler = &Submission{}

func (s *Submission) MarshalJSON() ([]byte, error) {
	return nil, errors.New("marshaling Submission is not implemented")
}

func (s *Submission) UnmarshalJSON(b []byte) error {
	var j submissionJSON
	if err := json.Unmarshal(b, &j); err != nil {
		return err
	}

	s.ID = j.ID
	s.ProblemID = j.ProblemID
	s.SubmittedAt = j.SubmittedAt

	var processingJSON string
	if err := json.Unmarshal(j.Score, &processingJSON); err == nil && processingJSON == "Processing" {
		s.Done = false
		s.Score = 0
		s.Error = ""
		return nil
	}

	var failureJSON struct {
		Failure string `json:"Failure"`
	}
	if err := json.Unmarshal(j.Score, &failureJSON); err == nil && failureJSON.Failure != "" {
		s.Done = true
		s.Score = 0
		s.Error = failureJSON.Failure
		return nil
	}

	var successJSON struct {
		Success float64 `json:"Success"`
	}
	if err := json.Unmarshal(j.Score, &successJSON); err == nil {
		s.Done = true
		s.Score = int64(successJSON.Success)
		s.Error = ""
		return nil
	}

	return errors.New("corrupted score")
}

type Client struct {
	apiKey string
}

func NewClient(apiKey string) *Client {
	return &Client{
		apiKey: apiKey,
	}
}

func (c *Client) ListAllSubmissions(ctx context.Context) ([]*Submission, error) {
	const endpoint = "https://api.icfpcontest.com/submissions?offset=0&limit=1000000"

	var response struct {
		Success []*Submission `json:"Success"`
		Failure string        `json:"Failure"`
	}
	if err := httputil.GetJSONWithAuth(ctx, endpoint, c.apiKey, &response); err != nil {
		return nil, err
	}
	if response.Failure != "" {
		return nil, errors.New(response.Failure)
	}
	return response.Success, nil
}

func (c *Client) GetSubmissionSpec(ctx context.Context, id string) (string, error) {
	params := url.Values{}
	params.Set("submission_id", id)
	url := "https://api.icfpcontest.com/submission?" + params.Encode()

	var response struct {
		Success struct {
			Submission Submission `json:"submission"`
			Contents   string     `json:"contents"`
		} `json:"Success"`
		Failure string `json:"Failure"`
	}
	if err := httputil.GetJSONWithAuth(ctx, url, c.apiKey, &response); err != nil {
		return "", err
	}
	if response.Failure != "" {
		return "", errors.New(response.Failure)
	}
	return response.Success.Contents, nil
}

func (c *Client) Submit(ctx context.Context, spec string) error {
	var partial struct {
		ProblemID int `json:"problem_id"`
	}
	if err := json.Unmarshal([]byte(spec), &partial); err != nil {
		return err
	}

	problemID := partial.ProblemID
	if problemID == 0 {
		return errors.New("problem ID missing")
	}

	req, err := json.Marshal(struct {
		ProblemID int    `json:"problem_id"`
		Contents  string `json:"contents"`
	}{
		ProblemID: problemID,
		Contents:  spec,
	})
	if err != nil {
		return err
	}

	res, err := http.Post("https://api.icfpcontest.com/submission", "application/json", bytes.NewBuffer(req))
	if err != nil {
		return err
	}
	res.Body.Close()

	if res.StatusCode/100 != 2 {
		return fmt.Errorf("HTTP status %d", res.StatusCode)
	}

	return nil
}
