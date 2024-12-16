use clap::Parser;
use regex::Regex;
use std::fs;
use std::path::Path;

#[derive(Parser)]
struct Cli {
    /// 対象パス
    path: String,

    /// 正規表現パターン
    #[clap(short = 'e')]
    pattern: String,

    /// ファイルをリネームせずに実行結果を表示
    #[clap(short = 'd', long = "dry_run")]
    dry_run: bool,
}

/// 正規表現パターンに基づいてディレクトリ名からプレフィックスを取得します。
///
/// # Arguments
///
/// * `pattern` - 正規表現パターン
/// * `dirname` - ディレクトリ名
///
/// # Returns
///
/// プレフィックス文字列
///
/// # Errors
///
/// 正規表現のコンパイルに失敗した場合
fn get_prefix(pattern: &str, dirname: &str) -> Result<String, regex::Error> {
    let re = Regex::new(if pattern.is_empty() { r".*" } else { pattern })?;
    Ok(re
        .captures(dirname)
        .and_then(|caps| caps.get(0))
        .map_or_else(|| "".to_string(), |m| m.as_str().to_string()))
}

/// 指定されたパス内のファイルをリネームします。
///
/// # Arguments
///
/// * `path` - 対象パス
/// * `prefix` - プレフィックス
/// * `dry_run` - ドライランフラグ
///
/// # Returns
///
/// 処理結果
///
/// # Errors
///
/// ファイルのリネームに失敗した場合
fn rename_files(path: &Path, prefix: &str, dry_run: bool) -> std::io::Result<()> {
    for entry in fs::read_dir(path)? {
        let entry = entry?;
        let filename = entry.file_name();
        let src_name = filename.to_string_lossy();
        let dest_name = format!("{}_{}", prefix, src_name);
        println!("{} -> {}", src_name, dest_name);

        if !dry_run {
            fs::rename(entry.path(), path.join(dest_name))?;
        }
    }
    Ok(())
}

fn main() {
    // コマンドライン引数を解析
    let args = Cli::parse();

    let path = Path::new(&args.path);
    let pattern = &args.pattern;
    let dry_run = args.dry_run;

    // ディレクトリ名を取得
    let dirname = match path.file_name() {
        Some(name) => name.to_string_lossy().to_string(),
        None => {
            eprintln!("Invalid path");
            return;
        }
    };
    dbg!(&dirname);

    // プレフィックスを取得
    let prefix = match get_prefix(pattern, &dirname) {
        Ok(prefix) => prefix,
        Err(err) => {
            eprintln!("Error compiling regex: {}", err);
            return;
        }
    };
    dbg!(&prefix);

    // ファイルをリネーム
    if let Err(err) = rename_files(path, &prefix, dry_run) {
        eprintln!("Error renaming files: {}", err);
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_get_prefix() {
        let pattern = r"\d+";
        let dirname = "20241231_sample";
        let prefix = get_prefix(pattern, dirname).unwrap();
        assert_eq!(prefix, "20241231");
    }

    #[test]
    fn test_get_prefix_long_path() {
        let pattern = r"\d+";
        let dirname = "sample/20241231_sample";
        let prefix = get_prefix(pattern, dirname).unwrap();
        assert_eq!(prefix, "20241231");
    }

    #[test]
    fn test_get_prefix_empty_pattern() {
        let pattern = "";
        let dirname = "20241231_sample";
        let prefix = get_prefix(pattern, dirname).unwrap();
        assert_eq!(prefix, "20241231_sample");
    }

    #[test]
    fn test_invalid_regex() {
        let pattern = r"(\d+";
        let dirname = "20241231_sample";
        let result = get_prefix(pattern, dirname);
        assert!(result.is_err());
    }
}
