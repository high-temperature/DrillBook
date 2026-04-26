use std::fs;
use std::path::Path;

use crate::models::Question;

pub fn export_questions(
    path: &str,
    questions: &[Question],
) -> Result<(), Box<dyn std::error::Error>> {
    let output = Path::new(path);
    if let Some(parent) = output.parent() {
        fs::create_dir_all(parent)?;
    }

    let content = serde_json::to_string_pretty(questions)?;
    fs::write(output, content)?;

    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn exports_json_file() {
        let path = std::env::temp_dir().join("drillbook_export_questions_test.json");

        let questions = vec![Question {
            id: "2025_test_01".to_string(),
            year: 2025,
            subject: "テスト".to_string(),
            question_text: "問題".to_string(),
            choices: vec!["A".to_string(), "B".to_string()],
            answer_index: 0,
            explanation: "解説".to_string(),
        }];

        export_questions(path.to_str().expect("utf8 path"), &questions)
            .expect("export should succeed");
        let output = fs::read_to_string(&path).expect("should read output");
        assert!(output.contains("2025_test_01"));

        let _ = fs::remove_file(path);
    }
}
