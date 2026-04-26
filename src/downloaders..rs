use std::fs::File;
use std::io::copy;

pub fn download_pdf(url: &str, save_path: &str) -> Result<(), Box<dyn std::error::Error>> {
    let response = reqwest::blocking::get(url)?;

    if !response.status().is_success() {
        return Err(format!("ダウンロード失敗: {}", response.status()).into());
    }

    let mut file = File::create(save_path)?;
    let mut content = response;
    copy(&mut content, &mut file)?;

    println!("ダウンロード完了: {}", save_path);

    Ok(())
}