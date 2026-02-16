# FFmpegのインストール手順（Windows）

このプロジェクトをビルドするには、FFmpegの開発ライブラリが必要です。

## オプション1: vcpkg を使用（推奨）

```powershell
# vcpkgをインストール
git clone https://github.com/Microsoft/vcpkg.git C:\vcpkg
cd C:\vcpkg
.\bootstrap-vcpkg.bat

# FFmpegをインストール
.\vcpkg install ffmpeg:x64-windows

# 環境変数を設定
$env:VCPKG_ROOT = "C:\vcpkg"
$env:PATH += ";C:\vcpkg\installed\x64-windows\bin"

# ビルド
cd C:\Users\bene-0103\source\repos\TestTools
cargo build --release
```

## オプション2: ビルド済みFFmpegを使用

```powershell
# 1. FFmpegをダウンロード
# https://github.com/BtbN/FFmpeg-Builds/releases から ffmpeg-master-latest-win64-gpl-shared.zip をダウンロード

# 2. 展開
Expand-Archive -Path ffmpeg-master-latest-win64-gpl-shared.zip -DestinationPath C:\ffmpeg

# 3. 環境変数を設定
$env:FFMPEG_DIR = "C:\ffmpeg\ffmpeg-master-latest-win64-gpl-shared"
$env:PATH += ";C:\ffmpeg\ffmpeg-master-latest-win64-gpl-shared\bin"

# 4. pkg-configをインストール
winget install JFLarvoire.Pkg-config

# 5. ビルド
cd C:\Users\bene-0103\source\repos\TestTools
cargo build --release
```

## オプション3: 静的リンク

環境変数を設定してFFmpegをソースからビルド:

```powershell
$env:FFMPEG_BUILD_FLAGS = "--enable-gpl --enable-version3"
cargo build --release --features=static
```

## トラブルシューティング

エラーが発生した場合:
1. PowerShellを管理者権限で再起動
2. 環境変数が正しく設定されているか確認
3. cargoのキャッシュをクリア: `cargo clean`
