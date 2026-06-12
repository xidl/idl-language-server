.PHONY: build-idea build-server build-vscode build clean help idea server vscode

help:
	@echo "Available commands:"
	@echo "  make build-idea    - Build the IntelliJ extension (alias: make idea)"
	@echo "  make build-server  - Build the Rust language server (alias: make server)"
	@echo "  make build-vscode  - Build the VS Code extension (alias: make vscode)"
	@echo "  make build         - Build all components"
	@echo "  make clean         - Clean all build artifacts"

idea: build-idea
server: build-server
vscode: build-vscode

build-idea:
	@echo "Building IntelliJ extension..."
	cd intellij-extension && ./gradlew buildPlugin

build-server:
	@echo "Building Rust language server..."
	cargo build --release

build-vscode:
	@echo "Building VS Code extension..."
	pnpm install
	pnpm run compile

build: build-server build-vscode build-idea

clean:
	@echo "Cleaning artifacts..."
	cargo clean
	cd intellij-extension && ./gradlew clean
	rm -rf dist
