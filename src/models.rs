use serde::{Deserialize, Serialize};

#[derive(Debug, Serialize, Deserialize)]
pub struct Question {
    pub id: String,
    pub year: u32,
    pub subject: String,
    pub question_text: String,
    pub choices: Vec<String>,
    pub answer_index: usize,
    pub explanation: String,
}