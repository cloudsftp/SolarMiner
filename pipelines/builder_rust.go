package main

import (
	"dagger/solar-miner/internal/dagger"
)

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

func (b *SolarMiner) CrossCompileController(
	source *dagger.Directory,
) *dagger.File {
	return dag.Container().
		From("rust:"+RustVersion+"-alpine"+AlpineVersion).

		// Clang
		WithExec([]string{"apk", "update"}).
		WithExec([]string{
			"apk", "add", "--no-cache",
			//"musl-dev",
			"clang", "lld",
		}).

		// Target
		WithExec([]string{"rustup", "target", "add", "armv7-unknown-linux-musleabihf"}).

		// Caches
		//WithMountedCache("/cache/cargo", dag.CacheVolume("rust-packages")).
		//WithEnvVariable("CARGO_HOME", "/cache/cargo").
		//WithMountedCache("target", dag.CacheVolume("rust-target")).

		// Source code
		WithDirectory("/source", source).
		WithWorkdir("/source").
		WithEnvVariable("CC", "clang").
		WithExec([]string{"cargo", "build", "-p", controllerName, "--target", "armv7-unknown-linux-musleabihf"}).
		File("target/debug/solarminer-controller")
}
