package main

import (
	"fmt"
	"io"
	"net/http"
	"os"

	"github.com/joho/godotenv"
	"github.com/sirupsen/logrus"
)

const (
	apiUrl = "https://monitoringapi.solaredge.com/site/"
)

func main() {
	err := godotenv.Load()
	if err != nil {
		logrus.WithError(err).Fatal("Error loading .env file")
	}

	siteId := os.Getenv("SITE_ID")
	if "" == siteId {
		logrus.Fatal("Could not load SITE_ID from environment")
	}

	apiKey := os.Getenv("API_KEY")
	if "" == apiKey {
		logrus.Fatal("Could not load API_KEY from environment")
	}

	baseUrl := apiUrl + siteId + "/"
	url := baseUrl + "currentPowerFlow?api_key=" + apiKey

	resp, err := http.Get(url)
	if err != nil {
		logrus.WithError(err).Fatal("Could not get current power flow")
	}
	defer resp.Body.Close()

	body, err := io.ReadAll(resp.Body)
	if err != nil {
		logrus.WithError(err).Fatal("Could not read body")
	}

	fmt.Println("Body:", string(body))
}
