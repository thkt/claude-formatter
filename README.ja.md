[English](README.md) | **日本語**

# claude-formatter

Claude Code の PostToolUse hook。Write/Edit 後にファイルを自動整形します（oxfmt, biome 対応）。

## 特徴

- **oxfmt 統合**（優先）: [oxc.rs](https://oxc.rs) の Rust 製 Prettier 互換フォーマッター
- **biome 統合**（フォールバック）: [biomejs.dev](https://biomejs.dev) のコード整形 + import 整理
- **EOF 改行**: 言語フォーマッターの対象外ファイルに末尾改行を付与
- **プロジェクトローカル解決**: `node_modules/.bin/` のバイナリを優先使用

## インストール

### Homebrew（推奨）

```bash
brew install thkt/tap/formatter
```

### リリースバイナリから

[Releases](https://github.com/thkt/claude-formatter/releases) から最新バイナリをダウンロード:

```bash
# macOS (Apple Silicon)
curl -L https://github.com/thkt/claude-formatter/releases/latest/download/formatter-aarch64-apple-darwin -o formatter
chmod +x formatter
mv formatter ~/.local/bin/
```

### ソースから

> **注意**: プロジェクトディレクトリ内にクローンしないでください。ネストされた git リポジトリとして残り、プロジェクトの git 操作に干渉する可能性があります。

```bash
cd /tmp
git clone https://github.com/thkt/claude-formatter.git
cd claude-formatter
cargo build --release
cp target/release/formatter ~/.local/bin/
cd .. && rm -rf claude-formatter
```

## 使い方

### Claude Code Hook として

`~/.claude/settings.json` に追加:

```json
{
  "hooks": {
    "PostToolUse": [
      {
        "hooks": [
          {
            "type": "command",
            "command": "formatter",
            "timeout": 2000
          }
        ],
        "matcher": "Write|Edit|MultiEdit"
      }
    ]
  }
}
```

### guardrails 併用（推奨）

```json
{
  "hooks": {
    "PreToolUse": [
      {
        "hooks": [
          {
            "type": "command",
            "command": "guardrails",
            "timeout": 1000
          }
        ],
        "matcher": "Write|Edit|MultiEdit"
      }
    ],
    "PostToolUse": [
      {
        "hooks": [
          {
            "type": "command",
            "command": "formatter",
            "timeout": 2000
          }
        ],
        "matcher": "Write|Edit|MultiEdit"
      }
    ]
  }
}
```

## 要件

フォーマッターを少なくとも 1 つインストール（oxfmt 推奨）:

- [oxfmt](https://oxc.rs/docs/guide/usage/formatter)（`npm i -g oxfmt`）— **推奨**
- [biome](https://biomejs.dev)（`brew install biome` または `npm i -g @biomejs/biome`）— フォールバック

### フォーマッター優先順位

formatter は **oxfmt を優先**します。oxfmt が利用できない場合、biome にフォールバックします。ファイルごとに 1 つだけ実行されます。

| 条件                             | 使用フォーマッター |
| -------------------------------- | ------------------ |
| oxfmt がインストール済み         | oxfmt              |
| oxfmt 未インストール、biome あり | biome              |
| どちらも未インストール           | EOF 改行のみ       |

## 対応ファイル

| フォーマッター | 拡張子                                                                                                                                                                      |
| -------------- | --------------------------------------------------------------------------------------------------------------------------------------------------------------------------- |
| oxfmt          | `.ts` `.tsx` `.js` `.jsx` `.mts` `.cts` `.mjs` `.cjs` `.json` `.jsonc` `.json5` `.css` `.scss` `.less` `.html` `.vue` `.yaml` `.yml` `.toml` `.md` `.mdx` `.graphql` `.gql` |
| biome          | `.ts` `.tsx` `.js` `.jsx` `.mts` `.cts` `.mjs` `.cjs` `.json` `.jsonc` `.css`                                                                                               |

## 動作フロー

1. stdin から PostToolUse hook の JSON を読み取る
2. Write/Edit/MultiEdit 以外のツールは無視
3. ファイルパスを正規化（シンボリックリンク、null バイト、相対パスを拒否）
4. ファイルがカレントディレクトリ配下にあることを検証
5. git ルートの `.claude-formatter.json` を読み込む（存在する場合）
6. 優先順位に従ってフォーマッターを選択: oxfmt > biome
7. ファイルをインプレースで整形

## 終了コード

| コード | 意味 |
| ------ | ---- |
| 0      | 常に |

フォーマッターは操作をブロックしません。成功時はサイレントに整形し、エラーは stderr に出力します。

## 設定

プロジェクトルート（`.git/` の隣）に `.claude-formatter.json` を配置します。すべてのフィールドはオプションで、オーバーライドしたいもののみ指定してください。

**デフォルト**（設定ファイル不要）:

- すべてのフォーマッターが有効

### スキーマ

```json
{
  "enabled": true,
  "formatters": {
    "oxfmt": true,
    "biome": true,
    "eofNewline": true
  }
}
```

### 設定例

biome を無効化（oxfmt のみ使用）:

```json
{
  "formatters": {
    "biome": false
  }
}
```

oxfmt を無効化（biome を使用）:

```json
{
  "formatters": {
    "oxfmt": false
  }
}
```

プロジェクトで formatter を無効化:

```json
{
  "enabled": false
}
```

### 設定の解決

設定ファイルは、対象ファイルから最も近い `.git` ディレクトリまで上方向に探索されます。`.claude-formatter.json` が存在すればデフォルトとマージされます。

```text
project-root/          ← .git/ + .claude-formatter.json はここ
├── src/
│   └── app.ts         ← 整形対象ファイル → 上方向に設定を探索
├── .git/
└── .claude-formatter.json
```

### `~/.claude` によるグローバル設定

`~/.claude` が git リポジトリの場合、そこに `.claude-formatter.json` を置くと `~/.claude/` 配下の全ファイルに対するグローバルデフォルトとして機能します。各プロジェクトの `.claude-formatter.json` が優先されます。

## ライセンス

MIT
