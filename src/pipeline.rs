use std::path::PathBuf;

#[derive(Debug, Clone)]
pub struct PipelineConfig {
    pub years: Vec<u32>,
    pub subjects: Vec<String>,
    pub output_path: PathBuf,
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

pub fn run_ingest(config: &PipelineConfig) -> Result<PipelineSummary, String> {
    if config.years.is_empty() {
        return Err("対象年度が指定されていません。".to_string());
    }
    if config.subjects.is_empty() {
        return Err("対象科目が指定されていません。".to_string());
    }

    println!("=== 問題集JSON生成パイプライン ===");
    println!("対象年度: {:?}", config.years);
    println!("対象科目: {:?}", config.subjects);
    println!("出力先: {}", config.output_path.display());
    println!("force: {}, dry_run: {}", config.force, config.dry_run);

    if config.dry_run {
        println!("dry-run のため処理は実行しません。");
        return Ok(PipelineSummary {
            downloaded_files: 0,
            ocr_outputs: 0,
            parsed_questions: 0,
            failed_targets: 0,
        });
    }

    println!("現在はパイプライン骨組みのみ実装済みです。次タスクで各段階を接続します。");

    Ok(PipelineSummary {
        downloaded_files: 0,
        ocr_outputs: 0,
        parsed_questions: 0,
        failed_targets: 0,
    })
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn run_ingest_requires_years() {
        let config = PipelineConfig {
            years: vec![],
            subjects: vec!["財務・会計".to_string()],
            output_path: PathBuf::from("data/questions.json"),
            force: false,
            dry_run: true,
        };

        let result = run_ingest(&config);
        assert!(result.is_err());
    }

    #[test]
    fn run_ingest_requires_subjects() {
        let config = PipelineConfig {
            years: vec![2025],
            subjects: vec![],
            output_path: PathBuf::from("data/questions.json"),
            force: false,
            dry_run: true,
        };

        let result = run_ingest(&config);
        assert!(result.is_err());
    }

    #[test]
    fn run_ingest_dry_run_succeeds() {
        let config = PipelineConfig {
            years: vec![2025],
            subjects: vec!["財務・会計".to_string()],
            output_path: PathBuf::from("data/questions.json"),
            force: false,
            dry_run: true,
        };

        let result = run_ingest(&config).expect("dry-run should succeed");
        assert_eq!(result.downloaded_files, 0);
        assert_eq!(result.ocr_outputs, 0);
        assert_eq!(result.parsed_questions, 0);
        assert_eq!(result.failed_targets, 0);
    }
}
