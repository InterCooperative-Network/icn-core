const vscode = require('vscode');
const { LanguageClient, TransportKind } = require('vscode-languageclient/node');

let client;

function activate(context) {
  // Compile command
  const compileCmd = vscode.commands.registerCommand('icn-ccl.compile', () => {
    const editor = vscode.window.activeTextEditor;
    if (!editor) {
      vscode.window.showInformationMessage('No active CCL file');
      return;
    }
    const file = editor.document.fileName;
    const terminal = vscode.window.createTerminal('CCL Compile');
    terminal.show(true);
    terminal.sendText(`cargo run --bin icn-cli ccl compile "${file}"`);
  });

  // Debug command
  const debugCmd = vscode.commands.registerCommand('icn-ccl.debug', () => {
    const editor = vscode.window.activeTextEditor;
    if (!editor) {
      vscode.window.showInformationMessage('No active CCL file');
      return;
    }
    const file = editor.document.fileName;
    const terminal = vscode.window.createTerminal('CCL Debug');
    terminal.show(true);
    terminal.sendText(`cargo run --bin icn-ccl-debug`);
    // Send command to create debug session
    setTimeout(() => {
      const fileName = file.split('/').pop().replace('.ccl', '');
      const sourceContent = editor.document.getText();
      terminal.sendText(`create ${fileName} "${sourceContent.replace(/"/g, '\\"')}"`);
    }, 1000);
  });

  // Package commands
  const packageInitCmd = vscode.commands.registerCommand('icn-ccl.package.init', async () => {
    const name = await vscode.window.showInputBox({
      prompt: 'Enter package name',
      placeHolder: 'my-governance-package'
    });
    if (!name) return;

    const category = await vscode.window.showQuickPick(
      ['governance', 'economics', 'identity', 'utility', 'template'],
      { placeHolder: 'Select package category' }
    );
    if (!category) return;

    const terminal = vscode.window.createTerminal('CCL Package');
    terminal.show(true);
    terminal.sendText(`cargo run --bin icn-ccl-pkg init ${name} ${category}`);
  });

  const packageInstallCmd = vscode.commands.registerCommand('icn-ccl.package.install', async () => {
    const name = await vscode.window.showInputBox({
      prompt: 'Enter package name to install',
      placeHolder: 'liquid-democracy'
    });
    if (!name) return;

    const terminal = vscode.window.createTerminal('CCL Package');
    terminal.show(true);
    terminal.sendText(`cargo run --bin icn-ccl-pkg install ${name}`);
  });

  // Task provider
  const provider = vscode.tasks.registerTaskProvider('ccl', {
    provideTasks: () => {
      const editor = vscode.window.activeTextEditor;
      if (!editor) return [];
      const file = editor.document.fileName;
      
      return [
        new vscode.Task(
          { type: 'ccl', file, action: 'compile' },
          vscode.TaskScope.Workspace,
          'Compile Current CCL',
          'ccl',
          new vscode.ShellExecution(`cargo run --bin icn-cli ccl compile "${file}"`)
        ),
        new vscode.Task(
          { type: 'ccl', file, action: 'build' },
          vscode.TaskScope.Workspace,
          'Build CCL Package',
          'ccl',
          new vscode.ShellExecution(`cargo run --bin icn-ccl-pkg build`)
        )
      ];
    },
    resolveTask: (task) => task
  });

  // Language Server Client setup
  const config = vscode.workspace.getConfiguration('icn-ccl');
  if (config.get('lsp.enabled')) {
    const serverOptions = {
      command: 'cargo',
      args: ['run', '--bin', 'icn-ccl-lsp'],
      transport: TransportKind.stdio
    };

    const clientOptions = {
      documentSelector: [{ scheme: 'file', language: 'ccl' }],
      synchronize: {
        fileEvents: vscode.workspace.createFileSystemWatcher('**/.ccl')
      }
    };

    client = new LanguageClient(
      'icn-ccl-lsp',
      'ICN CCL Language Server',
      serverOptions,
      clientOptions
    );

    // Start the client and server
    client.start().catch(err => {
      console.log('Failed to start LSP server:', err);
      vscode.window.showWarningMessage('CCL Language Server failed to start. Advanced features may not be available.');
    });
  }

  context.subscriptions.push(
    compileCmd,
    debugCmd, 
    packageInitCmd,
    packageInstallCmd,
    provider
  );
}

function deactivate() {
  if (client) {
    return client.stop();
  }
}

module.exports = { activate, deactivate };
