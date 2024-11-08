use clap::Parser;
use serde::Deserialize;
use serde_json::json;
use std::fs;
use std::path::Path;
use tokio;

#[derive(Parser)]
struct Args {
    /// extensions.jsonファイルへのパス
    #[arg(short, long, default_value = "extensions.json")]
    input: String,

    /// 出力先フォルダ
    #[arg(short, long, default_value = "extensions")]
    destination: String,

    /// 既に存在する場合でも再ダウンロードする
    #[arg(long)]
    no_cache: bool,

    /// プロキシURL
    #[arg(long)]
    proxy: Option<String>,

    /// 詳細な出力を表示
    #[arg(short, long)]
    verbose: bool,
}

#[derive(Deserialize)]
struct Extensions {
    recommendations: Vec<String>,
}

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    let args = Args::parse();

    // extensions.jsonファイルを読み込む
    let file_content = fs::read_to_string(&args.input)?;
    let extensions: Extensions = serde_json::from_str(&file_content)?;

    // 出力先ディレクトリを作成
    fs::create_dir_all(&args.destination)?;

    // 各拡張機能をダウンロード
    for extension in extensions.recommendations {
        if let Err(e) = download_extension(
            &extension,
            &args.destination,
            args.no_cache,
            args.proxy.as_deref(),
            args.verbose,
        )
        .await
        {
            eprintln!("{}のダウンロード中にエラーが発生しました: {}", extension, e);
        }
    }

    Ok(())
}

async fn download_extension(
    extension: &str,
    destination: &str,
    no_cache: bool,
    proxy: Option<&str>,
    verbose: bool,
) -> Result<(), Box<dyn std::error::Error>> {
    if verbose {
        println!("拡張機能を処理中: {}", extension);
    }

    let parts: Vec<&str> = extension.split('.').collect();
    if parts.len() != 2 {
        eprintln!("無効な拡張機能識別子: {}", extension);
        return Ok(());
    }
    let publisher = parts[0];
    let extension_name = parts[1];

    // 最新バージョンを取得
    let version = get_extension_version(publisher, extension_name, proxy, verbose).await?;
    if verbose {
        println!("{}の最新バージョン: {}", extension, version);
    }

    // ダウンロードURLを作成
    let download_url = format!(
        "https://{publisher}.gallery.vsassets.io/_apis/public/gallery/publisher/{publisher}/extension/{extension_name}/{version}/assetbyname/Microsoft.VisualStudio.Services.VSIXPackage",
        publisher = publisher,
        extension_name = extension_name,
        version = version
    );

    // ファイルパスを準備
    let file_name = format!("{publisher}.{extension_name}-{version}.vsix");
    let file_path = format!("{}/{}", destination, file_name);

    // ファイルが既に存在するか確認
    if !no_cache && Path::new(&file_path).exists() {
        if verbose {
            println!("ファイル{}は既に存在します。ダウンロードをスキップします。", file_path);
        }
        return Ok(());
    }

    // HTTPクライアントを構築
    let client_builder = reqwest::Client::builder();
    let client = if let Some(proxy_url) = proxy {
        if verbose {
            println!("プロキシを使用中: {}", proxy_url);
        }
        let proxy = reqwest::Proxy::all(proxy_url)?;
        client_builder.proxy(proxy).build()?
    } else {
        client_builder.build()?
    };

    // VSIXファイルをダウンロード
    if verbose {
        println!("{}からダウンロード中", download_url);
    }
    let resp = client.get(&download_url).send().await?;
    if !resp.status().is_success() {
        eprintln!("{}のダウンロードに失敗しました", extension);
        return Err(Box::from("VSIXのダウンロードに失敗しました"));
    }
    let vsix_content = resp.bytes().await?;

    // ファイルを保存
    fs::write(&file_path, &vsix_content)?;
    if verbose {
        println!("{}に保存しました", file_path);
    }

    Ok(())
}

async fn get_extension_version(
    publisher: &str,
    extension_name: &str,
    proxy: Option<&str>,
    verbose: bool,
) -> Result<String, Box<dyn std::error::Error>> {
    let api_url = "https://marketplace.visualstudio.com/_apis/public/gallery/extensionquery";

    let payload = json!({
        "filters": [{
            "criteria": [
                {"filterType": 7, "value": format!("{}.{}", publisher, extension_name)}
            ]
        }],
        "flags": 914
    });

    // HTTPクライアントを構築
    let client_builder = reqwest::Client::builder();
    let client = if let Some(proxy_url) = proxy {
        if verbose {
            println!("APIリクエストにプロキシを使用中: {}", proxy_url);
        }
        let proxy = reqwest::Proxy::all(proxy_url)?;
        client_builder.proxy(proxy).build()?
    } else {
        client_builder.build()?
    };

    // POSTリクエストを送信
    if verbose {
        println!("Marketplace APIにクエリを送信中: {}.{}", publisher, extension_name);
    }
    let resp = client
        .post(api_url)
        .header("Content-Type", "application/json")
        .header("Accept", "application/json;api-version=3.0-preview.1")
        .header("User-Agent", "Offline VSIX/1.0")
        .json(&payload)
        .send()
        .await?;

    if !resp.status().is_success() {
        eprintln!("Marketplace APIのクエリに失敗しました");
        return Err(Box::from("Marketplace APIのクエリに失敗しました"));
    }

    let resp_json: serde_json::Value = resp.json().await?;

    // バージョンを抽出
    let version = resp_json["results"][0]["extensions"][0]["versions"][0]["version"]
        .as_str()
        .ok_or("拡張機能のバージョン取得に失敗しました")?
        .to_string();

    Ok(version)
}

