const vscode = require('vscode');

function activate(context) {
  const compileCmd = vscode.commands.registerCommand('icn-ccl.compile', () => {
    const editor = vscode.window.activeTextEditor;
    if (!editor) {
      vscode.window.showInformationMessage('No active CCL file');
      return;
    }
    const file = editor.document.fileName;
    const terminal = vscode.window.createTerminal('CCL Compile');
    terminal.show(true);
    terminal.sendText(`icn-cli ccl compile "${file}"`);
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
        new vscode.ShellExecution(`icn-cli ccl compile "${file}"`)
      );
      return [task];
    },
    resolveTask: (task) => task
  });

  context.subscriptions.push(compileCmd, provider);
}

function deactivate() {}

module.exports = { activate, deactivate };
