package main

import (
	"dagger/solar-miner/internal/dagger"

	"context"
)

// Publishes the image of a rust program to the github container registry
func (b *SolarMiner) PublishRustImage(
	ctx context.Context,
	executable *dagger.File,
	packageName string,
	actor string,
	token *dagger.Secret,
) (string, error) {
	return b.BuildDockerImage(executable, packageName).
		WithRegistryAuth("ghcr.io", actor, token).
		Publish(ctx, "ghcr.io/cloudsftp/"+packageName+":latest")
}

// Publishes the image of a rust program to the github container registry with an extra tag
func (b *SolarMiner) PublishRustImageCrossArm(
	ctx context.Context,
	executable *dagger.File,
	packageName string,
	actor string,
	token *dagger.Secret,
) (string, error) {
	return b.BuildDockerImage(executable, packageName).
		WithRegistryAuth("ghcr.io", actor, token).
		Publish(ctx, "ghcr.io/cloudsftp/"+packageName+":arm64")
}

func (b *SolarMiner) BuildDockerImage(
	executable *dagger.File,
	name string,
) *dagger.Container {
	return dag.Container().
		From("alpine:"+AlpineVersion).
		WithFile("/"+name, executable).
		WithEntrypoint([]string{"/" + name})
}

func (b *SolarMiner) BuildDockerImageCrossArm(
	executable *dagger.File,
	name string,
) *dagger.Container {
	return dag.Container(dagger.ContainerOpts{
		Platform: "arm64",
	}).
		From("alpine:"+AlpineVersion).
		WithFile("/"+name, executable).
		WithEntrypoint([]string{"/" + name})
}

func buildBaseImage(
	executable *dagger.File,
	name string,
) *dagger.Container {
	return dag.Container().
		From("alpine:"+AlpineVersion).
		WithFile("/"+name, executable)
}
