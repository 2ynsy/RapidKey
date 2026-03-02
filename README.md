# RapidKey⚡

[![Rust](https://img.shields.io/badge/rust-2026-orange.svg)](https://www.rust-lang.org/)
[![Platform](https://img.shields.io/badge/platform-windows-blue.svg)](https://www.microsoft.com/windows)

**RapidKey** は、究極のパフォーマンスと直感的な操作性を追求した Windows 専用の超高速キー連打ツールです。
ゲーム、自動化テスト、アクセシビリティなど、ミリ秒単位の正確さが求められるシーンで真価を発揮します。

---

## 💎 特徴

- **高性能・低レイテンシ**: Rust 言語による実装で、CPU負荷を最小限に抑えつつ安定した最高速クリックを実現。
- **高精度アナリティクス**: 実測 CPS（1秒間の打鍵数）、総打鍵数、稼働時間をリアルタイム計算。
- **柔軟なアクションモード**:
  - **Toggle**: ホットキー（F8/F9）で ON/OFF を切り替え。
  - **Hold**: ホットキーを押している間のみ動作。
  - **Burst**: 指定した回数に到達すると自動停止。
- **グローバルホットキー**: どのアプリケーションがアクティブでも `F8` または `F9` で即座に制御。
- **ポータブル**: インストール不要。単一の `.exe` ファイルのみで動作。

---

## 🚀 インストール

[GitHub Releases](https://github.com/Azure/summer-time-renda/releases) ページから最新の `RapidKey.exe` をダウンロードして実行してください。

## 🛠 使い方

1. **TARGET KEY**: 中央の大きなボタンをクリックし、連打したいキーを押します。
2. **CONFIGURATION**:
   - **Speed**: 1〜120 CPS の範囲で速度を設定。
   - **Mode**: 挙動（トグル/ホールド/バースト）を選択。
3. **INITIALIZE**: `START` ボタンまたはホットキー（**F8 / F9**）で連打を開始。

> [!IMPORTANT]
> Windows のセキュリティ仕様により、他のアプリ（ゲーム等）でホットキーを有効にするには **「管理者として実行」** する必要がある場合があります。
