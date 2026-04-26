mod downloader;
mod exporter;
mod loader;
mod models;
mod parser;
mod pipeline;
mod quiz;

use std::io::{self, Write};
use std::path::PathBuf;

use loader::load_questions;
use pipeline::{PipelineConfig, run_ingest};
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
    println!("2. 過去問を取得して問題集JSONを生成");
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
    let config = PipelineConfig {
        years: vec![2025],
        subjects: vec!["財務・会計".to_string()],
        output_path: PathBuf::from(QUESTIONS_PATH),
        raw_download_dir: PathBuf::from("data/raw"),
        ocr_text_dir: PathBuf::from("data/ocr"),
        source_url_template: Some("https://example.com/{year}/{subject}.pdf".to_string()),
        force: false,
        dry_run: true,
    };

    match run_ingest(&config) {
        Ok(summary) => {
            println!("パイプライン完了サマリー:");
            println!("  ダウンロード件数: {}", summary.downloaded_files);
            println!("  OCR出力件数: {}", summary.ocr_outputs);
            println!("  パース済み問題数: {}", summary.parsed_questions);
            println!("  失敗件数: {}", summary.failed_targets);
        }
        Err(e) => {
            eprintln!("パイプライン実行に失敗しました: {}", e);
        }
    }
}

fn main() {
    match prompt_mode() {
        Some(AppMode::Quiz) => run_quiz_mode(),
        Some(AppMode::BuildDataset) => run_dataset_builder_mode(),
        None => println!("終了します。"),
    }
}
