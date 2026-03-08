[English](README.md) | **日本語**

# formatter

Claude Code の PostToolUse hook。Write/Edit 後にファイルを自動整形します（oxfmt, biome 対応）。

## 特徴

- **oxfmt 統合**（優先）: [oxc.rs](https://oxc.rs) の Rust 製 Prettier 互換フォーマッター
- **biome 統合**（フォールバック）: [biomejs.dev](https://biomejs.dev) のコード整形 + import 整理
- **EOF 改行**: 言語フォーマッターの対象外ファイルに末尾改行を付与
- **プロジェクトローカル解決**: `node_modules/.bin/` のバイナリを優先使用

## インストール

### Claude Code Plugin（推奨）

バイナリのインストールと hook の登録を自動で行います:

```bash
claude plugins marketplace add github:thkt/formatter
claude plugins install formatter
```

バイナリが未インストールの場合、同梱のインストーラを実行:

```bash
~/.claude/plugins/cache/formatter/formatter/*/hooks/install.sh
```

### Homebrew

```bash
brew install thkt/tap/formatter
```

### リリースバイナリから

[Releases](https://github.com/thkt/formatter/releases) から最新バイナリをダウンロード:

```bash
# macOS (Apple Silicon)
curl -L https://github.com/thkt/formatter/releases/latest/download/formatter-aarch64-apple-darwin.tar.gz | tar xz
mv formatter ~/.local/bin/
```

### ソースから

```bash
cd /tmp
git clone https://github.com/thkt/formatter.git
cd formatter
cargo build --release
cp target/release/formatter ~/.local/bin/
cd .. && rm -rf formatter
```

## 使い方

### Claude Code Hook として

プラグインとしてインストールした場合、hook は自動で登録されます。手動で設定する場合は `~/.claude/settings.json` に追加:

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
5. `.claude/tools.json` または `.claude-formatter.json` から設定を読み込む（存在する場合）
6. 優先順位に従ってフォーマッターを選択: oxfmt > biome
7. ファイルをインプレースで整形

## 終了コード

| コード | 意味 |
| ------ | ---- |
| 0      | 常に |

フォーマッターは操作をブロックしません。成功時はサイレントに整形し、エラーは stderr に出力します。

## 設定

プロジェクトルートの `.claude/tools.json` に `formatter` キーを追加します。すべてのフィールドはオプションで、オーバーライドしたいもののみ指定してください。

> **移行**: プロジェクトルートの `.claude-formatter.json` はレガシーフォールバックとして引き続きサポート。両方存在する場合は `.claude/tools.json` が優先。

**デフォルト**（設定ファイル不要）:

- すべてのフォーマッターが有効

### スキーマ

```json
{
  "formatter": {
    "enabled": true,
    "formatters": {
      "oxfmt": true,
      "biome": true,
      "eofNewline": true
    }
  }
}
```

### 設定例

biome を無効化（oxfmt のみ使用）:

```json
{
  "formatter": {
    "formatters": {
      "biome": false
    }
  }
}
```

oxfmt を無効化（biome を使用）:

```json
{
  "formatter": {
    "formatters": {
      "oxfmt": false
    }
  }
}
```

プロジェクトで formatter を無効化:

```json
{
  "formatter": {
    "enabled": false
  }
}
```

### 設定の解決

設定ファイルは、対象ファイルから最も近い `.git` ディレクトリまで上方向に探索されます。`.claude/tools.json` に `formatter` キーがあればデフォルトとマージされます。

```text
project-root/          ← .git/ + .claude/tools.json はここ
├── .claude/
│   └── tools.json     ← {"formatter": {"formatters": {"oxfmt": false}}}
├── src/
│   └── app.ts         ← 整形対象ファイル → 上方向に設定を探索
└── .git/
```

## ライセンス

MIT
