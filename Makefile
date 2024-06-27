.PHONY: help
help:
	@echo "Usage:"
	@echo "  make build    Build the project"
	@echo "  make publish  Publish the project into NPM"

.PHONY: build
build:
	# wasm-pack build --dev --target web
	wasm-pack build --release --target web

.PHONY: publish
publish:
	npm publish ./pkg --access public
