package main

import (
	"dagger/solar-miner/internal/dagger"

	"context"
)

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

// Publishes the image of a rust program to the github container registry
func (b *SolarMiner) PublishRustImage(
	ctx context.Context,
	source *dagger.Directory,
	packageName string,
	actor string,
	token *dagger.Secret,
) (string, error) {
	return b.
		BuildRustImage(ctx, source, packageName).
		WithRegistryAuth("ghcr.io", actor, token).
		Publish(ctx, "ghcr.io/cloudsftp/"+packageName+":latest")
}

// Builds the image of a rust program
func (b *SolarMiner) BuildRustImage(
	ctx context.Context,
	source *dagger.Directory,
	packageName string,
) *dagger.Container {
	return b.
		buildBaseImage(source, packageName).
		WithEntrypoint([]string{"/" + packageName})
}

func (b *SolarMiner) buildBaseImage(
	source *dagger.Directory,
	packageName string,
) *dagger.Container {
	executable := b.Build(source, packageName)

	return dag.Container().
		From("alpine:"+AlpineVersion).
		WithFile("/"+packageName, executable)
}

// Deploys the backend of the service
func (b *SolarMiner) DeployService(
	ctx context.Context,
	host *dagger.Secret,
	username *dagger.Secret,
	key *dagger.Secret,
) (string, error) {
	usernamePlain, err := username.Plaintext(ctx)
	if err != nil {
		return "", err
	}

	hostPlain, err := host.Plaintext(ctx)
	if err != nil {
		return "", err
	}

	return NewSSH(
		usernamePlain+"@"+hostPlain,
		key,
		AlpineVersion,
	).Execute(ctx, "./deploy.sh")
}
