use std::fs;
use crate::models::Question;

pub fn load_questions(path: &str) -> Result<Vec<Question>, Box<dyn std::error::Error>> {
    let content = fs::read_to_string(path)?;
    let questions: Vec<Question> = serde_json::from_str(&content)?;
    Ok(questions)
}