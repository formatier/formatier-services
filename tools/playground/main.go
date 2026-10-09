package main

import (
	"context"
	"errors"
	"fmt"
	"os"
	"regexp"

	"github.com/urfave/cli/v3"
)

var tagRegex = regexp.MustCompile("^([a-z0-9]+(?:-[a-z0-9]+)*)-v([0-9]+.[0-9]+.[0-9]+)$")

func main() {
	cmd := &cli.Command{
		Name:  "parse-tag",
		Usage: "parse git tag",
		Flags: []cli.Flag{
			&cli.BoolFlag{
				Name: "github-output",
			},
		},
		Arguments: []cli.Argument{
			&cli.StringArg{
				Name: "package-name",
			},
		},
		Action: func(ctx context.Context, cmd *cli.Command) error {
			refTag := cmd.Args().Get(0)
			tags := tagRegex.FindStringSubmatch(refTag)

			if len(tags) < 3 {
				return errors.New("invalid tag")
			}

			packageName := tags[1]
			version := tags[2]

			githubOutput, ok := cmd.Flags[0].Get().(bool)
			if !ok {
				return errors.New("cannot get flag")
			}

			if githubOutput {
				err := setOutput("package", packageName)
				err = setOutput("version", version)
				if err != nil {
					return err
				}
			} else {
				fmt.Printf(
					"package=%s\nversion=%s\n",
					packageName,
					version,
				)
			}
			return nil
		},
	}

	err := cmd.Run(context.Background(), os.Args)
	if err != nil {
		panic(err)
	}
}

func setOutput(key, value string) error {
	path := os.Getenv("GITHUB_OUTPUT")
	f, err := os.OpenFile(path, os.O_APPEND|os.O_WRONLY|os.O_CREATE, 0o644)
	if err != nil {
		return fmt.Errorf("open GITHUB_OUTPUT: %w", err)
	}
	defer f.Close()

	_, err = fmt.Fprintf(f, "%s=%s\n", key, value)
	return err
}
