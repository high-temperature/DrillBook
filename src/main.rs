mod models;
mod loader;
mod quiz;

use loader::load_questions;
use quiz::run_quiz;

fn main() {
    let path = "data/questions.json";

    match load_questions(path) {
        Ok(questions) => {
            run_quiz(&questions);
        }
        Err(e) => {
            eprintln!("問題データの読み込みに失敗しました: {}", e);
        }
    }
}