package database

import (
	"compress/gzip"
	"context"
	"database/sql"
	"errors"
	"fmt"
	"io"
	"time"

	"cloud.google.com/go/storage"
)

type Problem struct {
	ID string `json:"id"`
}

type Solution struct {
	UUID      string    `json:"uuid"`
	ProblemID string    `json:"problem_id"`
	Created   time.Time `json:"created"`
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
		var id string
		if err := rows.Scan(&id); err != nil {
			return nil, err
		}
		problems = append(problems, &Problem{ID: id})
	}
	return problems, nil
}

func (db *DB) GetProblem(ctx context.Context, id string) (*Problem, error) {
	row := db.raw.QueryRowContext(ctx, `SELECT id FROM problems where id = ?`, id)

	var _id string
	if err := row.Scan(&_id); err != nil {
		return nil, err
	}

	problem := &Problem{
		ID: id,
	}
	return problem, nil
}

func (db *DB) AddProblem(ctx context.Context, id string, spec string) error {
	// Check if the problem already exists.
	if _, err := db.GetProblem(ctx, id); err == nil {
		return fmt.Errorf("problem %s already exists", id)
	} else if !errors.Is(err, sql.ErrNoRows) {
		return err
	}

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

	// Finally create an entry in DB.
	if _, err := db.raw.ExecContext(ctx, `INSERT INTO problems (id) VALUES (?)`, id); err != nil {
		return err
	}

	return nil
}

func (db *DB) ListSolutionsForProblem(ctx context.Context, problemID string) ([]*Solution, error) {
	rows, err := db.raw.QueryContext(ctx, `SELECT uuid, created FROM solutions WHERE problem_id = ? ORDER BY created DESC`, problemID)
	if err != nil {
		return nil, err
	}
	defer rows.Close()

	var solutions []*Solution
	for rows.Next() {
		var uuid string
		var created time.Time
		if err := rows.Scan(&uuid, &created); err != nil {
			return nil, err
		}
		solutions = append(solutions, &Solution{
			UUID:      uuid,
			ProblemID: problemID,
			Created:   created,
		})
	}
	return solutions, nil
}

func (db *DB) ListAllSolutions(ctx context.Context) ([]*Solution, error) {
	rows, err := db.raw.QueryContext(ctx, `SELECT uuid, problem_id, created FROM solutions ORDER BY created DESC`)
	if err != nil {
		return nil, err
	}
	defer rows.Close()

	var solutions []*Solution
	for rows.Next() {
		var uuid, problemID string
		var created time.Time
		if err := rows.Scan(&uuid, &problemID, &created); err != nil {
			return nil, err
		}
		solutions = append(solutions, &Solution{
			UUID:      uuid,
			ProblemID: problemID,
			Created:   created,
		})
	}
	return solutions, nil
}

func (db *DB) ProblemURL(id string) string {
	object := db.problemObject(id)
	return fmt.Sprintf("https://%s.storage.googleapis.com/%s", object.BucketName(), object.ObjectName())
}

func (db *DB) problemObject(id string) *storage.ObjectHandle {
	return db.bucket.Object(fmt.Sprintf("problems/%s.json", id))
}
