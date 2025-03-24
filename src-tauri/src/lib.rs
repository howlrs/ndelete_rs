use std::{collections::HashMap, path::PathBuf, result::Result};

use log::info;
use serde_json::{Value, json};

mod delete;

// Learn more about Tauri commands at https://tauri.app/develop/calling-rust/
#[tauri::command]
fn function_analytics(n: usize, paths: &str) -> Result<Value, String> {
    info!("delete count: {}, paths: {:?}", n, paths);
    let to_filepaths = {
        let mut to_filepaths = vec![];
        for path in paths.split('\n') {
            // start, endの""を削除
            let path = path.trim().trim_matches('"');

            let pathbuf = std::path::PathBuf::from(path);
            // ファイルを検証し、ファイルパスが正当であることを確認する
            // ファイルパスをチェックし、配列として返す
            let paths = match delete::ndelete::walk_dir(&pathbuf) {
                Ok(paths) => paths,
                Err(e) => {
                    return Err(e.to_string());
                }
            };
            to_filepaths.extend(paths);
        }
        to_filepaths
    };

    info!("delete count: {}, paths: {:?}", n, to_filepaths);
    Ok(json!({
        "message": "success",
        "delete_count": n,
        "paths": to_filepaths,
    }))
}

#[tauri::command]
async fn function_ndelete(n: usize, paths: Vec<PathBuf>) -> Result<Value, String> {
    info!("delete count: {}, paths: {:?}", n, paths);
    let mut success_paths = HashMap::new();
    let mut error_paths = HashMap::new();

    for path in paths.iter() {
        match delete::ndelete::overwrite_file(n as u32, path.to_path_buf()) {
            Ok(_) => {
                success_paths.insert(
                    path,
                    format!("{} deletions have been performed.", n).to_string(),
                );
            }
            Err(e) => {
                error_paths.insert(path, e.to_string());
                continue;
            }
        };

        info!("done: {:?}", path);
    }

    let result = format_result(&success_paths, &error_paths);
    let report_path = match save_result_file(&result) {
        Ok(report_path) => report_path,
        Err(e) => {
            return Err(e);
        }
    };

    Ok(json!({
        "message": format!("done, file has been overwritten and deleted {} times", n),
        "delete_count": n,
        "success_paths": success_paths,
        "error_paths": error_paths,
        "report_path": report_path,
    }))
}

#[cfg_attr(mobile, tauri::mobile_entry_point)]
pub fn run() {
    dotenv::from_filename(".env.local").unwrap();
    env_logger::Builder::from_env(env_logger::Env::default())
        .parse_env("RUST_LOG")
        .init();

    tauri::Builder::default()
        .plugin(tauri_plugin_opener::init())
        .invoke_handler(tauri::generate_handler![
            function_analytics,
            function_ndelete
        ])
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
}

// 成功・失敗結果を人間が理解しやすいように整形し、文字列として返す
fn format_result(
    success_paths: &HashMap<&PathBuf, String>,
    error_paths: &HashMap<&PathBuf, String>,
) -> String {
    let mut result = String::new();

    // 成功結果を整形
    if !success_paths.is_empty() {
        result += "The deletion was completed:\n";
        for (path, message) in success_paths {
            result += &format!("{}: \n{}\n\n", path.display(), message);
        }
    }

    // 失敗結果を整形
    if !error_paths.is_empty() {
        result += "Error:\n";
        for (path, message) in error_paths {
            result += &format!("{}: {}\n", path.display(), message);
        }
    }

    // 本日の日時と
    // プログラム名を記載
    result += &format!(
        "-------------------------\nDeleted program: {}\nDeleted at: {}\n",
        env!("CARGO_PKG_NAME"),
        chrono::Local::now().format("%Y-%m-%d %H:%M:%S")
    );

    result
}

// クロスプラットフォーム対応
// ドキュメントファイルパスにディレクトリを設けて、ファイルを保存する
fn save_result_file(result: &str) -> Result<PathBuf, String> {
    // ドキュメントファイルパスを取得
    let document_dir = match dirs::document_dir() {
        Some(document_dir) => document_dir,
        None => {
            return Err("Failed to get the document directory path.".into());
        }
    };

    // ドキュメントファイルパスにディレクトリを設ける
    let report_dir = document_dir.join("ndelete_report");
    if !report_dir.exists() {
        std::fs::create_dir(&report_dir).unwrap();
    }

    // ファイル名を生成
    let file_name = format!("report_{}.txt", chrono::Local::now().format("%Y%m%d%H%M%S"));

    // ファイルパスを生成
    let file_path = report_dir.join(file_name);

    // ファイルに書き込む
    match std::fs::write(&file_path, result) {
        Ok(_) => Ok(file_path),
        Err(e) => Err(e.to_string()),
    }
}
