import * as vscode from 'vscode';
import * as WebSocket from 'ws';

let ws: WebSocket | null = null;
let localChange = false;

export function activate(context: vscode.ExtensionContext) {
  let disposable = vscode.commands.registerCommand('extension.connect', () => {
    ws = new WebSocket('ws://localhost:3030/ws');

    ws.onmessage = (event) => {
      const change = JSON.parse(event.data.toString());
      
      const editor = vscode.window.activeTextEditor;
      if (editor) {
        localChange = true;
        const startPos = editor.document.positionAt(change.start);
        const endPos = editor.document.positionAt(change.end);
        const edit = new vscode.WorkspaceEdit();
        if (change.text === '') {
          const range = new vscode.Range(startPos, endPos);
          edit.delete(editor.document.uri, range);
        } else {
          edit.insert(editor.document.uri, startPos, change.text);

        }
        
        vscode.workspace.applyEdit(edit).then(() => {
          localChange = false;
        });
      }
    };

    vscode.workspace.onDidChangeTextDocument((event) => {
      if (localChange) return;

      const change = event.contentChanges[0];
      const start = event.document.offsetAt(change.range.start);
      const end = event.document.offsetAt(change.range.end);
      const message = JSON.stringify({ text: change.text, start, end });
      if (ws && ws.readyState === WebSocket.OPEN) {
        ws.send(message);
      }
    });

    vscode.window.showInformationMessage('Connected to Rust Editor!');
  });

  context.subscriptions.push(disposable);
}

export function deactivate() {
  if (ws) {
    ws.close();
  }
}
