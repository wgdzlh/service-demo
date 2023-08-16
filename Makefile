# grep the version
VERSION=v$(shell cat Cargo.toml | sed -nr 's/^version = "([^"]+)"/\1/p')

APP=service-demo
REPO=k8shubtest.com:1180/dev
IMAGE=$(REPO)/$(APP):$(VERSION)
PORT=8080

# This will output the help for each task
# thanks to https://marmelab.com/blog/2016/02/29/auto-documented-makefile.html
.PHONY: help build build-nc run up stop push release info

help: ## This help
	@awk 'BEGIN {FS = ":.*?## "} /^[a-zA-Z0-9_-]+:.*?## / {printf "\033[36m%-30s\033[0m %s\n", $$1, $$2}' $(MAKEFILE_LIST)

.DEFAULT_GOAL := help

# DOCKER TASKS
# Build the image
build: ## Build the image for dev
	docker build -t $(IMAGE) . --build-arg APP_VERSION="$(VERSION)-`git rev-parse --short HEAD`"

build-nc: ## Build the image for dev without caching
	docker build --no-cache -t $(IMAGE) . --build-arg APP_VERSION="$(VERSION)-`git rev-parse --short HEAD`"

run: ## Run container
	docker run -it --rm -v $(CURDIR)/config.toml:/app/config.toml -p $(PORT):$(PORT) --name $(APP) $(IMAGE) -l

up: build run ## Build and Run

stop: ## Stop a running container
	docker stop $(APP)

push: ## Push image to test repo
	docker push $(IMAGE)

release: build push ## Build and Push

info: ## Output make context
	@echo "$(VERSION) - $(CURDIR) - $(IMAGE) - $(PORT)"
