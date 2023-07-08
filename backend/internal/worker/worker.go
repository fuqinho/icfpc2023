package worker

import (
	"context"
	"database/sql"
	"encoding/json"
	"errors"
	"fmt"
	"icfpc2023/backend/internal/database"
	"icfpc2023/backend/internal/official"
	"log"
	"strings"
	"time"
)

func patchSolutionSpec(spec string, problemID int) (string, error) {
	var partial struct {
		ProblemID int `json:"problem_id"`
	}
	if err := json.Unmarshal([]byte(spec), &partial); err != nil {
		return "", err
	}

	if partial.ProblemID != 0 {
		return spec, nil
	}

	spec = strings.TrimSpace(spec)
	if !strings.HasPrefix(spec, "{") {
		return "", errors.New("solution spec does not start with {")
	}

	spec = fmt.Sprintf("{\"problem_id\":%d,%s", problemID, spec[1:])

	// Ensure validity.
	if err := json.Unmarshal([]byte(spec), &json.RawMessage{}); err != nil {
		return "", err
	}
	return spec, nil
}

type worker struct {
	db     *database.DB
	client *official.Client

	knownSubmissionIDs map[string]struct{}
}

func newWorker(db *database.DB, client *official.Client) *worker {
	return &worker{
		db:                 db,
		client:             client,
		knownSubmissionIDs: make(map[string]struct{}),
	}
}

func (w *worker) DownloadSubmissions(ctx context.Context) error {
	submissions, err := w.client.ListAllSubmissions(ctx)
	if err != nil {
		return err
	}

	for _, submission := range submissions {
		if _, ok := w.knownSubmissionIDs[submission.ID]; ok {
			continue
		}

		if _, err := w.db.GetSolutionBySubmissionID(ctx, submission.ID); err == nil {
			w.knownSubmissionIDs[submission.ID] = struct{}{}
			continue
		} else if !errors.Is(err, sql.ErrNoRows) {
			return err
		}

		log.Printf("Downloading submission %s", submission.ID)

		spec, err := w.client.GetSubmissionSpec(ctx, submission.ID)
		if err != nil {
			return err
		}

		spec, err = patchSolutionSpec(spec, submission.ProblemID)
		if err != nil {
			return err
		}

		solutionUUID, err := w.db.SubmitSolution(ctx, submission.ProblemID, spec)
		if err != nil {
			return err
		}

		created, err := time.Parse("2006-01-02T15:04:05.999999999Z", submission.SubmittedAt)
		if err != nil {
			return err
		}

		newSubmission := &database.Submission{
			SolutionUUID: solutionUUID,
			ID:           submission.ID,
			Created:      created,
		}
		if submission.Done {
			newSubmission.State = "FINISHED"
			newSubmission.Error = submission.Error
			newSubmission.Score = submission.Score
			newSubmission.Accepted = submission.Error == ""
		} else {
			newSubmission.State = "PROCESSING"
		}

		if err := w.db.ReplaceSubmission(ctx, newSubmission); err != nil {
			return err
		}

		w.knownSubmissionIDs[submission.ID] = struct{}{}
	}

	return nil
}

func (w *worker) Tick(ctx context.Context) error {
	if err := w.DownloadSubmissions(ctx); err != nil {
		return err
	}
	return nil
}

func Run(ctx context.Context, db *database.DB, client *official.Client) error {
	worker := newWorker(db, client)

	for {
		if err := worker.Tick(ctx); err != nil {
			log.Printf("ERROR: %v", err)
		}
		log.Print(".")
		time.Sleep(5 * time.Second)
	}
}
