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
		_, err := b.Lint(ctx, source)
		if err != nil {
			return "", err
		}
	*/

	b.Build(source, serviceName)
	b.Build(source, controllerName)
	b.Build(source, tuiName)

	_, err := b.Test(ctx, source)
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

// Runs a linter
func (b *SolarMiner) Lint(ctx context.Context, source *dagger.Directory) (string, error) {
	return cachedRustBuilder(source).
		WithExec([]string{"cargo", "clippy", "--", "-D", "warnings"}).
		Stdout(ctx)
}

// Builds the service executable
func (b *SolarMiner) Build(
	source *dagger.Directory,
	packageName string,
) *dagger.File {
	return cachedRustBuilder(source).
		WithExec([]string{"cargo", "build", "-p", packageName, "--release"}).
		File("target/release/" + packageName)
}

// Runs unit tests
func (b *SolarMiner) Test(
	ctx context.Context,
	source *dagger.Directory,
) (string, error) {
	return cachedRustBuilder(source).
		WithExec([]string{"cargo", "test"}).
		Stdout(ctx)
}
