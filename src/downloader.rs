use std::fs;
use std::path::{Path, PathBuf};

#[derive(Debug, Clone)]
pub struct DownloadTarget {
    pub year: u32,
    pub subject: String,
    pub url: String,
    pub output_path: PathBuf,
}

pub fn download_targets(
    targets: &[DownloadTarget],
    force: bool,
) -> Result<Vec<PathBuf>, Box<dyn std::error::Error>> {
    let client = reqwest::blocking::Client::builder()
        .user_agent("DrillBook/0.1")
        .build()?;

    let mut downloaded_paths = Vec::with_capacity(targets.len());

    for target in targets {
        validate_target(target)?;

        if target.output_path.exists() && !force {
            downloaded_paths.push(target.output_path.clone());
            continue;
        }

        ensure_parent_dir(&target.output_path)?;

        let response = client.get(&target.url).send()?.error_for_status()?;
        let bytes = response.bytes()?;
        fs::write(&target.output_path, bytes.as_ref())?;

        downloaded_paths.push(target.output_path.clone());
    }

    Ok(downloaded_paths)
}

fn validate_target(target: &DownloadTarget) -> Result<(), Box<dyn std::error::Error>> {
    if target.subject.trim().is_empty() {
        return Err(format!("subject is empty for year {}", target.year).into());
    }
    if target.output_path == Path::new("") {
        return Err(format!("output_path is empty for year {}", target.year).into());
    }

    reqwest::Url::parse(&target.url)?;
    Ok(())
}

fn ensure_parent_dir(path: &Path) -> Result<(), Box<dyn std::error::Error>> {
    if let Some(parent) = path.parent() {
        fs::create_dir_all(parent)?;
    }
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn validates_invalid_url() {
        let target = DownloadTarget {
            year: 2025,
            subject: "財務・会計".to_string(),
            url: "not-a-valid-url".to_string(),
            output_path: PathBuf::from("data/raw/2025_zaimu.pdf"),
        };

        let result = validate_target(&target);
        assert!(result.is_err());
    }

    #[test]
    fn skips_existing_file_when_not_force() {
        let output_path = std::env::temp_dir().join("drillbook_skip_existing_test.bin");
        fs::write(&output_path, b"already-exists").expect("failed to setup temp file");

        let target = DownloadTarget {
            year: 2025,
            subject: "財務・会計".to_string(),
            url: "https://example.com/2025/zaimu.pdf".to_string(),
            output_path: output_path.clone(),
        };

        let result = download_targets(&[target], false).expect("should skip existing file");
        assert_eq!(result, vec![output_path.clone()]);

        let content = fs::read(&output_path).expect("failed to read output file");
        assert_eq!(content, b"already-exists");

        let _ = fs::remove_file(output_path);
    }
}
