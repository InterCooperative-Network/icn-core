# ICN CCL Tools

This VS Code extension adds basic language support for the
Cooperative Contract Language (CCL).

## Features

- Syntax highlighting for `.ccl` files
- Command and task to compile the current CCL file using `icn-cli`

## Usage

1. Install dependencies with `npm install` inside the `vscode-extension` folder.
2. Press `F5` in VS Code to launch an Extension Development Host.
3. Use the `CCL: Compile CCL File` command or run the `ccl: Compile Current CCL` task.

The task runs `icn-cli ccl compile <active file>` in a new terminal.
