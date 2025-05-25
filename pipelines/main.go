// A generated module for SolarMiner functions
//
// This module has been generated via dagger init and serves as a reference to
// basic module structure as you get started with Dagger.
//
// Two functions have been pre-created. You can modify, delete, or add to them,
// as needed. They demonstrate usage of arguments and return types using simple
// echo and grep commands. The functions can be called from the dagger CLI or
// from one of the SDKs.
//
// The first line in this comment block is a short description line and the
// rest is a long description with more detail on the module's purpose or usage,
// if appropriate. All modules should have a short description.

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

func cachedRustBuilder(
	source *dagger.Directory,
) *dagger.Container {
	return dag.Container().
		From("rust:"+RustVersion+"-alpine"+AlpineVersion).

		// Openssl
		WithExec([]string{"apk", "update"}).
		WithExec([]string{
			"apk", "add", "--no-cache",
			"pkgconfig", "musl-dev",
			"openssl-dev", "openssl-libs-static",
		}).

		// Clippy
		WithExec([]string{"rustup", "component", "add", "clippy"}).

		// Caches
		WithMountedCache("/cache/cargo", dag.CacheVolume("rust-packages")).
		WithEnvVariable("CARGO_HOME", "/cache/cargo").
		WithMountedCache("target", dag.CacheVolume("rust-target")).

		// Source code
		WithDirectory("/service", source).
		WithWorkdir("/service")
}
