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

	_, err := b.BuildRust(source, serviceName).Name(ctx)
	if err != nil {
		return "", err
	}

	_, err = b.BuildRust(source, controllerName).Name(ctx)
	if err != nil {
		return "", err
	}

	_, err = b.BuildRustCrossArm(source, controllerName).Name(ctx)
	if err != nil {
		return "", err
	}

	_, err = b.BuildRust(source, tuiName).Name(ctx)
	if err != nil {
		return "", err
	}

	_, err = b.TestRust(ctx, source)
	if err != nil {
		return "", err
	}

	/*
		_, err := b.TestIntegration(ctx, source, mittlifeSource)
		if err != nil {
			return "", err
		}
	*/

	output := "SUCCESS"
	return output, nil
}

// Publishes all images and deploys the service to the backend
func (b *SolarMiner) PublishAndDeploy(
	ctx context.Context,
	source *dagger.Directory,
	actor string,
	token *dagger.Secret,
	host *dagger.Secret,
	username *dagger.Secret,
	key *dagger.Secret,
) error {
	err := b.PublishAndDeployService(ctx, source, actor, token, host, username, key)
	if err != nil {
		return err
	}

	err = b.PublishController(ctx, source, actor, token)
	if err != nil {
		return err
	}

	return nil
}

// Publishes and deploys the service to the backend
func (b *SolarMiner) PublishAndDeployService(
	ctx context.Context,
	source *dagger.Directory,
	actor string,
	token *dagger.Secret,
	host *dagger.Secret,
	username *dagger.Secret,
	key *dagger.Secret,
) error {
	serviceExecutable := b.BuildRust(source, serviceName)
	_, err := b.PublishRustImage(ctx, serviceExecutable, serviceName, actor, token)
	if err != nil {
		return err
	}

	_, err = b.DeployService(ctx, host, username, key)
	if err != nil {
		return err
	}

	return nil
}

// Publishes the controller images for both regular and ARM architectures
func (b *SolarMiner) PublishController(
	ctx context.Context,
	source *dagger.Directory,
	actor string,
	token *dagger.Secret,
) error {
	controllerExecutable := b.BuildRust(source, controllerName)
	_, err := b.PublishRustImage(ctx, controllerExecutable, controllerName, actor, token)
	if err != nil {
		return err
	}

	controllerExecutableArm := b.BuildRustCrossArm(source, controllerName)
	_, err = b.PublishRustImageCrossArm(ctx, controllerExecutableArm, controllerName, actor, token)
	if err != nil {
		return err
	}

	return nil
}
