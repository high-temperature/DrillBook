use crate::models::Question;

#[derive(Debug)]
pub struct ParseError {
    pub message: String,
}

impl std::fmt::Display for ParseError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.message)
    }
}

impl std::error::Error for ParseError {}

pub fn parse_questions(text: &str, year: u32, subject: &str) -> Result<Vec<Question>, ParseError> {
    let blocks = split_blocks(text);
    if blocks.is_empty() {
        return Err(ParseError {
            message: "設問ブロックが見つかりませんでした。".to_string(),
        });
    }

    let subject_slug = slugify(subject);
    let mut questions = Vec::with_capacity(blocks.len());

    for (idx, block) in blocks.iter().enumerate() {
        let (question_text, choices, answer_index, explanation) = parse_block(block)?;
        questions.push(Question {
            id: format!("{}_{}_{:02}", year, subject_slug, idx + 1),
            year,
            subject: subject.to_string(),
            question_text,
            choices,
            answer_index,
            explanation,
        });
    }

    Ok(questions)
}

fn split_blocks(text: &str) -> Vec<Vec<String>> {
    let mut blocks = Vec::new();
    let mut current = Vec::new();

    for line in text.lines() {
        let trimmed = line.trim();
        if trimmed.is_empty() {
            continue;
        }

        if (trimmed.starts_with("Q:") || trimmed.starts_with("問題:")) && !current.is_empty() {
            blocks.push(current);
            current = Vec::new();
        }

        current.push(trimmed.to_string());
    }

    if !current.is_empty() {
        blocks.push(current);
    }

    blocks
}

fn parse_block(block: &[String]) -> Result<(String, Vec<String>, usize, String), ParseError> {
    let mut question_text = String::new();
    let mut choices = Vec::new();
    let mut answer_index = None;
    let mut explanation = String::new();

    for line in block {
        if let Some(q) = extract_prefixed(line, &["Q:", "問題:"]) {
            question_text = q;
            continue;
        }

        if let Some(choice) = extract_choice(line) {
            choices.push(choice);
            continue;
        }

        if let Some(ans) = extract_prefixed(line, &["A:", "答え:"]) {
            let parsed = ans.parse::<usize>().map_err(|_| ParseError {
                message: format!("正答番号の解析に失敗しました: {}", ans),
            })?;
            if parsed == 0 {
                return Err(ParseError {
                    message: "正答番号は1以上で指定してください。".to_string(),
                });
            }
            answer_index = Some(parsed - 1);
            continue;
        }

        if let Some(exp) = extract_prefixed(line, &["E:", "解説:"]) {
            explanation = exp;
            continue;
        }
    }

    if question_text.is_empty() {
        return Err(ParseError {
            message: "問題文が見つかりませんでした。".to_string(),
        });
    }

    if choices.len() < 2 {
        return Err(ParseError {
            message: "選択肢が2件未満です。".to_string(),
        });
    }

    let answer_index = answer_index.ok_or_else(|| ParseError {
        message: "正答が見つかりませんでした。".to_string(),
    })?;

    if answer_index >= choices.len() {
        return Err(ParseError {
            message: "正答番号が選択肢範囲外です。".to_string(),
        });
    }

    if explanation.is_empty() {
        explanation = "（解説なし）".to_string();
    }

    Ok((question_text, choices, answer_index, explanation))
}

fn extract_prefixed(line: &str, prefixes: &[&str]) -> Option<String> {
    prefixes
        .iter()
        .find_map(|prefix| line.strip_prefix(prefix).map(|v| v.trim().to_string()))
}

fn extract_choice(line: &str) -> Option<String> {
    for marker in [". ", ") ", "．"] {
        let mut split = line.splitn(2, marker);
        let head = split.next()?.trim();
        let tail = split.next()?.trim();

        if (head.parse::<usize>().is_ok() || matches!(head, "１" | "２" | "３" | "４"))
            && !tail.is_empty()
        {
            return Some(tail.to_string());
        }
    }

    None
}

fn slugify(input: &str) -> String {
    let mut out = String::new();
    let mut prev_sep = false;

    for c in input.trim().chars() {
        if c.is_alphanumeric() {
            out.push(c.to_ascii_lowercase());
            prev_sep = false;
        } else if !prev_sep {
            out.push('_');
            prev_sep = true;
        }
    }

    out.trim_matches('_').to_string()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn parse_single_question() {
        let text = r#"
Q: 損益分岐点売上高を求める式として正しいものはどれか。
1. 固定費 ÷ 限界利益率
2. 変動費 ÷ 限界利益率
3. 固定費 ÷ 変動費率
4. 売上高 ÷ 固定費率
A: 1
E: 損益分岐点売上高は固定費を限界利益率で割って求める。
"#;

        let parsed = parse_questions(text, 2025, "財務・会計").expect("should parse");
        assert_eq!(parsed.len(), 1);
        assert_eq!(parsed[0].answer_index, 0);
        assert_eq!(parsed[0].choices.len(), 4);
    }

    #[test]
    fn fails_when_answer_out_of_range() {
        let text = r#"
Q: 問題
1. A
2. B
A: 3
E: 解説
"#;

        let result = parse_questions(text, 2025, "財務・会計");
        assert!(result.is_err());
    }
}
