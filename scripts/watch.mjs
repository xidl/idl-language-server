import { spawn } from "node:child_process";

const commands = [
	["pnpm", ["run", "watch:esbuild"]],
	["pnpm", ["run", "watch:tsc"]],
];

const children = commands.map(([cmd, args]) => {
	const child = spawn(cmd, args, {
		stdio: "inherit",
		shell: process.platform === "win32",
	});
	return child;
});

let exitCode = 0;
let exiting = false;

function shutdown(code) {
	if (exiting) {
		return;
	}
	exiting = true;
	exitCode = code ?? exitCode;
	for (const child of children) {
		child.kill();
	}
	process.exit(exitCode);
}

for (const child of children) {
	child.on("exit", (code) => {
		if (typeof code === "number" && code !== 0) {
			shutdown(code);
		}
	});
}

process.on("SIGINT", () => shutdown(130));
process.on("SIGTERM", () => shutdown(143));
