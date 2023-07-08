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

	finishedSubmissionIDs map[string]struct{}
}

func newWorker(db *database.DB, client *official.Client) *worker {
	return &worker{
		db:                    db,
		client:                client,
		finishedSubmissionIDs: make(map[string]struct{}),
	}
}

func (w *worker) DownloadSubmissions(ctx context.Context) error {
	submissions, err := w.client.ListAllSubmissions(ctx)
	if err != nil {
		return err
	}

	for _, submission := range submissions {
		if _, ok := w.finishedSubmissionIDs[submission.ID]; ok {
			continue
		}

		// Consider creating a solution.
		var solutionUUID string
		if solution, err := w.db.GetSolutionBySubmissionID(ctx, submission.ID); err == nil {
			if solution.Submission.State == "FINISHED" {
				// No need to process again.
				w.finishedSubmissionIDs[submission.ID] = struct{}{}
				continue
			}
			solutionUUID = solution.UUID
		} else if errors.Is(err, sql.ErrNoRows) {
			log.Printf("Creating a solution for submission %s", submission.ID)

			spec, err := w.client.GetSubmissionSpec(ctx, submission.ID)
			if err != nil {
				return err
			}

			spec, err = patchSolutionSpec(spec, submission.ProblemID)
			if err != nil {
				return err
			}

			solutionUUID, err = w.db.SubmitSolution(ctx, submission.ProblemID, spec)
			if err != nil {
				return err
			}
		} else if err != nil {
			return err
		}

		// Create or update a submission.
		log.Printf("Updating submission %s", submission.ID)

		created, err := time.Parse("2006-01-02T15:04:05.999999999Z", submission.SubmittedAt)
		if err != nil {
			return err
		}

		newSubmission := &database.Submission{
			SolutionUUID: solutionUUID,
			ID:           submission.ID,
			Created:      created,
			Updated:      time.Now().UTC(),
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

		if submission.Done {
			w.finishedSubmissionIDs[submission.ID] = struct{}{}
		}
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
	log.Print("Worker started")

	worker := newWorker(db, client)

	for {
		if err := worker.Tick(ctx); err != nil {
			log.Printf("ERROR: %v", err)
		}
		log.Print(".")
		time.Sleep(5 * time.Second)
	}
}
