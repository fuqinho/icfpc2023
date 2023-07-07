package main

import (
	"context"
	"fmt"
	"icfpc2023/backend/internal/server"
	"log"
	"net/http"
	"os"

	"github.com/urfave/cli/v3"
)

var flagPort = &cli.IntFlag{
	Name:  "port",
	Value: 8080,
}

var app = &cli.Command{
	Name: "backend",
	Flags: []cli.Flag{
		flagPort,
	},
	Action: func(c *cli.Context) error {
		port := c.Int(flagPort.Name)

		handler := server.NewHandler()
		log.Printf("Listening at :%d ...", port)
		return http.ListenAndServe(fmt.Sprintf(":%d", port), handler)
	},
}

func main() {
	app.Run(context.Background(), os.Args)
}
