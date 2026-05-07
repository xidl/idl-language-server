/* --------------------------------------------------------------------------------------------
 * Copyright (c) Microsoft Corporation. All rights reserved.
 * Licensed under the MIT License. See License.txt in the project root for license information.
 * ------------------------------------------------------------------------------------------ */

import * as fs from "node:fs";
import * as path from "node:path";
import {
	commands,
	type ExtensionContext,
	ViewColumn,
	window,
	workspace,
} from "vscode";

import {
	type Executable,
	LanguageClient,
	type LanguageClientOptions,
	type ServerOptions,
} from "vscode-languageclient/node";

let client: LanguageClient;

const CMD_INSPECT_HIR = "idl-language-server.inspectHir";
const CMD_INSPECT_TYPEDAST = "idl-language-server.inspectTypedAst";
const CMD_PREVIEW = "idl-language-server.preview";
const SERVER_CMD_INSPECT_HIR = "idl-language-server.inspect-hir";
const SERVER_CMD_INSPECT_TYPEDAST = "idl-language-server.inspect-typedast";
const SERVER_CMD_PREVIEW = "idl-language-server.preview";

function resolveServerCommand(context: ExtensionContext): string {
	const envPath =
		process.env.IDL_LANGUAGE_SERVER_PATH || process.env.SERVER_PATH;
	if (envPath && fs.existsSync(envPath)) {
		return envPath;
	}

	const binName =
		process.platform === "win32"
			? "idl-language-server.exe"
			: "idl-language-server";
	const bundledPath = context.asAbsolutePath(path.join("server", binName));
	if (fs.existsSync(bundledPath)) {
		return bundledPath;
	}

	return binName;
}

async function inspectCurrentDocument(
	command: string,
	title: string,
	jsonTitle: string,
) {
	const editor = window.activeTextEditor;
	if (!editor || editor.document.languageId !== "idl") {
		await window.showErrorMessage("Open an IDL file first.");
		return;
	}

	try {
		const result = await client.sendRequest("workspace/executeCommand", {
			command,
			arguments: [editor.document.uri.toString()],
		});

		const panel = window.createWebviewPanel(
			"idlInspectJson",
			title,
			ViewColumn.Beside,
			{
				enableFindWidget: true,
			},
		);
		panel.webview.html = renderInspectHtml(
			jsonTitle,
			JSON.stringify(result ?? null, null, 2),
		);
	} catch (error) {
		const message =
			error instanceof Error ? error.message : "Failed to inspect document.";
		await window.showErrorMessage(message);
	}
}

function renderInspectHtml(
	title: string,
	content: string,
	language = "json",
): string {
	return `<!DOCTYPE html>
<html lang="en">
<head>
	<meta charset="UTF-8" />
	<meta name="viewport" content="width=device-width, initial-scale=1.0" />
	<title>${escapeHtml(title)}</title>
	<link rel="stylesheet" href="https://cdnjs.cloudflare.com/ajax/libs/prism/1.29.0/themes/prism-tomorrow.min.css">
	<style>
		:root {
			color-scheme: light dark;
		}
		body {
			margin: 0;
			font-family: ui-monospace, SFMono-Regular, Menlo, Monaco, Consolas, monospace;
			background: var(--vscode-editor-background);
			color: var(--vscode-editor-foreground);
		}
		header {
			padding: 12px 16px;
			border-bottom: 1px solid var(--vscode-panel-border);
			font-size: 12px;
			letter-spacing: 0.08em;
			text-transform: uppercase;
			color: var(--vscode-descriptionForeground);
		}
		pre {
			margin: 0 !important;
			padding: 16px !important;
			white-space: pre-wrap !important;
			word-break: break-word !important;
			font-size: 13px !important;
			line-height: 1.5 !important;
			background: transparent !important;
		}
		code {
			font-family: inherit !important;
		}
	</style>
</head>
<body>
	<header>${escapeHtml(title)}</header>
	<pre><code class="language-${language}">${escapeHtml(content)}</code></pre>
	<script src="https://cdnjs.cloudflare.com/ajax/libs/prism/1.29.0/prism.min.js"></script>
	<script src="https://cdnjs.cloudflare.com/ajax/libs/prism/1.29.0/components/prism-rust.min.js"></script>
	<script src="https://cdnjs.cloudflare.com/ajax/libs/prism/1.29.0/components/prism-typescript.min.js"></script>
	<script src="https://cdnjs.cloudflare.com/ajax/libs/prism/1.29.0/components/prism-go.min.js"></script>
	<script src="https://cdnjs.cloudflare.com/ajax/libs/prism/1.29.0/components/prism-python.min.js"></script>
	<script src="https://cdnjs.cloudflare.com/ajax/libs/prism/1.29.0/components/prism-json.min.js"></script>
</body>
</html>`;
}

function escapeHtml(value: string): string {
	return value
		.replaceAll("&", "&amp;")
		.replaceAll("<", "&lt;")
		.replaceAll(">", "&gt;");
}

async function previewCurrentDocument() {
	const editor = window.activeTextEditor;
	if (!editor || editor.document.languageId !== "idl") {
		await window.showErrorMessage("Open an IDL file first.");
		return;
	}

	const plugins = [
		{ label: "HIR", value: "hir" },
		{ label: "Typed AST", value: "typed-ast" },
		{ label: "Rust", value: "rust" },
		{ label: "Rust Axum", value: "axum" },
		{ label: "Rust JSON-RPC", value: "rust-jsonrpc" },
		{ label: "TypeScript", value: "typescript" },
		{ label: "TypeScript Rest", value: "typescript-rest" },
		{ label: "Go", value: "go" },
		{ label: "Go Rest", value: "go-rest" },
		{ label: "Python", value: "python" },
		{ label: "Python Rest", value: "python-rest" },
		{ label: "OpenAPI", value: "openapi" },
		{ label: "OpenRPC", value: "open-rpc" },
	];

	const selected = await window.showQuickPick(plugins, {
		placeHolder: "Select a plugin to preview generated code",
	});

	if (!selected) {
		return;
	}

	try {
		const result = (await client.sendRequest("workspace/executeCommand", {
			command: SERVER_CMD_PREVIEW,
			arguments: [editor.document.uri.toString(), selected.value],
		})) as { content: string; language: string } | null;

		if (!result) {
			return;
		}

		const panel = window.createWebviewPanel(
			"idlPreview",
			`Preview ${selected.label}`,
			ViewColumn.Beside,
			{
				enableFindWidget: true,
			},
		);
		panel.webview.html = renderInspectHtml(
			selected.label,
			result.content,
			result.language,
		);
	} catch (error) {
		const message =
			error instanceof Error ? error.message : "Failed to preview document.";
		await window.showErrorMessage(message);
	}
}

export async function activate(context: ExtensionContext) {
	const traceOutputChannel = window.createOutputChannel(
		"IDL Language Server trace",
	);
	const command = resolveServerCommand(context);
	const run: Executable = {
		command,
		options: {
			env: {
				...process.env,
				// eslint-disable-next-line @typescript-eslint/naming-convention
				RUST_LOG: "debug",
			},
		},
	};
	const serverOptions: ServerOptions = {
		run,
		debug: run,
	};
	// If the extension is launched in debug mode then the debug server options are used
	// Otherwise the run options are used
	// Options to control the language client
	const clientOptions: LanguageClientOptions = {
		// Register the server for plain text documents
		documentSelector: [{ scheme: "file", language: "idl" }],
		synchronize: {
			// Notify the server about file changes to '.idlrc' files contained in the workspace
			fileEvents: workspace.createFileSystemWatcher("**/.idlrc"),
		},
		traceOutputChannel,
	};

	// Create the language client and start the client.
	client = new LanguageClient(
		"idl-language-server",
		"IDL language server",
		serverOptions,
		clientOptions,
	);
	context.subscriptions.push(await client.start());

	context.subscriptions.push(
		commands.registerCommand(CMD_INSPECT_HIR, async () => {
			await inspectCurrentDocument(
				SERVER_CMD_INSPECT_HIR,
				"Inspect HIR",
				"HIR",
			);
		}),
	);
	context.subscriptions.push(
		commands.registerCommand(CMD_INSPECT_TYPEDAST, async () => {
			await inspectCurrentDocument(
				SERVER_CMD_INSPECT_TYPEDAST,
				"Inspect Typesast",
				"TypedAst",
			);
		}),
	);
	context.subscriptions.push(
		commands.registerCommand(CMD_PREVIEW, async () => {
			await previewCurrentDocument();
		}),
	);
}

export function deactivate(): Thenable<void> | undefined {
	if (!client) {
		return undefined;
	}
	return client.stop();
}
