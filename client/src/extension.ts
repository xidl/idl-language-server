/* --------------------------------------------------------------------------------------------
 * Copyright (c) Microsoft Corporation. All rights reserved.
 * Licensed under the MIT License. See License.txt in the project root for license information.
 * ------------------------------------------------------------------------------------------ */

import * as fs from "node:fs";
import * as path from "node:path";
import { type ExtensionContext, window, workspace } from "vscode";

import {
	type Executable,
	LanguageClient,
	type LanguageClientOptions,
	type ServerOptions,
} from "vscode-languageclient/node";

let client: LanguageClient;

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
	client.start();
}

export function deactivate(): Thenable<void> | undefined {
	if (!client) {
		return undefined;
	}
	return client.stop();
}
