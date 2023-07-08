package httputil

import (
	"context"
	"encoding/json"
	"fmt"
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
