use crate::error::ArcError;
use crate::parser::{grammar, lexer};

use super::{read_dot_file, ParseArgs};

/// Parse a DOT file and print its raw AST as JSON.
///
/// # Errors
///
/// Returns an error if the file cannot be read, parsed, or contains trailing content.
pub fn parse_command(args: &ParseArgs) -> anyhow::Result<()> {
    let source = read_dot_file(&args.workflow)?;
    let stripped = lexer::strip_comments(&source);
    let (rest, ast) = grammar::parse_dot_graph(&stripped)
        .map_err(|e| ArcError::Parse(format!("grammar error: {e}")))?;

    let remaining = rest.trim();
    if !remaining.is_empty() {
        return Err(ArcError::Parse(format!(
            "unexpected trailing content: {:?}",
            &remaining[..remaining.len().min(50)]
        ))
        .into());
    }

    println!("{}", serde_json::to_string_pretty(&ast)?);
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::parser::ast::DotGraph;
    use std::io::Write;
    use std::path::PathBuf;

    #[test]
    fn parse_command_outputs_json_ast() {
        let mut tmp = tempfile::NamedTempFile::new().unwrap();
        write!(
            tmp,
            r#"digraph Hello {{
    start [shape=Mdiamond]
    exit [shape=Msquare]
    start -> exit
}}"#
        )
        .unwrap();

        let args = ParseArgs {
            workflow: tmp.path().to_path_buf(),
        };
        let result = parse_command(&args);
        assert!(result.is_ok(), "expected Ok but got: {result:?}");

        // Re-run and capture stdout by reading the file through the parser directly
        let source = std::fs::read_to_string(tmp.path()).unwrap();
        let stripped = lexer::strip_comments(&source);
        let (_, ast) = grammar::parse_dot_graph(&stripped).unwrap();
        let json = serde_json::to_string_pretty(&ast).unwrap();
        let deserialized: DotGraph = serde_json::from_str(&json).unwrap();

        assert_eq!(deserialized.name, "Hello");
        assert_eq!(deserialized.statements.len(), 3);
    }

    #[test]
    fn parse_command_rejects_invalid_dot() {
        let mut tmp = tempfile::NamedTempFile::new().unwrap();
        write!(tmp, "not a valid dot file").unwrap();

        let args = ParseArgs {
            workflow: tmp.path().to_path_buf(),
        };
        let result = parse_command(&args);
        assert!(result.is_err(), "expected Err for invalid syntax");
    }

    #[test]
    fn parse_command_rejects_missing_file() {
        let args = ParseArgs {
            workflow: PathBuf::from("/tmp/nonexistent_parse_test_12345.dot"),
        };
        let result = parse_command(&args);
        assert!(result.is_err(), "expected Err for missing file");
    }
}
