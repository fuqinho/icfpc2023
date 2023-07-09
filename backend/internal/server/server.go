package server

import (
	"encoding/json"
	"errors"
	"fmt"
	"icfpc2023/backend/internal/database"
	"icfpc2023/backend/internal/httputil"
	"io"
	"net/http"
	"strconv"

	"cloud.google.com/go/storage"
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
	r.HandleFunc("/api/problems/{id}", h.handleProblem).Methods(http.MethodGet)
	r.HandleFunc("/api/problems/{id}/spec", h.handleProblemSpec).Methods(http.MethodGet)
	r.HandleFunc("/api/problems/{id}/solutions", h.handleProblemSolutions).Methods(http.MethodGet)
	r.HandleFunc("/api/problems/{id}/image", h.handleProblemImage).Methods(http.MethodGet)
	r.HandleFunc("/api/problems/{id}/best-solution", h.handleProblemBestSolution).Methods(http.MethodGet)
	r.HandleFunc("/api/solutions", h.handleSolutions).Methods(http.MethodGet)
	r.HandleFunc("/api/solutions/{uuid}", h.handleSolution).Methods(http.MethodGet)
	r.HandleFunc("/api/solutions/{uuid}/spec", h.handleSolutionSpec).Methods(http.MethodGet)
	r.HandleFunc("/api/solutions/{uuid}/image", h.handleSolutionImage).Methods(http.MethodGet)
	r.HandleFunc("/api/submit", h.handleSubmit).Methods(http.MethodPost)
	r.HandleFunc("/batch/update-problems", h.handleUpdateProblems).Methods(http.MethodPost)

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
	withJSONResponse(w, r, func() (any, error) {
		ctx := r.Context()
		problems, err := h.db.ListProblems(ctx)
		if err != nil {
			return nil, err
		}
		if problems == nil {
			problems = []*database.Problem{}
		}
		return problems, nil
	})
}

func (h *Handler) handleProblem(w http.ResponseWriter, r *http.Request) {
	withJSONResponse(w, r, func() (any, error) {
		ctx := r.Context()
		vars := mux.Vars(r)
		id, err := strconv.Atoi(vars["id"])
		if err != nil {
			return nil, err
		}

		problem, err := h.db.GetProblem(ctx, id)
		if err != nil {
			return nil, err
		}

		return problem, nil
	})
}

func (h *Handler) handleProblemSpec(w http.ResponseWriter, r *http.Request) {
	withResponse(w, r, func() error {
		ctx := r.Context()
		vars := mux.Vars(r)
		id, err := strconv.Atoi(vars["id"])
		if err != nil {
			return err
		}

		if _, err := h.db.GetProblem(ctx, id); err != nil {
			return err
		}

		w.Header().Set("Location", h.db.ProblemURL(id))
		w.WriteHeader(http.StatusFound)
		return nil
	})
}

func (h *Handler) handleProblemBestSolution(w http.ResponseWriter, r *http.Request) {
	withResponse(w, r, func() error {
		ctx := r.Context()
		vars := mux.Vars(r)
		id, err := strconv.Atoi(vars["id"])
		if err != nil {
			return err
		}

		solutions, err := h.db.ListSolutionsForProblem(ctx, id)
		if err != nil {
			return err
		}
		var best *database.Solution
		for _, solution := range solutions {
			if solution.Submission == nil {
				continue
			}
			if best == nil || (solution.Submission.Score > best.Submission.Score) {
				best = solution
			}
		}

		if best == nil {
			return errors.New("cannot find the best solution")
		}

		w.Header().Set("Location", h.db.SolutionURL(best.UUID))
		w.WriteHeader(http.StatusFound)
		return nil
	})
}

func (h *Handler) handleProblemSolutions(w http.ResponseWriter, r *http.Request) {
	withJSONResponse(w, r, func() (any, error) {
		ctx := r.Context()
		vars := mux.Vars(r)
		id, err := strconv.Atoi(vars["id"])
		if err != nil {
			return nil, err
		}

		solutions, err := h.db.ListSolutionsForProblem(ctx, id)
		if err != nil {
			return nil, err
		}
		if solutions == nil {
			solutions = []*database.Solution{}
		}
		return solutions, nil
	})
}

func (h *Handler) handleProblemImage(w http.ResponseWriter, r *http.Request) {
	withResponse(w, r, func() error {
		ctx := r.Context()
		vars := mux.Vars(r)
		id, err := strconv.Atoi(vars["id"])
		if err != nil {
			return err
		}

		if _, err := h.db.GetProblem(ctx, id); err != nil {
			return err
		}

		imageObj := h.db.ProblemImageObject(id)
		if _, err := imageObj.Attrs(r.Context()); err == storage.ErrObjectNotExist {
			bs, err := httputil.GetImage(r.Context(), id, "")
			if err != nil {
				return err
			}
			w := imageObj.NewWriter(ctx)
			w.ContentType = "image/png"
			if _, err := w.Write(bs); err != nil {
				return err
			}
			if err := w.Close(); err != nil {
				return err
			}
		}

		w.Header().Set("Location", h.db.ProblemImageURL(id))
		w.WriteHeader(http.StatusFound)
		return nil
	})
}

func (h *Handler) handleSolutions(w http.ResponseWriter, r *http.Request) {
	withJSONResponse(w, r, func() (any, error) {
		ctx := r.Context()
		solutions, err := h.db.ListAllSolutions(ctx)
		if err != nil {
			return nil, err
		}
		if solutions == nil {
			solutions = []*database.Solution{}
		}
		return solutions, nil
	})
}

func (h *Handler) handleSolution(w http.ResponseWriter, r *http.Request) {
	withJSONResponse(w, r, func() (any, error) {
		ctx := r.Context()
		vars := mux.Vars(r)
		uuid := vars["uuid"]

		solution, err := h.db.GetSolution(ctx, uuid)
		if err != nil {
			return nil, err
		}
		return solution, nil
	})
}

func (h *Handler) handleSolutionSpec(w http.ResponseWriter, r *http.Request) {
	withResponse(w, r, func() error {
		ctx := r.Context()
		vars := mux.Vars(r)
		uuid := vars["uuid"]

		if _, err := h.db.GetSolution(ctx, uuid); err != nil {
			return err
		}

		w.Header().Set("Location", h.db.SolutionURL(uuid))
		w.WriteHeader(http.StatusFound)
		return nil
	})
}

func (h *Handler) handleSolutionImage(w http.ResponseWriter, r *http.Request) {
	withResponse(w, r, func() error {
		ctx := r.Context()
		vars := mux.Vars(r)
		uuid := vars["uuid"]

		solution, err := h.db.GetSolution(ctx, uuid)
		if err != nil {
			return err
		}

		imageObj := h.db.SolutionImageObject(solution.UUID)
		if _, err := imageObj.Attrs(r.Context()); err == storage.ErrObjectNotExist {
			bs, err := httputil.GetImage(r.Context(), solution.ProblemID, solution.UUID)
			if err != nil {
				return err
			}
			w := imageObj.NewWriter(ctx)
			w.ContentType = "image/png"
			if _, err := w.Write(bs); err != nil {
				return err
			}
			if err := w.Close(); err != nil {
				return err
			}
		}

		w.Header().Set("Location", h.db.SolutionImageURL(solution.UUID))
		w.WriteHeader(http.StatusFound)
		return nil
	})
}

func (h *Handler) handleSubmit(w http.ResponseWriter, r *http.Request) {
	withResponse(w, r, func() error {
		ctx := r.Context()

		var req map[string]any
		if err := json.NewDecoder(r.Body).Decode(&req); err != nil {
			return err
		}

		problemIDFloat, ok := req["problem_id"].(float64)
		if !ok {
			return errors.New("problem ID missing")
		}
		problemID := int(problemIDFloat)

		solutionSpec, err := json.Marshal(req)
		if err != nil {
			return err
		}

		uuid, err := h.db.SubmitSolution(ctx, problemID, string(solutionSpec))
		if err != nil {
			return err
		}

		io.WriteString(w, uuid)
		return nil
	})
}

func (h *Handler) handleUpdateProblems(w http.ResponseWriter, r *http.Request) {
	withResponse(w, r, func() error {
		ctx := r.Context()
		force := r.Form.Get("force") == "true"

		var problemsResponse struct {
			NumberOfProblems int `json:"number_of_problems"`
		}
		if err := httputil.GetJSON(ctx, "http://api.icfpcontest.com/problems", &problemsResponse); err != nil {
			return err
		}

		knownProblemIDs := make(map[int]struct{})
		if !force {
			problems, err := h.db.ListProblems(ctx)
			if err != nil {
				return err
			}
			for _, p := range problems {
				knownProblemIDs[p.ID] = struct{}{}
			}
		}

		for id := 1; id <= problemsResponse.NumberOfProblems; id++ {
			if _, ok := knownProblemIDs[id]; ok {
				continue
			}

			var problemResponse struct {
				Success string `json:"Success"`
				Failure string `json:"Failure"`
			}

			if err := httputil.GetJSON(ctx, fmt.Sprintf("http://api.icfpcontest.com/problem?problem_id=%d", id), &problemResponse); err != nil {
				return err
			}

			if problemResponse.Success == "" {
				return fmt.Errorf("failed to get problem %d: %s", id, problemResponse.Failure)
			}

			if err := h.db.UpdateProblem(ctx, id, problemResponse.Success); err != nil {
				return err
			}
		}

		io.WriteString(w, "OK\n")
		return nil
	})
}

// withResponse is a helper function to implement a handler function that
// returns a response on success.
func withResponse(w http.ResponseWriter, r *http.Request, f func() error) {
	if err := f(); err != nil {
		w.WriteHeader(http.StatusInternalServerError)
		fmt.Fprintf(w, "ERROR: %v\n", err)
	}
}

// withJSONResponse is a helper function to implement a handler function that
// returns a JSON response.
func withJSONResponse(w http.ResponseWriter, r *http.Request, f func() (any, error)) {
	withResponse(w, r, func() error {
		res, err := f()
		if err != nil {
			return err
		}

		w.Header().Set("Content-Type", "application/json")
		w.WriteHeader(http.StatusOK)
		json.NewEncoder(w).Encode(res)
		return nil
	})
}
