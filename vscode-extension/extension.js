const vscode = require('vscode');
const { LanguageClient, TransportKind } = require('vscode-languageclient/node');
const path = require('path');
const { spawn } = require('child_process');

let client;

function activate(context) {
  // Start Language Server if enabled
  const config = vscode.workspace.getConfiguration('ccl');
  if (config.get('lsp.enabled', true)) {
    startLanguageServer(context);
  }

  // Register commands
  const compileCmd = vscode.commands.registerCommand('icn-ccl.compile', () => {
    const editor = vscode.window.activeTextEditor;
    if (!editor) {
      vscode.window.showInformationMessage('No active CCL file');
      return;
    }
    const file = editor.document.fileName;
    const terminal = vscode.window.createTerminal('CCL Compile');
    terminal.show(true);
    terminal.sendText(`cargo run -p icn-ccl --bin ccl-lsp --quiet -- compile "${file}"`);
  });

  const debugCmd = vscode.commands.registerCommand('icn-ccl.debug', () => {
    const editor = vscode.window.activeTextEditor;
    if (!editor) {
      vscode.window.showInformationMessage('No active CCL file');
      return;
    }
    vscode.window.showInformationMessage('CCL debugging support coming soon!');
  });

  const initPackageCmd = vscode.commands.registerCommand('icn-ccl.package.init', () => {
    const workspaceFolder = vscode.workspace.workspaceFolders?.[0];
    if (!workspaceFolder) {
      vscode.window.showErrorMessage('No workspace folder open');
      return;
    }
    vscode.window.showInformationMessage('CCL package manager coming soon!');
  });

  const installPackageCmd = vscode.commands.registerCommand('icn-ccl.package.install', () => {
    vscode.window.showInformationMessage('CCL package manager coming soon!');
  });

  const provider = vscode.tasks.registerTaskProvider('ccl', {
    provideTasks: () => {
      const editor = vscode.window.activeTextEditor;
      if (!editor) return [];
      const file = editor.document.fileName;
      const task = new vscode.Task(
        { type: 'ccl', file },
        vscode.TaskScope.Workspace,
        'Compile Current CCL',
        'ccl',
        new vscode.ShellExecution(`cargo run -p icn-ccl --quiet -- compile "${file}"`)
      );
      return [task];
    },
    resolveTask: (task) => task
  });

  context.subscriptions.push(compileCmd, debugCmd, initPackageCmd, installPackageCmd, provider);
}

function startLanguageServer(context) {
  const config = vscode.workspace.getConfiguration('ccl');
  let serverPath = config.get('lsp.serverPath', '');
  
  if (!serverPath) {
    // Try to find the CCL LSP server binary
    serverPath = findCclLspServer();
  }

  if (!serverPath) {
    vscode.window.showWarningMessage(
      'CCL Language Server not found. Install it using: cargo install --path icn-ccl --bin ccl-lsp'
    );
    return;
  }

  const serverOptions = {
    command: serverPath,
    transport: TransportKind.stdio,
  };

  const clientOptions = {
    documentSelector: [{ scheme: 'file', language: 'ccl' }],
    synchronize: {
      fileEvents: vscode.workspace.createFileSystemWatcher('**/*.ccl')
    }
  };

  client = new LanguageClient(
    'ccl-lsp',
    'CCL Language Server',
    serverOptions,
    clientOptions
  );

  client.start().then(
    () => {
      vscode.window.showInformationMessage('CCL Language Server started');
    },
    (error) => {
      vscode.window.showErrorMessage(`Failed to start CCL Language Server: ${error}`);
    }
  );

  context.subscriptions.push(client);
}

function findCclLspServer() {
  // Try different common locations for the CCL LSP server
  const possiblePaths = [
    'ccl-lsp',  // In PATH
    'cargo run -p icn-ccl --bin ccl-lsp --quiet --',  // Development build
    path.join(process.env.HOME || '', '.cargo', 'bin', 'ccl-lsp'),  // Cargo install
  ];

  for (const serverPath of possiblePaths) {
    try {
      // Test if the server exists and is executable
      if (serverPath.includes('cargo run')) {
        return serverPath;
      }
      
      const result = spawn(serverPath, ['--version'], { stdio: 'ignore' });
      if (result) {
        return serverPath;
      }
    } catch (error) {
      // Continue to next path
    }
  }

  return null;
}

function deactivate() {
  if (client) {
    return client.stop();
  }
}

module.exports = { activate, deactivate };
