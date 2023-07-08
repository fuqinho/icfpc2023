package httputil

import (
	"context"
	"encoding/json"
	"fmt"
	"io"
	"net/http"
)

func GetJSON(ctx context.Context, url string, out interface{}) error {
	return GetJSONWithAuth(ctx, url, "", out)
}

func GetJSONWithAuth(ctx context.Context, url string, token string, out interface{}) error {
	req, err := http.NewRequestWithContext(ctx, http.MethodGet, url, nil)
	if err != nil {
		return fmt.Errorf("%s: %w", url, err)
	}

	if token != "" {
		req.Header.Set("Authorization", fmt.Sprintf("Bearer %s", token))
	}

	res, err := http.DefaultClient.Do(req)
	if err != nil {
		return fmt.Errorf("%s: %w", url, err)
	}
	defer res.Body.Close()

	if res.StatusCode/100 != 2 {
		return fmt.Errorf("%s: HTTP status %d", url, res.StatusCode)
	}

	if err := json.NewDecoder(res.Body).Decode(out); err != nil {
		return fmt.Errorf("%s: %w", url, err)
	}

	return nil
}

func GetImage(ctx context.Context, problemID int, solutionID string) ([]byte, error) {
	var url string
	if solutionID != "" {
		url = fmt.Sprintf("https://icfpc2023-frontend-uadsges7eq-an.a.run.app/api/render?problem=%d&solution=%s", problemID, solutionID)
	} else {
		url = fmt.Sprintf("https://icfpc2023-frontend-uadsges7eq-an.a.run.app/api/render?problem=%d", problemID)
	}
	req, err := http.NewRequestWithContext(ctx, http.MethodGet, url, nil)
	if err != nil {
		return nil, fmt.Errorf("%s: %w", url, err)
	}

	res, err := http.DefaultClient.Do(req)
	if err != nil {
		return nil, fmt.Errorf("%s: %w", url, err)
	}
	defer res.Body.Close()

	if res.StatusCode/100 != 2 {
		return nil, fmt.Errorf("%s: HTTP status %d", url, res.StatusCode)
	}

	return io.ReadAll(res.Body)
}
