package database

import (
	"compress/gzip"
	"context"
	"database/sql"
	"fmt"
	"io"
	"time"

	"cloud.google.com/go/storage"
	"github.com/google/uuid"
)

type Problem struct {
	ID int `json:"id"`
}

type Solution struct {
	UUID       string      `json:"uuid"`
	ProblemID  int         `json:"problem_id"`
	Created    time.Time   `json:"created"`
	Submission *Submission `json:"submission"`
}

type Submission struct {
	SolutionUUID string    `json:"solution_uuid"`
	ID           string    `json:"id"`
	State        string    `json:"state"`
	Accepted     bool      `json:"accepted"`
	Score        int64     `json:"score"`
	Error        string    `json:"error"`
	Created      time.Time `json:"created"`
}

type DB struct {
	raw    *sql.DB
	bucket *storage.BucketHandle
}

func New(raw *sql.DB, bucket *storage.BucketHandle) *DB {
	return &DB{
		raw:    raw,
		bucket: bucket,
	}
}

func (db *DB) Close() error {
	return db.raw.Close()
}

func (db *DB) ListProblems(ctx context.Context) ([]*Problem, error) {
	rows, err := db.raw.QueryContext(ctx, `SELECT id FROM problems ORDER BY id ASC`)
	if err != nil {
		return nil, err
	}
	defer rows.Close()

	var problems []*Problem
	for rows.Next() {
		var id int
		if err := rows.Scan(&id); err != nil {
			return nil, err
		}
		problems = append(problems, &Problem{ID: id})
	}
	return problems, nil
}

func (db *DB) GetProblem(ctx context.Context, id int) (*Problem, error) {
	row := db.raw.QueryRowContext(ctx, `SELECT id FROM problems where id = ?`, id)

	var _id int
	if err := row.Scan(&_id); err != nil {
		return nil, err
	}

	problem := &Problem{
		ID: id,
	}
	return problem, nil
}

func (db *DB) UpdateProblem(ctx context.Context, id int, spec string) error {
	// Create JSON on GCS.
	w := db.problemObject(id).NewWriter(ctx)
	w.ContentType = "application/json"
	w.ContentEncoding = "gzip"
	gz := gzip.NewWriter(w)
	if _, err := io.WriteString(gz, spec); err != nil {
		return err
	}
	if err := gz.Close(); err != nil {
		return err
	}
	if err := w.Close(); err != nil {
		return err
	}

	// Create or update an entry in DB.
	if _, err := db.raw.ExecContext(ctx, `REPLACE INTO problems (id) VALUES (?)`, id); err != nil {
		return err
	}
	return nil
}

func (db *DB) GetSolution(ctx context.Context, uuid string) (*Solution, error) {
	row := db.raw.QueryRowContext(ctx, querySolutions+`WHERE uuid = ?`, uuid)

	solution, err := scanSolution(row)
	if err != nil {
		return nil, err
	}
	return solution, nil
}

func (db *DB) ListSolutionsForProblem(ctx context.Context, problemID int) ([]*Solution, error) {
	rows, err := db.raw.QueryContext(ctx, querySolutions+`
	WHERE problem_id = ?
	ORDER BY created DESC`, problemID)
	if err != nil {
		return nil, err
	}
	defer rows.Close()

	var solutions []*Solution
	for rows.Next() {
		solution, err := scanSolution(rows)
		if err != nil {
			return nil, err
		}
		solutions = append(solutions, solution)
	}
	return solutions, nil
}

func (db *DB) ListAllSolutions(ctx context.Context) ([]*Solution, error) {
	rows, err := db.raw.QueryContext(ctx, querySolutions+`ORDER BY solutions.created DESC`)
	if err != nil {
		return nil, err
	}
	defer rows.Close()

	var solutions []*Solution
	for rows.Next() {
		solution, err := scanSolution(rows)
		if err != nil {
			return nil, err
		}
		solutions = append(solutions, solution)
	}
	return solutions, nil
}

func (db *DB) SubmitSolution(ctx context.Context, problemID int, solutionSpec string) (string, error) {
	// Ensure the problem exists.
	if _, err := db.GetProblem(ctx, problemID); err != nil {
		return "", err
	}

	uuid := uuid.New().String()

	// Create JSON on GCS.
	w := db.solutionObject(uuid).NewWriter(ctx)
	w.ContentType = "application/json"
	w.ContentEncoding = "gzip"
	gz := gzip.NewWriter(w)
	if _, err := io.WriteString(gz, solutionSpec); err != nil {
		return "", err
	}
	if err := gz.Close(); err != nil {
		return "", err
	}
	if err := w.Close(); err != nil {
		return "", err
	}

	// Finally create an entry in DB.
	if _, err := db.raw.ExecContext(ctx, `INSERT INTO solutions (uuid, problem_id) VALUES (?, ?)`, uuid, problemID); err != nil {
		return "", err
	}

	return uuid, nil
}

func (db *DB) GetSolutionBySubmissionID(ctx context.Context, submissionID string) (*Solution, error) {
	row := db.raw.QueryRowContext(ctx, querySolutions+`WHERE submissions.submission_id = ?`, submissionID)

	solution, err := scanSolution(row)
	if err != nil {
		return nil, err
	}
	return solution, nil
}

func (db *DB) ReplaceSubmission(ctx context.Context, submission *Submission) error {
	if _, err := db.raw.ExecContext(ctx, `
	REPLACE INTO submissions (uuid, submission_id, state, accepted, score, error, created)
	VALUES (?, ?, ?, ?, ?, ?, ?)
	`, submission.SolutionUUID, submission.ID, submission.State, submission.Accepted, submission.Score, submission.Error, submission.Created); err != nil {
		return err
	}
	return nil
}

func (db *DB) ProblemURL(id int) string {
	object := db.problemObject(id)
	return fmt.Sprintf("https://%s.storage.googleapis.com/%s", object.BucketName(), object.ObjectName())
}

func (db *DB) SolutionURL(uuid string) string {
	object := db.solutionObject(uuid)
	return fmt.Sprintf("https://%s.storage.googleapis.com/%s", object.BucketName(), object.ObjectName())
}

func (db *DB) problemObject(id int) *storage.ObjectHandle {
	return db.bucket.Object(fmt.Sprintf("problems/%d.json", id))
}

func (db *DB) solutionObject(uuid string) *storage.ObjectHandle {
	return db.bucket.Object(fmt.Sprintf("solutions/%s.json", uuid))
}

type rowScanner interface {
	Scan(values ...any) error
}

const querySolutions = `
SELECT
	solutions.uuid,
	solutions.problem_id,
	solutions.created,
	submissions.submission_id,
	submissions.state,
	submissions.accepted,
	submissions.score,
	submissions.error,
	submissions.created
FROM solutions
LEFT OUTER JOIN submissions USING (uuid)
`

func scanSolution(row rowScanner) (*Solution, error) {
	var uuid string
	var problemID int
	var solutionCreated time.Time
	var submissionID *string
	var state *string
	var accepted *bool
	var score *int64
	var errorMsg *string
	var submissionCreated *time.Time
	if err := row.Scan(&uuid, &problemID, &solutionCreated, &submissionID, &state, &accepted, &score, &errorMsg, &submissionCreated); err != nil {
		return nil, err
	}

	var submission *Submission
	if submissionID != nil {
		submission = &Submission{
			SolutionUUID: uuid,
			ID:           *submissionID,
			State:        *state,
			Accepted:     *accepted,
			Score:        *score,
			Error:        *errorMsg,
			Created:      *submissionCreated,
		}
	}

	solution := &Solution{
		UUID:       uuid,
		ProblemID:  problemID,
		Created:    solutionCreated,
		Submission: submission,
	}

	return solution, nil
}
