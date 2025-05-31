# Collaborative Text Editing System

A real-time collaborative text editing system consisting of a Rust-based terminal text editor (master) and a Visual Studio Code extension that synchronizes via WebSocket connections.

## 🎯 Project Overview

This project implements a collaborative text editing solution where:
- A **Rust terminal editor** serves as the master editor with file access
- A **VS Code extension** connects as a remote client
- Both editors synchronize in real-time via WebSocket communication
- Multiple developers can edit the same file simultaneously on localhost

## 🏗️ Architecture

### System Components

```
┌─────────────────┐    WebSocket         ┌─────────────────┐
│   Rust Editor      │◄────────────────►│  VS Code Plugin    │
│   (Master)         │    localhost:3030    │   (Client)         │
│                    │                      │                    │
│ • File I/O         │                      │ • Text Changes     │
│ • WebSocket        │                      │ • Change Events    │
│   Server           │                      │                    │
│ • Terminal UI      │                      │                    │
└─────────────────┘                      └─────────────────┘
```

### Key Design Decisions

1. **Master-Client Architecture**: The Rust editor maintains file system access and acts as the source of truth
2. **WebSocket Communication**: Real-time bidirectional communication using JSON messages
3. **Event-Driven Synchronization**: Changes are broadcast immediately to maintain consistency
4. **Operational Transformation**: Basic conflict resolution through position-based operations

### Data Flow

1. User types in either editor
2. Change event is captured and serialized to JSON
3. Change is sent via WebSocket to the other editor
4. Receiving editor applies the change and updates display
5. Both editors remain synchronized

## 🚀 Quick Start

### Prerequisites

- Rust (latest stable version)
- Node.js and npm
- Visual Studio Code
- Unix-like system (Linux/macOS) for terminal operations

### Running the Rust Editor

1. Clone the repository
2. Navigate to the Rust editor directory
3. Build and run:

```bash
cargo build --release
cargo run [filename]
```

**Controls:**
- `Ctrl+S`: Save file
- `Ctrl+Q`: Quit (press twice if unsaved changes)
- `Ctrl+H`: Show help
- Arrow keys: Navigate
- Enter: New line
- Backspace/Delete: Remove characters

### Installing the VS Code Extension

1. Navigate to the extension directory
2. Install dependencies:

```bash
npm install
```

3. Package the extension:

```bash
npm install -g vsce
vsce package
```

4. Install in VS Code:

```bash
code --install-extension collaborative-editor-0.0.1.vsix
```

### Usage

1. **Start the Rust editor** with a file:
   ```bash
   cargo run test.txt
   ```

2. **Open the same file in VS Code**

3. **Connect the VS Code extension**:
   - Open Command Palette (`Ctrl+Shift+P`)
   - Run: `Connect to Rust Editor`

4. **Start collaborating**: Both editors will now sync changes in real-time!

## 📡 WebSocket API

The system uses JSON messages over WebSocket for synchronization:

### Message Format

```typescript
interface TextChange {
  text: string;    // Text to insert (empty string for deletion)
  start: number;   // Start position in document
  end: number;     // End position in document
}
```

### Example Messages

**Text Insertion:**
```json
{
  "text": "Hello",
  "start": 10,
  "end": 10
}
```

**Text Deletion:**
```json
{
  "text": "",
  "start": 5,
  "end": 10
}
```

**Newline Insertion:**
```json
{
  "text": "\n",
  "start": 15,
  "end": 15
}
```

## 🔧 Configuration

### Rust Editor Configuration

The editor supports basic configuration through command-line arguments:

```bash
cargo run [filename]  # Open specific file
cargo run            # Start with empty document
```

### VS Code Extension Configuration

The extension connects to `ws://localhost:3030/ws` by default. This can be modified in the `extension.ts` file.

## 🏃‍♂️ Performance Considerations

- **Memory Usage**: The Rust editor loads the entire file into memory for fast access
- **Network Latency**: Changes are sent immediately, optimizing for responsiveness
- **Concurrent Edits**: Basic conflict resolution handles simultaneous edits
- **File Size**: Suitable for typical source code files (< 1MB)

## 🐛 Known Limitations

1. **Platform Dependency**: Terminal operations require Unix-like systems
2. **Localhost Only**: No network security or authentication
3. **Single File**: Only one file can be synchronized at a time
4. **Basic Conflict Resolution**: May not handle complex simultaneous edits optimally
5. **No Undo/Redo Sync**: Undo operations are not synchronized between editors

## 🛠️ Development

### Building from Source

```bash
# Rust editor
cd rust-editor
cargo build

# VS Code extension
cd vscode-extension
npm install
npm run compile
```

### Project Structure

```
├── src/
│   └── main.rs              # Rust editor implementation
├── vscode-extension/
│   ├── src/
│   │   └── extension.ts     # VS Code extension
│   ├── package.json
│   └── tsconfig.json
├── tests/                   # Test files
├── Cargo.toml              # Rust dependencies
└── README.md
```

### Dependencies

**Rust:**
- `warp`: WebSocket server
- `tokio`: Async runtime
- `serde`: JSON serialization
- `parking_lot`: Thread-safe primitives
- `simplelog`: Logging

**Node.js:**
- `vscode`: VS Code API
- `ws`: WebSocket client

## 🧪 Testing

See the `tests/` directory for comprehensive unit and integration tests.

Run tests with:
```bash
cargo test                    # Rust tests
cd vscode-extension && npm test  # Extension tests
```

## 📝 License

This project is created for educational purposes as part of a software engineering assessment.

## 🤝 Contributing

This is an assessment project. For production use, consider implementing:
- Authentication and authorization
- Network security (WSS, encryption)
- More sophisticated conflict resolution
- Support for multiple files
- Persistent connection handling
- Better error recovery

---

**Note**: This system is designed for demonstration purposes on localhost. Additional security measures would be required for production deployment.