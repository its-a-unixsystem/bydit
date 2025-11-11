// Utility functions

pub fn escape_csv_field(field: &str) -> String {
    field
        .replace("\r\n", "\\n") // Normalize all common line endings to \n
        .replace("\n", "\\n")   // Escape newline characters
        .replace("\r", "\\n")   // Should be covered by previous, but just in case
        .replace("\"", "\"\"") // Escape double quotes for CSV
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_escape_empty_string() {
        assert_eq!(escape_csv_field(""), "");
    }

    #[test]
    fn test_escape_no_special_chars() {
        assert_eq!(escape_csv_field("hello world"), "hello world");
    }

    #[test]
    fn test_escape_double_quotes() {
        assert_eq!(escape_csv_field("hello \"world\""), "hello \"\"world\"\"");
    }

    #[test]
    fn test_escape_newline_lf() {
        assert_eq!(escape_csv_field("hello\nworld"), "hello\\nworld");
    }

    #[test]
    fn test_escape_newline_cr() {
        assert_eq!(escape_csv_field("hello\rworld"), "hello\\nworld");
    }

    #[test]
    fn test_escape_newline_crlf() {
        assert_eq!(escape_csv_field("hello\r\nworld"), "hello\\nworld");
    }

    #[test]
    fn test_escape_mixed_newlines() {
        assert_eq!(escape_csv_field("line1\nline2\r\nline3\rline4"), "line1\\nline2\\nline3\\nline4");
    }

    #[test]
    fn test_escape_mixed_special_chars() {
        assert_eq!(
            escape_csv_field("field with \"quotes\" and\nnew line\r\nand another\rCR"),
            "field with \"\"quotes\"\" and\\nnew line\\nand another\\nCR"
        );
    }

    #[test]
    fn test_escape_quotes_and_newlines() {
        assert_eq!(
            escape_csv_field("\"first line\nsecond \"line\"\r\nthird\""),
            "\"\"first line\\nsecond \"\"line\"\"\\nthird\"\""
        );
    }
}
