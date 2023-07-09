package worker

import (
	"bytes"
	"context"
	"database/sql"
	"encoding/json"
	"errors"
	"fmt"
	"icfpc2023/backend/internal/database"
	"icfpc2023/backend/internal/httputil"
	"icfpc2023/backend/internal/official"
	"log"
	"net/http"
	"os"
	"time"
)

func patchSolutionSpec(spec string, problemID int) (string, error) {
	var values map[string]any
	if err := json.Unmarshal([]byte(spec), &values); err != nil {
		return "", err
	}

	if p, ok := values["problem_id"].(float64); ok && p != 0 {
		return spec, nil
	}
	values["problem_id"] = problemID

	b, err := json.Marshal(values)
	if err != nil {
		return "", nil
	}

	return string(b), nil
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

func (w *worker) EvaluateSolutions(ctx context.Context) error {
	const endpoint = "https://icfpc2023-frontend-uadsges7eq-an.a.run.app/api/evaluate"

	solutions, err := w.db.ListUnevaluatedSolutions(ctx)
	if err != nil {
		return err
	}

	for _, solution := range solutions {
		log.Printf("Evaluating solution %s", solution.UUID)

		problemSpec, err := os.ReadFile(fmt.Sprintf("../problems/%d.json", solution.ProblemID))
		if err != nil {
			return err
		}

		solutionSpecURL := w.db.SolutionURL(solution.UUID)
		solutionSpec, err := httputil.Get(ctx, solutionSpecURL)
		if err != nil {
			return err
		}

		score, err := func() (int64, error) {
			req, err := json.Marshal(struct {
				Problem  json.RawMessage `json:"problem"`
				Solution json.RawMessage `json:"solution"`
			}{
				Problem:  json.RawMessage(problemSpec),
				Solution: json.RawMessage(solutionSpec),
			})
			if err != nil {
				return 0, err
			}

			res, err := http.Post(endpoint, "application/json", bytes.NewBuffer(req))
			if err != nil {
				return 0, err
			}
			defer res.Body.Close()

			if res.StatusCode/100 != 2 {
				return 0, fmt.Errorf("HTTP status %d", res.StatusCode)
			}

			var result struct {
				Score int64 `json:"score"`
			}
			if err := json.NewDecoder(res.Body).Decode(&result); err != nil {
				return 0, err
			}

			return result.Score, nil
		}()
		if err != nil {
			return err
		}

		// TODO: Handle rejected cases.
		evaluation := &database.Evaluation{
			SolutionUUID: solution.UUID,
			Accepted:     true,
			Score:        score,
			Error:        "",
			Created:      time.Now().UTC(),
		}

		if err := w.db.ReplaceEvaluation(ctx, evaluation); err != nil {
			return err
		}
	}
	return nil
}

func (w *worker) Tick(ctx context.Context) error {
	if err := w.DownloadSubmissions(ctx); err != nil {
		return err
	}
	if err := w.EvaluateSolutions(ctx); err != nil {
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
