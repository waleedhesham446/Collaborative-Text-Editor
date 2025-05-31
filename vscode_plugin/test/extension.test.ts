import * as assert from 'assert';
import * as vscode from 'vscode';
import * as sinon from 'sinon';
import * as WebSocket from 'ws';
import { Server } from 'http';

// Import the extension module
import * as extension from '../src/extension';

suite('Extension Test Suite', () => {
    let context: vscode.ExtensionContext;
    let mockWebSocketServer: WebSocket.Server;
    let server: Server;

    suiteSetup(async () => {
        // Setup mock WebSocket server for testing
        server = new Server();
        mockWebSocketServer = new WebSocket.Server({ server });
        
        server.listen(3030, () => {
            console.log('Test WebSocket server started on port 3030');
        });
    });

    suiteTeardown(() => {
        if (mockWebSocketServer) {
            mockWebSocketServer.close();
        }
        if (server) {
            server.close();
        }
    });

    setup(() => {
        // Create mock extension context
        context = {
            subscriptions: [],
            workspaceState: {
                get: sinon.stub(),
                update: sinon.stub(),
                keys: sinon.stub().returns([])
            },
            globalState: {
                get: sinon.stub(),
                update: sinon.stub(),
                setKeysForSync: sinon.stub(),
                keys: sinon.stub().returns([])
            },
            secrets: {
                get: sinon.stub(),
                store: sinon.stub(),
                delete: sinon.stub(),
                onDidChange: sinon.stub()
            },
            extensionUri: vscode.Uri.file('/mock/path'),
            extensionPath: '/mock/path',
            environmentVariableCollection: {
                persistent: true,
                replace: sinon.stub(),
                append: sinon.stub(),
                prepend: sinon.stub(),
                get: sinon.stub(),
                forEach: sinon.stub(),
                delete: sinon.stub(),
                clear: sinon.stub(),
                getScopes: sinon.stub().returns([])
            },
            extensionMode: vscode.ExtensionMode.Test,
            globalStorageUri: vscode.Uri.file('/mock/global'),
            logUri: vscode.Uri.file('/mock/log'),
            storageUri: vscode.Uri.file('/mock/storage'),
            asAbsolutePath: sinon.stub().returns('/mock/absolute/path'),
            logPath: '/mock/log/path'
        } as unknown as vscode.ExtensionContext;
    });

    test('Extension activation', async () => {
        // Test that the extension activates without throwing
        assert.doesNotThrow(() => {
            extension.activate(context);
        });

        // Verify that a command was registered
        assert.strictEqual(context.subscriptions.length, 1);
    });

    test('Extension deactivation', () => {
        // Test that deactivation doesn't throw
        assert.doesNotThrow(() => {
            extension.deactivate();
        });
    });

    test('WebSocket connection establishment', (done) => {
        // Setup mock server to accept connections
        mockWebSocketServer.on('connection', (ws) => {
            assert.ok(ws, 'WebSocket connection established');
            ws.close();
            done();
        });

        // Simulate the connect command
        const ws = new WebSocket('ws://localhost:3030/ws');
        ws.on('open', () => {
            ws.close();
        });
    });

    test('Message serialization format', () => {
        const testChange = {
            text: 'Hello, World!',
            start: 0,
            end: 5
        };

        const serialized = JSON.stringify(testChange);
        const deserialized = JSON.parse(serialized);

        assert.strictEqual(deserialized.text, testChange.text);
        assert.strictEqual(deserialized.start, testChange.start);
        assert.strictEqual(deserialized.end, testChange.end);
    });

    test('Text change message handling', (done) => {
        const testMessage = {
            text: 'test text',
            start: 10,
            end: 15
        };

        mockWebSocketServer.on('connection', (ws) => {
            // Send test message to client
            ws.send(JSON.stringify(testMessage));
        });

        const client = new WebSocket('ws://localhost:3030/ws');
        client.on('message', (data) => {
            const received = JSON.parse(data.toString());
            assert.strictEqual(received.text, testMessage.text);
            assert.strictEqual(received.start, testMessage.start);
            assert.strictEqual(received.end, testMessage.end);
            client.close();
            done();
        });
    });

    test('Empty text change handling', (done) => {
        const deleteMessage = {
            text: '',
            start: 5,
            end: 10
        };

        mockWebSocketServer.on('connection', (ws) => {
            ws.send(JSON.stringify(deleteMessage));
        });

        const client = new WebSocket('ws://localhost:3030/ws');
        client.on('message', (data) => {
            const received = JSON.parse(data.toString());
            assert.strictEqual(received.text, '');
            assert.strictEqual(received.start, deleteMessage.start);
            assert.strictEqual(received.end, deleteMessage.end);
            client.close();
            done();
        });
    });

    test('Newline handling', (done) => {
        const newlineMessage = {
            text: '\n',
            start: 20,
            end: 20
        };

        mockWebSocketServer.on('connection', (ws) => {
            ws.send(JSON.stringify(newlineMessage));
        });

        const client = new WebSocket('ws://localhost:3030/ws');
        client.on('message', (data) => {
            const received = JSON.parse(data.toString());
            assert.strictEqual(received.text, '\n');
            assert.strictEqual(received.start, newlineMessage.start);
            assert.strictEqual(received.end, newlineMessage.end);
            client.close();
            done();
        });
    });

    test('WebSocket connection error handling', (done) => {
        // Try to connect to non-existent server
        const client = new WebSocket('ws://localhost:9999/ws');
        
        client.on('error', (error) => {
            assert.ok(error, 'Connection error properly handled');
            done();
        });

        client.on('open', () => {
            assert.fail('Connection should not be established');
        });
        client.on('close', () => {
            assert.fail('Connection should not close successfully');
        });
        client.on('message', () => {
            assert.fail('No messages should be received on error');
        });
    });
});