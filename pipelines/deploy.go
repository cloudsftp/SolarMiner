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
	_, err := b.PublishImage(ctx, source, actor, token)
	if err != nil {
		return err
	}

	_, err = b.Deploy(ctx, host, username, key)
	if err != nil {
		return err
	}

	return nil
}

// Publishes the image of the service to the github container registry
func (b *SolarMiner) PublishImage(
	ctx context.Context,
	source *dagger.Directory,
	actor string,
	token *dagger.Secret,
) (string, error) {
	return b.
		BuildImage(ctx, source).
		WithRegistryAuth("ghcr.io", actor, token).
		Publish(ctx, "ghcr.io/cloudsftp/solarminer-service:latest")
}

// Builds the image of the service
func (b *SolarMiner) BuildImage(
	ctx context.Context,
	source *dagger.Directory,
) *dagger.Container {
	return b.
		buildBaseImage(source).
		WithEntrypoint([]string{"/service"})
}

func (b *SolarMiner) buildBaseImage(
	source *dagger.Directory,
) *dagger.Container {
	executable := b.Build(source, serviceName)

	return dag.Container().
		From("alpine:"+AlpineVersion).
		WithFile("/service", executable)
}

// Deploys the backend of the service
func (b *SolarMiner) Deploy(
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
