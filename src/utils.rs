// Utility functions
use std::time::{SystemTime, UNIX_EPOCH};
use std::error::Error;

/// Parse an age string that can be either:
/// - A humantime duration (e.g., "1 week", "2 years")
/// - A date string (e.g., "2024-01-15", "2024-01-15T10:30:00")
/// 
/// Returns the Unix timestamp (seconds since epoch) corresponding to the parsed time.
pub fn parse_age_to_timestamp(age_str: &str) -> Result<f64, Box<dyn Error>> {
    // First try parsing as a humantime duration
    if let Ok(duration) = humantime::parse_duration(age_str) {
        // Duration represents "how long ago", so subtract from now
        let now = SystemTime::now()
            .duration_since(UNIX_EPOCH)?
            .as_secs_f64();
        return Ok(now - duration.as_secs_f64());
    }
    
    // Try parsing as a date using chrono
    use chrono::{DateTime, NaiveDate, NaiveDateTime};
    
    // Try parsing as RFC3339/ISO8601 with timezone
    if let Ok(dt) = DateTime::parse_from_rfc3339(age_str) {
        return Ok(dt.timestamp() as f64);
    }
    
    // Try parsing as naive datetime (without timezone)
    if let Ok(dt) = NaiveDateTime::parse_from_str(age_str, "%Y-%m-%d %H:%M:%S") {
        return Ok(dt.and_utc().timestamp() as f64);
    }
    
    // Try parsing as just a date (assume start of day UTC)
    if let Ok(date) = NaiveDate::parse_from_str(age_str, "%Y-%m-%d") {
        let dt = date.and_hms_opt(0, 0, 0).ok_or("Invalid time")?;
        return Ok(dt.and_utc().timestamp() as f64);
    }
    
    Err(format!("Could not parse '{}' as either a duration (e.g., '1 week') or a date (e.g., '2024-01-15')", age_str).into())
}

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
