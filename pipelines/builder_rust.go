package main

import (
	"dagger/solar-miner/internal/dagger"

	"context"
)

const (
	RaspberryPiTarget = "aarch64-unknown-linux-musl"
)

// Runs a linter on the rust code
func (b *SolarMiner) LintRust(ctx context.Context, source *dagger.Directory) (string, error) {
	return cachedRustBuilder(source).
		WithExec([]string{"cargo", "clippy", "--", "-D", "warnings"}).
		Stdout(ctx)
}

// Builds the executable of a specified package
func (b *SolarMiner) BuildRust(
	source *dagger.Directory,
	packageName string,
) *dagger.File {
	return cachedRustBuilder(source).
		WithExec([]string{"cargo", "build", "-p", packageName, "--release"}).
		File("target/release/" + packageName)
}

// Builds the executable of a specified package for the arm architecture
func (b *SolarMiner) BuildRustCrossArm(
	source *dagger.Directory,
	packageName string,
) *dagger.File {
	return cachedRustBuilderCrossArm(source).
		WithExec([]string{
			"cargo", "build", "-p", packageName, "--release",
			"--target", RaspberryPiTarget,
		}).
		File("target/" + RaspberryPiTarget + "/release/" + packageName)
}

// Runs unit tests for the rust code
func (b *SolarMiner) TestRust(
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

		// Clang
		WithExec([]string{"apk", "update"}).
		WithExec([]string{
			"apk", "add", "--no-cache",
			"clang", "lld",
		}).
		WithEnvVariable("CC", "clang").

		// Clippy
		WithExec([]string{"rustup", "component", "add", "clippy"}).

		// Caches
		WithMountedCache("/cache/cargo", dag.CacheVolume("rust-packages")).
		WithEnvVariable("CARGO_HOME", "/cache/cargo").
		WithMountedCache("target", dag.CacheVolume("rust-target")).

		// Source code
		WithDirectory("/source", source).
		WithWorkdir("/source")
}

func cachedRustBuilderCrossArm(
	source *dagger.Directory,
) *dagger.Container {
	return cachedRustBuilder(source).
		WithExec([]string{"rustup", "target", "add", RaspberryPiTarget})
}
