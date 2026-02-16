# Video Player UI (Slint + FFmpeg)

Slintを使用した動画プレイヤーUIで、FFmpegを使って実際に動画を再生できます。

## ✨ 機能

- 📁 **動画ファイル選択**: ファイルダイアログから動画ファイルを選択
- 🔁 **リピート回数設定**: 再生回数を指定（無限リピートも可能）
- ▶️ **動画再生**: FFmpegまたはシステムのデフォルトプレイヤーで再生
- ⏸️ **再生コントロール**: 再生/一時停止/停止
- ⏱️ **再生時間表示**: 現在の再生時間と総時間を表示
- 📊 **ステータス表示**: 現在の再生状態とリピート設定を表示

## 🔧 必要な環境

- **Rust** (1.70以上推奨)
- **Cargo**
- **FFmpeg** (推奨 - より良い再生体験のため)

## 📥 FFmpegのインストール（推奨）

FFmpegをインストールすることで、アプリ内で直接動画を再生できます。

### 方法1: 手動インストール（最も簡単）

1. [FFmpeg公式ダウンロード](https://www.gyan.dev/ffmpeg/builds/)から**ffmpeg-release-essentials.zip**をダウンロード
2. 任意の場所に解凍（例：`C:\ffmpeg`）
3. システムのPATH環境変数に`C:\ffmpeg\bin`を追加

**PATH環境変数の設定方法**:
```powershell
# PowerShellで実行（管理者権限）
[Environment]::SetEnvironmentVariable("Path", $env:Path + ";C:\ffmpeg\bin", "Machine")
```

または、
1. Windowsキー + R → `sysdm.cpl` → Enter
2. 「詳細設定」→「環境変数」
3. システム環境変数の「Path」を編集
4. 新規で`C:\ffmpeg\bin`を追加

### 方法2: Chocolatey

```powershell
choco install ffmpeg
```

### 方法3: winget

```powershell
winget install Gyan.FFmpeg
```

## 🚀 ビルドと実行

```bash
# ビルド
cargo build --release

# 実行
cargo run --release
```

初回ビルドには数分かかります。

## 📁 プロジェクト構造

```
TestTools/
├── Cargo.toml              # プロジェクト設定（依存関係）
├── build.rs                # Slintビルドスクリプト
├── src/
│   ├── main.rs             # メインアプリケーションロジック
│   └── player.rs           # 動画再生ロジック
└── ui/
    └── video_player.slint  # SlintによるUI定義
```

## 🎮 UI操作方法

### 1. 動画ファイルの選択
「選択...」ボタンをクリックして動画ファイルを選択します。
対応フォーマット: MP4, AVI, MKV, MOV, WMV, FLV, WebM など

### 2. リピート回数の設定
- **直接入力**: テキストボックスに数値を入力
- **+/-ボタン**: クリックで増減
- **∞ボタン**: 無限リピート設定

### 3. 再生コントロール
- **▶ 再生**: 動画の再生を開始
- **⏸ 一時停止**: 再生中に一時停止
- **⏹ 停止**: 再生を停止して先頭に戻る

### 4. シークバー
スライダーをドラッグして動画の任意の位置に移動できます。

## 🎬 動作モード

### FFmpegモード（推奨）✅
FFmpegがインストールされている場合
- `ffplay`でアプリ内ウィンドウで動画を再生
- `ffprobe`で動画の長さを自動取得
- 正確な再生時間追跡

### デフォルトプレイヤーモード📺
FFmpegがない場合の自動フォールバック
- Windowsのデフォルトプレイヤー（Windows Media Playerなど）で再生
- 動画の長さはデフォルト値（5分）を使用
- 基本的な再生機能のみ

## 🐛 トラブルシューティング

### FFmpegが見つからない

**症状**: 
```
ffplayが見つかりません。システムのデフォルトプレイヤーで開きます。
```

**解決方法**:
1. コマンドプロンプトで`ffmpeg -version`を実行して確認
2. 見つからない場合は上記の「FFmpegのインストール」を参照
3. PATH環境変数が正しく設定されているか確認
4. システムを再起動してPATHを反映

### ビルドエラー

**症状**: `ffmpeg-sys-next`のビルドエラー

これは以前のバージョンの問題です。最新のコードでは解決済みです。
`cargo clean`を実行してから再ビルドしてください。

```powershell
cargo clean
cargo build --release
```

### 警告メッセージ

**症状**: `padding-left only has effect on layout elements`

これは無害な警告で、動作には影響しません。

## 🎨 カスタマイズ

### UIのカスタマイズ
[ui/video_player.slint](ui/video_player.slint)を編集してUIをカスタマイズできます。

### 動画再生ロジックのカスタマイズ
[src/player.rs](src/player.rs)を編集して動画再生ロジックをカスタマイズできます。

### アプリケーションロジック
[src/main.rs](src/main.rs)を編集してアプリケーションの動作をカスタマイズできます。

## 📝 技術スタック

- **UI Framework**: [Slint](https://slint.dev/) - Rustネイティブな宣言的UIフレームワーク
- **動画再生**: FFmpeg (ffplay/ffprobe) - 業界標準の動画処理ツール
- **ファイル選択**: rfd - クロスプラットフォームなファイルダイアログ
- **言語**: Rust - 安全で高速なシステムプログラミング言語

## 🔮 今後の機能拡張予定

- [ ] 音量コントロール
- [ ] 再生速度変更
- [ ] プレイリスト機能
- [ ] 字幕表示
- [ ] フルスクリーンモード
- [ ] キーボードショートカット

## 📄 ライセンス

MIT

## 🙏 謝辞

- [Slint](https://slint.dev/) - 素晴らしいUIフレームワーク
- [FFmpeg](https://ffmpeg.org/) - 強力な動画処理ツール
