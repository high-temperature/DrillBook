use rand::seq::SliceRandom;
use std::io::{self, Write};

use crate::models::Question;

pub fn run_quiz(questions: &[Question]) {
    if questions.is_empty() {
        println!("問題データがありません。");
        return;
    }

    let mut rng = rand::thread_rng();
    let question = questions.choose(&mut rng).unwrap();

    println!("==============================");
    println!("科目: {}", question.subject);
    println!("年度: {}", question.year);
    println!("問題: {}", question.question_text);
    println!("------------------------------");

    for (i, choice) in question.choices.iter().enumerate() {
        println!("{}. {}", i + 1, choice);
    }

    println!("------------------------------");
    print!("回答を入力してください (1-{}): ", question.choices.len());
    io::stdout().flush().unwrap();

    let mut input = String::new();
    io::stdin().read_line(&mut input).unwrap();

    let selected = input.trim().parse::<usize>();

    match selected {
        Ok(num) if num >= 1 && num <= question.choices.len() => {
            let selected_index = num - 1;

            if selected_index == question.answer_index {
                println!("\n正解です！");
            } else {
                println!("\n不正解です。");
                println!(
                    "正解は {}. {}",
                    question.answer_index + 1,
                    question.choices[question.answer_index]
                );
            }

            println!("\n解説:");
            println!("{}", question.explanation);
        }
        _ => {
            println!("入力が正しくありません。1〜{}の数字を入力してください。", question.choices.len());
        }
    }
}