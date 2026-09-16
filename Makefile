.DEFAULT_GOAL := help

PI_HOST ?= danutz@192.168.0.25
export PI_HOST

.PHONY: help deploy

help:
	@printf '%s\n' 'make deploy  Build and redeploy Dario to the Pi on port 3041.' 'Override the SSH destination with PI_HOST=danutz@hostname.'

deploy:
	@sh scripts/deploy-pi.sh
