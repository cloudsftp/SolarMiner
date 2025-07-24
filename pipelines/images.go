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
	return buildDockerImage(executable, packageName).
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
	return buildDockerImageCrossArm(executable, packageName).
		WithRegistryAuth("ghcr.io", actor, token).
		Publish(ctx, "ghcr.io/cloudsftp/"+packageName+":latest-arm64")
}

func buildDockerImage(
	executable *dagger.File,
	name string,
) *dagger.Container {
	return dag.Container().
		From("alpine:"+AlpineVersion).
		WithFile("/"+name, executable).
		WithEntrypoint([]string{"/" + name})
}

func buildDockerImageCrossArm(
	executable *dagger.File,
	name string,
) *dagger.Container {
	return dag.Container(dagger.ContainerOpts{
		Platform: "linux/arm64/v8",
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
