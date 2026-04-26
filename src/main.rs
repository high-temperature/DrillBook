mod loader;
mod models;
mod quiz;

use std::io::{self, Write};

use loader::load_questions;
use quiz::run_quiz;

const QUESTIONS_PATH: &str = "data/questions.json";

#[derive(Debug, Clone, Copy)]
enum AppMode {
    Quiz,
    BuildDataset,
}

fn prompt_mode() -> Option<AppMode> {
    println!("==============================");
    println!("DrillBook 起動メニュー");
    println!("1. 既存JSONでクイズ開始");
    println!("2. 過去問を取得して問題集JSONを生成（準備中）");
    println!("0. 終了");
    println!("==============================");

    print!("実行する番号を入力してください: ");
    io::stdout().flush().ok()?;

    let mut input = String::new();
    io::stdin().read_line(&mut input).ok()?;

    match input.trim() {
        "1" => Some(AppMode::Quiz),
        "2" => Some(AppMode::BuildDataset),
        "0" => None,
        _ => {
            println!("入力が正しくありません。0, 1, 2 のいずれかを入力してください。");
            None
        }
    }
}

fn run_quiz_mode() {
    match load_questions(QUESTIONS_PATH) {
        Ok(questions) => {
            run_quiz(&questions);
        }
        Err(e) => {
            eprintln!("問題データの読み込みに失敗しました: {}", e);
        }
    }
}

fn run_dataset_builder_mode() {
    println!("この機能は現在実装中です。次のタスクで追加します。");
}

fn main() {
    match prompt_mode() {
        Some(AppMode::Quiz) => run_quiz_mode(),
        Some(AppMode::BuildDataset) => run_dataset_builder_mode(),
        None => println!("終了します。"),
    }
}
