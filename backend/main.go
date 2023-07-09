package main

import (
	"context"
	"database/sql"
	"fmt"
	"icfpc2023/backend/internal/database"
	"icfpc2023/backend/internal/official"
	"icfpc2023/backend/internal/server"
	"icfpc2023/backend/internal/worker"
	"log"
	"net/http"
	"os"

	"cloud.google.com/go/storage"
	_ "github.com/go-sql-driver/mysql"
	"github.com/urfave/cli/v3"
)

var flagPort = &cli.IntFlag{
	Name:  "port",
	Value: 8080,
}

var flagDB = &cli.StringFlag{
	Name:     "db",
	Required: true,
}

var flagBucket = &cli.StringFlag{
	Name:     "bucket",
	Required: true,
}

var flagAPIKey = &cli.StringFlag{
	Name:     "api-key",
	Required: true,
}

var cmdServer = &cli.Command{
	Name: "server",
	Flags: []cli.Flag{
		flagPort,
		flagDB,
		flagBucket,
		flagAPIKey,
	},
	Action: func(c *cli.Context) error {
		ctx := c.Context

		port := c.Int(flagPort.Name)
		dbURL := c.String(flagDB.Name)
		bucketName := c.String(flagBucket.Name)
		apiKey := c.String(flagAPIKey.Name)

		rawDB, err := sql.Open("mysql", dbURL)
		if err != nil {
			return err
		}
		defer rawDB.Close()

		store, err := storage.NewClient(ctx)
		if err != nil {
			return err
		}
		bucket := store.Bucket(bucketName)

		db := database.New(rawDB, bucket)
		client := official.NewClient(apiKey)

		handler := server.NewHandler(db, client)
		log.Printf("Listening at :%d ...", port)
		return http.ListenAndServe(fmt.Sprintf(":%d", port), handler)
	},
}

var cmdWorker = &cli.Command{
	Name: "worker",
	Flags: []cli.Flag{
		flagDB,
		flagBucket,
		flagAPIKey,
	},
	Action: func(c *cli.Context) error {
		ctx := c.Context
		dbURL := c.String(flagDB.Name)
		bucketName := c.String(flagBucket.Name)
		apiKey := c.String(flagAPIKey.Name)

		rawDB, err := sql.Open("mysql", dbURL)
		if err != nil {
			return err
		}
		defer rawDB.Close()

		store, err := storage.NewClient(ctx)
		if err != nil {
			return err
		}
		bucket := store.Bucket(bucketName)

		db := database.New(rawDB, bucket)
		client := official.NewClient(apiKey)

		return worker.Run(ctx, db, client)
	},
}

var cmdTest = &cli.Command{
	Name: "test",
	Flags: []cli.Flag{
		flagAPIKey,
	},
	Action: func(c *cli.Context) error {
		ctx := c.Context
		apiKey := c.String(flagAPIKey.Name)

		client := official.NewClient(apiKey)

		submissions, err := client.ListAllSubmissions(ctx)
		if err != nil {
			return err
		}

		for _, s := range submissions {
			log.Print(*s)
		}
		spec, err := client.GetSubmissionSpec(ctx, submissions[0].ID)
		if err != nil {
			return err
		}
		log.Print(spec)

		return nil
	},
}

var app = &cli.Command{
	Name: "backend",
	Commands: []*cli.Command{
		cmdServer,
		cmdWorker,
		cmdTest,
	},
}

func main() {
	if err := app.Run(context.Background(), os.Args); err != nil {
		log.Fatalf("ERROR: %v", err)
	}
}
