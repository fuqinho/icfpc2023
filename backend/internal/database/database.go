package database

import (
	"context"
	"database/sql"
)

type Problem struct {
	ID string `json:"_id"`
}

type DB struct {
	raw *sql.DB
}

func New(raw *sql.DB) *DB {
	return &DB{raw: raw}
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
