package main

import (
	"dagger/solar-miner/internal/dagger"

	"context"
)

type SolarMiner struct{}

const (
	AlpineVersion = "3.21"
	RustVersion   = "1.86"

	serviceName    = "solarminer-service"
	controllerName = "solarminer-controller"
	tuiName        = "solarminer-tui"
)

// Builds the service and runs all tests (none right now)
func (b *SolarMiner) BuildAndTestAll(
	ctx context.Context,
	source *dagger.Directory,
) (string, error) {
	/*
		_, err := b.LintRust(ctx, source)
		if err != nil {
			return "", err
		}
	*/

	b.BuildRust(source, serviceName)
	b.BuildRust(source, controllerName)
	b.BuildRust(source, tuiName)

	_, err := b.TestRust(ctx, source)
	if err != nil {
		return "", err
	}

	b.BuildRustImage(ctx, source, serviceName)
	b.BuildRustImage(ctx, source, controllerName)

	/*
		_, err := b.TestIntegration(ctx, source, mittlifeSource)
		if err != nil {
			return "", err
		}
	*/

	output := "SUCCESS"
	return output, nil
}

// Publishes and deploys the service to the backend
func (b *SolarMiner) PublishAndDeploy(
	ctx context.Context,
	source *dagger.Directory,
	actor string,
	token *dagger.Secret,
	host *dagger.Secret,
	username *dagger.Secret,
	key *dagger.Secret,
) error {
	_, err := b.PublishRustImage(ctx, source, serviceName, actor, token)
	if err != nil {
		return err
	}

	_, err = b.DeployService(ctx, host, username, key)
	if err != nil {
		return err
	}

	return nil
}
