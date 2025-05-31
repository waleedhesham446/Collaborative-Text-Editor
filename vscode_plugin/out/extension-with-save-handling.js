"use strict";
Object.defineProperty(exports, "__esModule", { value: true });
exports.deactivate = exports.activate = void 0;
const vscode = require("vscode");
const WebSocket = require("ws");
let ws = null;
let localChange = false;
function activate(context) {
    let disposable = vscode.commands.registerCommand('extension.connect', () => {
        ws = new WebSocket('ws://localhost:3030/ws');
        ws.onmessage = (event) => {
            const changeEvent = JSON.parse(event.data.toString());
            console.log('Received change event:', changeEvent);
            const editor = vscode.window.activeTextEditor;
            if (editor) {
                if (changeEvent.change_type === 'save') {
                    // save the file changes
                    editor.document.save().then(() => {
                        localChange = false;
                    });
                }
                else if (changeEvent.change_type === 'change' && changeEvent.change) {
                    const change = changeEvent.change;
                    localChange = true;
                    const startPos = editor.document.positionAt(change.start);
                    const endPos = editor.document.positionAt(change.end);
                    const edit = new vscode.WorkspaceEdit();
                    if (change.text === '')
                        edit.delete(editor.document.uri, new vscode.Range(startPos, endPos));
                    else
                        edit.insert(editor.document.uri, startPos, change.text);
                    vscode.workspace.applyEdit(edit).then(() => {
                        localChange = false;
                    });
                }
            }
        };
        vscode.workspace.onDidChangeTextDocument((event) => {
            if (localChange)
                return;
            const change = event.contentChanges[0];
            const start = event.document.offsetAt(change.range.start);
            const end = event.document.offsetAt(change.range.end);
            const message = JSON.stringify({
                change_type: 'change',
                change: { text: change.text, start, end }
            });
            if (ws && ws.readyState === WebSocket.OPEN) {
                ws.send(message);
            }
        });
        vscode.workspace.onWillSaveTextDocument((event) => {
            if (ws && ws.readyState === WebSocket.OPEN) {
                const message = JSON.stringify({ change_type: 'save' });
                if (ws && ws.readyState === WebSocket.OPEN) {
                    ws.send(message);
                }
            }
        });
        vscode.window.showInformationMessage('Connected to Rust Editor!');
    });
    context.subscriptions.push(disposable);
}
exports.activate = activate;
function deactivate() {
    if (ws) {
        ws.close();
    }
}
exports.deactivate = deactivate;
//# sourceMappingURL=extension-with-save-handling.js.map