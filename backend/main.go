package main

import (
	"context"
	"database/sql"
	"fmt"
	"icfpc2023/backend/internal/database"
	"icfpc2023/backend/internal/server"
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

var app = &cli.Command{
	Name: "backend",
	Flags: []cli.Flag{
		flagPort,
		flagDB,
		flagBucket,
	},
	Action: func(c *cli.Context) error {
		ctx := c.Context

		port := c.Int(flagPort.Name)
		dbURL := c.String(flagDB.Name)
		bucketName := c.String(flagBucket.Name)

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

		handler := server.NewHandler(db)
		log.Printf("Listening at :%d ...", port)
		return http.ListenAndServe(fmt.Sprintf(":%d", port), handler)
	},
}

func main() {
	app.Run(context.Background(), os.Args)
}
