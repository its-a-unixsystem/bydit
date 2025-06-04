// Utility functions

pub fn escape_csv_field(field: &str) -> String {
    field
        .replace("\r\n", "\\n") // Normalize all common line endings to \n
        .replace("\n", "\\n")   // Escape newline characters
        .replace("\r", "\\n")   // Should be covered by previous, but just in case
        .replace("\"", "\"\"") // Escape double quotes for CSV
}
