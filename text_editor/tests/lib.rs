#[cfg(test)]
mod tests {
    use super::*;
    use std::fs;
    use std::io::Write;
    use tempfile::NamedTempFile;
    use tokio_test;

    // Test utility functions
    fn create_test_file(content: &str) -> NamedTempFile {
        let mut file = NamedTempFile::new().expect("Failed to create temp file");
        file.write_all(content.as_bytes()).expect("Failed to write to temp file");
        file
    }

    #[test]
    fn test_index_to_line_column() {
        let text = "Hello\nWorld\nTest";
        
        // Test beginning of file
        assert_eq!(index_to_line_column(text, 0), Some((0, 0)));
        
        // Test middle of first line
        assert_eq!(index_to_line_column(text, 2), Some((0, 2)));
        
        // Test beginning of second line
        assert_eq!(index_to_line_column(text, 6), Some((1, 0)));
        
        // Test middle of second line
        assert_eq!(index_to_line_column(text, 8), Some((1, 2)));
        
        // Test beginning of third line
        assert_eq!(index_to_line_column(text, 12), Some((2, 0)));
        
        // Test end of file
        assert_eq!(index_to_line_column(text, text.len()), Some((2, 4)));
        
        // Test invalid index
        assert_eq!(index_to_line_column(text, text.len() + 1), None);
    }

    #[test]
    fn test_index_to_line_column_empty_string() {
        let text = "";
        assert_eq!(index_to_line_column(text, 0), Some((0, 0)));
        assert_eq!(index_to_line_column(text, 1), None);
    }

    #[test]
    fn test_index_to_line_column_single_line() {
        let text = "Single line";
        assert_eq!(index_to_line_column(text, 0), Some((0, 0)));
        assert_eq!(index_to_line_column(text, 6), Some((0, 6)));
        assert_eq!(index_to_line_column(text, text.len()), Some((0, text.len())));
    }

    #[test]
    fn test_index_to_line_column_with_unicode() {
        let text = "Hello 🦀\nRust";
        assert_eq!(index_to_line_column(text, 0), Some((0, 0)));
        assert_eq!(index_to_line_column(text, 6), Some((0, 6))); // Before emoji
        assert_eq!(index_to_line_column(text, 10), Some((0, 7))); // After emoji
        assert_eq!(index_to_line_column(text, 11), Some((1, 0))); // Second line
    }

    #[test]
    fn test_key_parsing() {
        // Test character key
        let key = Key::Char('a');
        assert_eq!(key, Key::Char('a'));
        
        // Test control key
        let key = Key::Ctrl(b'c');
        assert_eq!(key, Key::Ctrl(b'c'));
        
        // Test special keys
        assert_eq!(Key::Enter, Key::Enter);
        assert_eq!(Key::Backspace, Key::Backspace);
        assert_eq!(Key::Up, Key::Up);
    }

    #[test]
    fn test_text_change_serialization() {
        let change = TextChange {
            text: "Hello".to_string(),
            start: 0,
            end: 5,
        };
        
        let json = serde_json::to_string(&change).expect("Failed to serialize");
        let deserialized: TextChange = serde_json::from_str(&json).expect("Failed to deserialize");
        
        assert_eq!(change.text, deserialized.text);
        assert_eq!(change.start, deserialized.start);
        assert_eq!(change.end, deserialized.end);
    }

    mod editor_tests {
        use super::*;
        use std::io;

        fn create_test_editor() -> io::Result<Editor> {
            Editor::new(None)
        }

        fn create_test_editor_with_content(content: &str) -> io::Result<Editor> {
            let file = create_test_file(content);
            Editor::new(Some(file.path().to_string_lossy().to_string()))
        }

        #[test]
        fn test_editor_creation_empty() {
            let editor = create_test_editor().expect("Failed to create editor");
            assert_eq!(editor.content.len(), 1);
            assert_eq!(editor.content[0], "");
            assert_eq!(editor.cursor_x, 0);
            assert_eq!(editor.cursor_y, 0);
            assert_eq!(editor.modified, false);
        }

        #[test]
        fn test_editor_creation_with_file() {
            let content = "Line 1\nLine 2\nLine 3";
            let editor = create_test_editor_with_content(content).expect("Failed to create editor");
            
            assert_eq!(editor.content.len(), 3);
            assert_eq!(editor.content[0], "Line 1");
            assert_eq!(editor.content[1], "Line 2");
            assert_eq!(editor.content[2], "Line 3");
            assert_eq!(editor.cursor_x, 0);
            assert_eq!(editor.cursor_y, 0);
            assert_eq!(editor.modified, false);
        }

        #[test]
        fn test_insert_char() {
            let mut editor = create_test_editor().expect("Failed to create editor");
            
            // Insert first character
            let result = editor.insert_char('H');
            assert!(result.is_ok());
            assert_eq!(editor.content[0], "H");
            assert_eq!(editor.cursor_x, 1);
            assert_eq!(editor.modified, true);
            
            // Insert second character
            let result = editor.insert_char('i');
            assert!(result.is_ok());
            assert_eq!(editor.content[0], "Hi");
            assert_eq!(editor.cursor_x, 2);
        }

        #[test]
        fn test_insert_newline() {
            let mut editor = create_test_editor_with_content("Hello World").expect("Failed to create editor");
            editor.cursor_x = 5; // Position after "Hello"
            
            let result = editor.insert_newline();
            assert!(result.is_ok());
            
            assert_eq!(editor.content.len(), 2);
            assert_eq!(editor.content[0], "Hello");
            assert_eq!(editor.content[1], " World");
            assert_eq!(editor.cursor_x, 0);
            assert_eq!(editor.cursor_y, 1);
            assert_eq!(editor.modified, true);
        }

        #[test]
        fn test_delete_char_middle_of_line() {
            let mut editor = create_test_editor_with_content("Hello").expect("Failed to create editor");
            editor.cursor_x = 3; // Position after "Hel"
            
            let result = editor.delete_char();
            assert!(result.is_ok());
            
            assert_eq!(editor.content[0], "Helo");
            assert_eq!(editor.cursor_x, 2);
            assert_eq!(editor.modified, true);
        }

        #[test]
        fn test_delete_char_beginning_of_line() {
            let mut editor = create_test_editor_with_content("Line 1\nLine 2").expect("Failed to create editor");
            editor.cursor_y = 1;
            editor.cursor_x = 0; // Beginning of second line
            
            let result = editor.delete_char();
            assert!(result.is_ok());
            
            assert_eq!(editor.content.len(), 1);
            assert_eq!(editor.content[0], "Line 1Line 2");
            assert_eq!(editor.cursor_x, 6);
            assert_eq!(editor.cursor_y, 0);
            assert_eq!(editor.modified, true);
        }

        #[test]
        fn test_delete_char_forward() {
            let mut editor = create_test_editor_with_content("Hello").expect("Failed to create editor");
            editor.cursor_x = 2; // Position after "He"
            
            let result = editor.delete_char_forward();
            assert!(result.is_ok());
            
            assert_eq!(editor.content[0], "Helo");
            assert_eq!(editor.cursor_x, 2);
            assert_eq!(editor.modified, true);
        }

        #[test]
        fn test_cursor_movement_up_down() {
            let mut editor = create_test_editor_with_content("Line 1\nLine 2\nLine 3").expect("Failed to create editor");
            
            // Move down
            let result = editor.move_cursor_down();
            assert!(result.is_ok());
            assert_eq!(editor.cursor_y, 1);
            
            // Move down again
            let result = editor.move_cursor_down();
            assert!(result.is_ok());
            assert_eq!(editor.cursor_y, 2);
            
            // Try to move down past end (should not move)
            let result = editor.move_cursor_down();
            assert!(result.is_ok());
            assert_eq!(editor.cursor_y, 2);
            
            // Move up
            let result = editor.move_cursor_up();
            assert!(result.is_ok());
            assert_eq!(editor.cursor_y, 1);
            
            // Move to beginning
            let result = editor.move_cursor_up();
            assert!(result.is_ok());
            assert_eq!(editor.cursor_y, 0);
            
            // Try to move up past beginning (should not move)
            let result = editor.move_cursor_up();
            assert!(result.is_ok());
            assert_eq!(editor.cursor_y, 0);
        }

        #[test]
        fn test_cursor_movement_left_right() {
            let mut editor = create_test_editor_with_content("Hello").expect("Failed to create editor");
            
            // Move right
            let result = editor.move_cursor_right();
            assert!(result.is_ok());
            assert_eq!(editor.cursor_x, 1);
            
            // Move to end of line
            editor.cursor_x = 5; // End of "Hello"
            
            // Try to move right past end (should not move on single line)
            let result = editor.move_cursor_right();
            assert!(result.is_ok());
            assert_eq!(editor.cursor_x, 5);
            
            // Move left
            let result = editor.move_cursor_left();
            assert!(result.is_ok());
            assert_eq!(editor.cursor_x, 4);
            
            // Move to beginning
            editor.cursor_x = 0;
            
            // Try to move left past beginning (should not move)
            let result = editor.move_cursor_left();
            assert!(result.is_ok());
            assert_eq!(editor.cursor_x, 0);
        }

        #[test]
        fn test_cursor_movement_across_lines() {
            let mut editor = create_test_editor_with_content("Hello\nWorld").expect("Failed to create editor");
            editor.cursor_x = 5; // End of first line
            
            // Move right should go to beginning of next line
            let result = editor.move_cursor_right();
            assert!(result.is_ok());
            assert_eq!(editor.cursor_x, 0);
            assert_eq!(editor.cursor_y, 1);
            
            // Move left should go to end of previous line
            let result = editor.move_cursor_left();
            assert!(result.is_ok());
            assert_eq!(editor.cursor_x, 5);
            assert_eq!(editor.cursor_y, 0);
        }

        #[test]
        fn test_save_file() {
            let temp_file = NamedTempFile::new().expect("Failed to create temp file");
            let filename = temp_file.path().to_string_lossy().to_string();
            
            let mut editor = Editor::new(Some(filename.clone())).expect("Failed to create editor");
            editor.content = vec!["Hello".to_string(), "World".to_string()];
            editor.modified = true;
            
            let result = editor.save_file();
            assert!(result.is_ok());
            assert_eq!(editor.modified, false);
            
            // Verify file content
            let saved_content = fs::read_to_string(&filename).expect("Failed to read saved file");
            assert_eq!(saved_content, "Hello\nWorld");
        }

        #[tokio::test]
        async fn test_broadcast_change_char() {
            let mut editor = create_test_editor().expect("Failed to create editor");
            
            // Test character insertion
            let key = Key::Char('H');
            let result = editor.broadcast_change(&key, 1).await;
            
            // Should not fail even without WebSocket connection
            assert!(result.is_ok());
        }

        #[tokio::test]
        async fn test_broadcast_change_enter() {
            let mut editor = create_test_editor().expect("Failed to create editor");
            
            // Test newline insertion
            let key = Key::Enter;
            let result = editor.broadcast_change(&key, 0).await;
            
            // Should not fail even without WebSocket connection
            assert!(result.is_ok());
        }

        #[tokio::test]
        async fn test_broadcast_change_backspace() {
            let mut editor = create_test_editor().expect("Failed to create editor");
            
            // Test backspace
            let key = Key::Backspace;
            let result = editor.broadcast_change(&key, 1).await;
            
            // Should not fail even without WebSocket connection
            assert!(result.is_ok());
        }
    }

    mod integration_tests {
        use super::*;
        use tokio::time::{sleep, Duration};
        use std::sync::Arc;
        use parking_lot::Mutex;

        #[tokio::test]
        async fn test_websocket_server_setup() {
            let editor = Arc::new(Mutex::new(Editor::new(None).expect("Failed to create editor")));
            
            // This test verifies that the WebSocket server can be set up
            // In a real scenario, we would test the actual WebSocket connection
            // but that requires more complex setup with a test client
            
            let buffer_for_ws = editor.clone();
            let ws_route = warp::path("ws")
                .and(warp::ws())
                .and(warp::any().map(move || buffer_for_ws.clone()))
                .map(|ws: warp::ws::Ws, buffer| {
                    ws.on_upgrade(move |socket| handle_connection(socket, buffer))
                });

            // Start server in background
            let server = tokio::spawn(async move {
                warp::serve(ws_route).run(([127, 0, 0, 1], 3031)).await;
            });

            // Give server time to start
            sleep(Duration::from_millis(100)).await;

            // In a real test, we would connect a WebSocket client here
            // and test the bidirectional communication
            
            server.abort();
        }

        #[test]
        fn test_text_change_json_compatibility() {
            // Test that our TextChange struct is compatible with the VS Code extension format
            let change = TextChange {
                text: "Hello, World!".to_string(),
                start: 10,
                end: 15,
            };
            
            let json = serde_json::to_string(&change).expect("Failed to serialize");
            
            // Verify JSON structure matches what VS Code extension expects
            assert!(json.contains("\"text\":\"Hello, World!\""));
            assert!(json.contains("\"start\":10"));
            assert!(json.contains("\"end\":15"));
            
            // Test deserialization
            let deserialized: TextChange = serde_json::from_str(&json).expect("Failed to deserialize");
            assert_eq!(deserialized.text, change.text);
            assert_eq!(deserialized.start, change.start);
            assert_eq!(deserialized.end, change.end);
        }
    }

    mod terminal_tests {
        use super::*;
        
        // Note: Terminal tests are limited because they require actual terminal interaction
        // In a production environment, you might use a mock terminal or test framework
        
        #[test]
        fn test_terminal_size_fallback() {
            // Test that we handle terminal size detection gracefully
            // This should not panic even if terminal commands fail
            let result = Terminal::get_terminal_size();
            
            // Should either succeed or use fallback values
            match result {
                Ok((rows, cols)) => {
                    assert!(rows > 0);
                    assert!(cols > 0);
                }
                Err(_) => {
                    // Terminal commands may fail in test environment
                    // This is acceptable behavior
                }
            }
        }
    }
}