package main

import (
	"dagger/solar-miner/internal/dagger"

	"context"
)

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
