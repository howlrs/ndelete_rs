use log::{error, info};
use rand::TryRngCore;
use rand::rngs::OsRng;
use std::fs::OpenOptions;
use std::io::{Seek, SeekFrom, Write};
use std::path::{Path, PathBuf};

pub fn overwrite_file(
    overwrite_count: u32,
    file_path: PathBuf,
) -> Result<(), Box<dyn std::error::Error>> {
    if check_filepath(file_path.clone()).is_err() {
        return Err("Invalid file path".into());
    }

    let file_size = std::fs::metadata(file_path.clone())?.len();
    let mut rng = OsRng;

    for i in 0..overwrite_count {
        info!("Overwrite iteration: {}", i + 1);
        let mut buffer = vec![0u8; 4096];
        let mut current_position: u64 = 0;

        // 都度ファイルを開き、metadata を更新しながら書き込む
        let mut file = OpenOptions::new()
            .read(true)
            .write(true)
            .open(file_path.clone())?; // ここでファイルを開く

        while current_position < file_size {
            let bytes_to_write =
                std::cmp::min(buffer.len() as u64, file_size - current_position) as usize;
            match rng.try_fill_bytes(&mut buffer[..bytes_to_write]) {
                Ok(_) => {}
                Err(e) => {
                    error!("Error: {}", e);
                    break;
                }
            };
            file.seek(SeekFrom::Start(current_position))?;
            file.write_all(&buffer[..bytes_to_write])?;
            current_position += bytes_to_write as u64;

            // 定期的に sync_all を呼び出す
            if current_position % (1024 * 1024) == 0 {
                // 1MB ごとに sync
                file.sync_all()?;
            }
        }
        file.sync_all()?; // 最後に必ず sync
    }

    // データ領域を乱数で上書きした後、ファイルを削除
    std::fs::remove_file(file_path)?;
    info!("File successfully overwritten and deleted.");
    Ok(())
}

// rm *, rm /, rm -rf /, rm -rf /*, rm -rf ., rm -rf ./*, rm -rf ./*/*
// などの破壊的なコマンドを実行しないように、ファイルパスのチェックを行う
fn check_filepath(path: PathBuf) -> Result<(), String> {
    if path.is_dir() {
        // ディレクトリパスは許可しない
        return Err("Directory path is not allowed.".into());
    } else if path.is_relative() {
        // 相対パスは許可しない
        return Err("Relative path is not allowed.".into());
    }

    // 網羅的にルートディレクトリを指定するパスを弾く
    if path.starts_with("/") {
        return Err("Root directory path is not allowed.".into());
    }

    // 空白などの危険なパス
    let path = path.to_str().unwrap();
    if path.is_empty() {
        return Err("Empty path is not allowed.".into());
    }

    Ok(())
}

pub fn walk_dir(dir: &Path) -> Result<Vec<PathBuf>, Box<dyn std::error::Error>> {
    let mut files = Vec::new();
    for entry in std::fs::read_dir(dir)? {
        let entry = entry?;
        let path = entry.path();

        // ファイルが存在しない場合はスキップ
        if !path.exists() {
            continue;
        }

        if path.is_dir() {
            files.append(&mut walk_dir(&path)?);
        } else {
            files.push(path);
        }
    }
    Ok(files)
}
