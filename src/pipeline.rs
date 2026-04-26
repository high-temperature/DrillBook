use std::fs;
use std::path::PathBuf;

use crate::downloader::{DownloadTarget, download_targets};
use crate::exporter::export_questions;
use crate::models::Question;
use crate::parser::parse_questions;

#[derive(Debug, Clone)]
pub struct PipelineConfig {
    pub years: Vec<u32>,
    pub subjects: Vec<String>,
    pub output_path: PathBuf,
    pub raw_download_dir: PathBuf,
    pub ocr_text_dir: PathBuf,
    pub source_url_template: Option<String>,
    pub force: bool,
    pub dry_run: bool,
}

#[derive(Debug, Clone)]
pub struct PipelineSummary {
    pub downloaded_files: usize,
    pub ocr_outputs: usize,
    pub parsed_questions: usize,
    pub failed_targets: usize,
}

#[derive(Debug)]
struct ParseOcrResult {
    questions: Vec<Question>,
    file_count: usize,
}

pub fn run_ingest(config: &PipelineConfig) -> Result<PipelineSummary, String> {
    validate_config(config)?;

    println!("=== 問題集JSON生成パイプライン ===");
    println!("対象年度: {:?}", config.years);
    println!("対象科目: {:?}", config.subjects);
    println!("出力先: {}", config.output_path.display());
    println!("raw保存先: {}", config.raw_download_dir.display());
    println!("ocr入力先: {}", config.ocr_text_dir.display());
    println!("force: {}, dry_run: {}", config.force, config.dry_run);

    let targets = build_download_targets(config)?;

    if config.dry_run {
        println!(
            "dry-run のため処理は実行しません。ダウンロード対象件数: {}",
            targets.len()
        );
        return Ok(PipelineSummary {
            downloaded_files: 0,
            ocr_outputs: 0,
            parsed_questions: 0,
            failed_targets: 0,
        });
    }

    let downloaded = download_targets(&targets, config.force)
        .map_err(|e| format!("ダウンロード処理に失敗しました: {}", e))?;

    let parsed = parse_ocr_directory(config)?;
    export_questions(
        config
            .output_path
            .to_str()
            .ok_or_else(|| "output_path をUTF-8文字列に変換できませんでした。".to_string())?,
        &parsed.questions,
    )
    .map_err(|e| format!("JSON出力に失敗しました: {}", e))?;

    Ok(PipelineSummary {
        downloaded_files: downloaded.len(),
        ocr_outputs: parsed.file_count,
        parsed_questions: parsed.questions.len(),
        failed_targets: 0,
    })
}

fn validate_config(config: &PipelineConfig) -> Result<(), String> {
    if config.years.is_empty() {
        return Err("対象年度が指定されていません。".to_string());
    }
    if config.subjects.is_empty() {
        return Err("対象科目が指定されていません。".to_string());
    }
    if config.source_url_template.is_none() {
        return Err(
            "source_url_template が未設定です（例: https://example.com/{year}/{subject}.pdf）。"
                .to_string(),
        );
    }

    Ok(())
}

fn build_download_targets(config: &PipelineConfig) -> Result<Vec<DownloadTarget>, String> {
    let template = config
        .source_url_template
        .as_deref()
        .ok_or_else(|| "source_url_template が未設定です。".to_string())?;

    let mut targets = Vec::new();

    for year in &config.years {
        for subject in &config.subjects {
            let slug = slugify(subject);
            let file_name = format!("{}_{}.pdf", year, slug);
            let output_path = config.raw_download_dir.join(file_name);
            let url = template
                .replace("{year}", &year.to_string())
                .replace("{subject}", &slug);

            targets.push(DownloadTarget {
                year: *year,
                subject: subject.clone(),
                url,
                output_path,
            });
        }
    }

    Ok(targets)
}

fn parse_ocr_directory(config: &PipelineConfig) -> Result<ParseOcrResult, String> {
    if !config.ocr_text_dir.exists() {
        return Err(format!(
            "OCR入力ディレクトリが存在しません: {}",
            config.ocr_text_dir.display()
        ));
    }

    let entries = fs::read_dir(&config.ocr_text_dir)
        .map_err(|e| format!("OCR入力ディレクトリを読めません: {}", e))?;

    let mut all_questions = Vec::new();
    let mut file_count = 0usize;

    for entry in entries {
        let entry = entry.map_err(|e| format!("OCR入力の列挙に失敗しました: {}", e))?;
        let path = entry.path();

        if path.extension().and_then(|s| s.to_str()) != Some("txt") {
            continue;
        }

        file_count += 1;

        let stem = path
            .file_stem()
            .and_then(|s| s.to_str())
            .ok_or_else(|| format!("ファイル名を解釈できません: {}", path.display()))?;

        let (year, subject) = parse_stem(stem)?;
        let text = fs::read_to_string(&path)
            .map_err(|e| format!("OCRテキストを読めません: {} ({})", path.display(), e))?;

        let mut parsed = parse_questions(&text, year, &subject)
            .map_err(|e| format!("OCRテキストのパースに失敗: {} ({})", path.display(), e))?;
        all_questions.append(&mut parsed);
    }

    if file_count == 0 {
        return Err("OCRテキスト(.txt)が見つかりませんでした。".to_string());
    }

    if all_questions.is_empty() {
        return Err("OCRテキストから問題を1件も抽出できませんでした。".to_string());
    }

    Ok(ParseOcrResult {
        questions: all_questions,
        file_count,
    })
}

fn parse_stem(stem: &str) -> Result<(u32, String), String> {
    let mut parts = stem.splitn(2, '_');
    let year = parts
        .next()
        .ok_or_else(|| format!("年度が不明なファイル名です: {}", stem))?
        .parse::<u32>()
        .map_err(|_| format!("年度の解釈に失敗しました: {}", stem))?;

    let subject = parts
        .next()
        .ok_or_else(|| format!("科目が不明なファイル名です: {}", stem))?
        .replace('_', " ");

    Ok((year, subject))
}

fn slugify(subject: &str) -> String {
    let trimmed = subject.trim().to_lowercase();
    if trimmed.is_empty() {
        return "unknown".to_string();
    }

    let ascii_like = trimmed
        .chars()
        .map(|c| match c {
            'a'..='z' | '0'..='9' => c,
            'ァ'..='ヶ' | 'ぁ'..='ゖ' | '一'..='龥' => '_',
            _ => '_',
        })
        .collect::<String>();

    let compact = ascii_like
        .split('_')
        .filter(|s| !s.is_empty())
        .collect::<Vec<_>>()
        .join("_");

    if compact.is_empty() {
        "unknown".to_string()
    } else {
        compact
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn sample_config() -> PipelineConfig {
        PipelineConfig {
            years: vec![2025],
            subjects: vec!["財務・会計".to_string()],
            output_path: PathBuf::from("data/questions.json"),
            raw_download_dir: PathBuf::from("data/raw"),
            ocr_text_dir: std::env::temp_dir().join("drillbook_pipeline_ocr"),
            source_url_template: Some("https://example.com/{year}/{subject}.pdf".to_string()),
            force: false,
            dry_run: true,
        }
    }

    #[test]
    fn run_ingest_requires_years() {
        let mut config = sample_config();
        config.years = vec![];

        let result = run_ingest(&config);
        assert!(result.is_err());
    }

    #[test]
    fn run_ingest_requires_subjects() {
        let mut config = sample_config();
        config.subjects = vec![];

        let result = run_ingest(&config);
        assert!(result.is_err());
    }

    #[test]
    fn run_ingest_requires_template() {
        let mut config = sample_config();
        config.source_url_template = None;

        let result = run_ingest(&config);
        assert!(result.is_err());
    }

    #[test]
    fn run_ingest_dry_run_succeeds() {
        let config = sample_config();

        let result = run_ingest(&config).expect("dry-run should succeed");
        assert_eq!(result.downloaded_files, 0);
        assert_eq!(result.ocr_outputs, 0);
        assert_eq!(result.parsed_questions, 0);
        assert_eq!(result.failed_targets, 0);
    }

    #[test]
    fn build_download_targets_creates_matrix() {
        let mut config = sample_config();
        config.years = vec![2024, 2025];
        config.subjects = vec!["財務・会計".to_string(), "企業経営理論".to_string()];

        let targets = build_download_targets(&config).expect("targets should build");
        assert_eq!(targets.len(), 4);
        assert!(
            targets
                .iter()
                .all(|t| t.url.starts_with("https://example.com/"))
        );
    }

    #[test]
    fn parse_stem_works() {
        let (year, subject) = parse_stem("2025_zaimu_kaikei").expect("should parse");
        assert_eq!(year, 2025);
        assert_eq!(subject, "zaimu kaikei");
    }
}
