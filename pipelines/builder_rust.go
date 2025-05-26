package main

import (
	"dagger/solar-miner/internal/dagger"

	"context"
)

// Runs a linter
func (b *SolarMiner) LintRust(ctx context.Context, source *dagger.Directory) (string, error) {
	return cachedRustBuilder(source).
		WithExec([]string{"cargo", "clippy", "--", "-D", "warnings"}).
		Stdout(ctx)
}

// Builds the service executable
func (b *SolarMiner) BuildRust(
	source *dagger.Directory,
	packageName string,
) *dagger.File {
	return cachedRustBuilder(source).
		WithExec([]string{"cargo", "build", "-p", packageName, "--release"}).
		File("target/release/" + packageName)
}

// Runs unit tests
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
		WithExec([]string{"rustup", "target", "add", "armv7-unknown-linux-musleabihf"})
}

func (b *SolarMiner) CrossCompileController(
	source *dagger.Directory,
) *dagger.File {
	return cachedRustBuilder(source).
		WithExec([]string{"rustup", "target", "add", "armv7-unknown-linux-musleabihf"}).
		WithExec([]string{"cargo", "build", "-p", controllerName, "--target", "armv7-unknown-linux-musleabihf"}).
		File("target/debug/solarminer-controller")
}
