package main

import (
	"dagger/solar-miner/internal/dagger"

	"context"
	"time"
)

type SSH struct {
	BaseContainer *dagger.Container
	Destination   string
	Key           *dagger.Secret
}

// NesSSH creates a new ssh executor instance
func NewSSH(
	// Destination to connect to (ssh destination)
	destination string,
	// Private key to connect
	key *dagger.Secret,
	// Version of alpine to use
	alpineVersion string,
) *SSH {
	container := dag.
		Container().
		From("alpine:"+alpineVersion).
		WithExec([]string{
			"apk", "add", "--no-cache", "openssh-client",
		}).
		WithEnvVariable("CACHE_BUSTER", time.Now().String()).
		WithMountedSecret("key", key)

	return &SSH{
		Destination:   destination,
		BaseContainer: container,
	}
}

// Execute executes a command with via the specified ssh connection
func (s *SSH) Execute(
	ctx context.Context,
	command ...string,
) (string, error) {
	exec := append(
		[]string{
			"ssh",
			"-o", "StrictHostKeyChecking=accept-new",
			"-i", "key",
			s.Destination,
		},
		command...,
	)

	return s.BaseContainer.WithExec(exec).Stdout(ctx)
}
