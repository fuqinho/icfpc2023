package server

import (
	"io"
	"net/http"

	"github.com/gorilla/mux"
)

type Handler struct {
	router *mux.Router
}

var _ http.Handler = &Handler{}

func NewHandler() *Handler {
	h := &Handler{
		router: nil, // assigned later
	}

	// Set up routes.
	r := mux.NewRouter()
	r.HandleFunc("/api/health", h.handleHealth).Methods(http.MethodGet)

	h.router = r
	return h
}

func (h *Handler) ServeHTTP(w http.ResponseWriter, r *http.Request) {
	w.Header().Set("Access-Control-Allow-Origin", "*")
	h.router.ServeHTTP(w, r)
}

func (h *Handler) handleHealth(w http.ResponseWriter, r *http.Request) {
	io.WriteString(w, "ok\n")
}
