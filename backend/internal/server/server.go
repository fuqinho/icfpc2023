package server

import (
	"encoding/json"
	"fmt"
	"icfpc2023/backend/internal/database"
	"io"
	"net/http"

	"github.com/gorilla/mux"
)

type Handler struct {
	db     *database.DB
	router *mux.Router
}

var _ http.Handler = &Handler{}

func NewHandler(db *database.DB) *Handler {
	h := &Handler{
		db:     db,
		router: nil, // assigned later
	}

	// Set up routes.
	r := mux.NewRouter()
	r.HandleFunc("/api/health", h.handleHealth).Methods(http.MethodGet)
	r.HandleFunc("/api/problems", h.handleProblems).Methods(http.MethodGet)

	h.router = r
	return h
}

func (h *Handler) ServeHTTP(w http.ResponseWriter, r *http.Request) {
	// Set CORS header to all responses.
	w.Header().Set("Access-Control-Allow-Origin", "*")
	h.router.ServeHTTP(w, r)
}

func (h *Handler) handleHealth(w http.ResponseWriter, r *http.Request) {
	io.WriteString(w, "ok\n")
}

func (h *Handler) handleProblems(w http.ResponseWriter, r *http.Request) {
	withJSONResponse(w, r, func() (interface{}, error) {
		problems, err := h.db.ListProblems(r.Context())
		if err != nil {
			return nil, err
		}
		if problems == nil {
			problems = []*database.Problem{}
		}
		return problems, nil
	})
}

/// withJSONResponse is a helper function to implement a handler function that
/// returns a JSON response.
func withJSONResponse(w http.ResponseWriter, r *http.Request, f func() (interface{}, error)) {
	res, err := f()
	if err != nil {
		w.WriteHeader(http.StatusInternalServerError)
		fmt.Fprintf(w, "ERROR: %v\n", err)
		return
	}

	w.Header().Set("Content-Type", "application/json")
	w.WriteHeader(http.StatusOK)
	json.NewEncoder(w).Encode(res)
}
