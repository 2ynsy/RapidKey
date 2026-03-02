# FastPulseKey

[![Rust](https://img.shields.io/badge/rust-2024-orange.svg)](https://www.rust-lang.org/)
[![Platform](https://img.shields.io/badge/platform-windows-blue.svg)](https://www.microsoft.com/windows)

**FastPulseKey** は、Windows専用のシンプルなキー連打ツールです。
軽量に動作し、特定のキーを指定した速度で自動入力できます。

---

## 主な機能

- **軽量・高速**: Rustで書かれているため、動作が軽く安定しています。
- **統計表示**: 1秒あたりの連打数（CPS）や総打鍵数を表示します。
- **選べるモード**:
  - **Toggle**: ホットキー（F8/F9）で開始・停止を切り替えます。
  - **Hold**: ホットキーを押している間だけ連打します。
  - **Burst**: 指定した回数だけ連打して止まります。
- **ホットキー**: どのアプリを使っていても `F8` または `F9` で操作可能です。
- **ポータブル**: インストール不要。`.exe` ファイルをダウンロードするだけで使えます。

---

## 🚀 インストール

[GitHub Releases](https://github.com/Azure/summer-time-renda/releases) から最新の `fastpulsekey.exe` をダウンロードして実行してください。

---

## 🛠 使い方

1. **TARGET KEY**: ボタンをクリックし、連打したいキーを押して設定します。
2. **CONFIGURATION**:
   - **Speed**: 連打速度（CPS）を調整。
   - **Mode**: 連打の挙動（トグル/ホールド/バースト）を選択。
3. **START**: 中央のボタンをクリックするか、ホットキー（**F8** または **F9**）で連打を開始します。

> [!NOTE]
> ゲーム内などでホットキーが反応しない場合は、**「管理者として実行」** してみてください。

---

Developed with Rust.
