# RapidKey ⚡ キー連打ツール

Windows上でキーを自動連打するデスクトップツールです。

## 機能

- 任意のキーを連打（クリックでキャプチャ or よく使うキーをワンクリック設定）
- 連打速度を 1〜60 CPS で調整
- **3 つのモード**
  - 🔁 トグル：ボタンまたは F8 でオン/オフ
  - 🖱 ホールド：F8 を押している間だけ連打
  - 🔢 回数指定：指定した回数で自動停止
- グローバルホットキー **F8**（どのウィンドウがアクティブでも動作）
- リアルタイム統計表示（総打鍵数 / 実測 CPS / 経過時間）

## 使い方

### Python で実行

```bash
# 依存ライブラリのインストール
pip install keyboard

# 起動
python rapidkey.py
```

> ⚠️ `keyboard` ライブラリは管理者権限を推奨します。

### EXE をビルド

```bash
pip install pyinstaller
python -m PyInstaller --onefile --noconsole --name RapidKey rapidkey.py
# → dist/RapidKey.exe が生成されます
```

## 動作環境

- Windows 10 / 11
- Python 3.11+
