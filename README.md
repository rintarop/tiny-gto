# tiny-gto

Kuhn Poker のゲーム理論的最適解（GTO）を計算するプログラムです。Counterfactual Regret Minimization（CFR）アルゴリズムを使用して実装されています。

## Kuhn Poker とは

Kuhn Poker は最もシンプルなポーカーの一種で、ゲーム理論の研究によく使用されます。

**ルール:**
- 3枚のカード（J, Q, K）を使用
- 2人のプレイヤーが1枚ずつカードを引く
- 各プレイヤーは1チップずつアンティを支払う
- プレイヤー1から順番にアクションを選択
- 可能なアクション: Check, Bet, Call, Fold
- より強いカードを持っているプレイヤーが勝利

## セットアップ

### 1. Rust と Cargo のインストール

Rustがまだインストールされていない場合は、以下の方法でインストールできます。

**macOS / Linux:**
```bash
curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs | sh
```

**Windows:**

[rustup-init.exe](https://rustup.rs/) をダウンロードして実行してください。

インストール後、新しいターミナルセッションを開いて以下のコマンドで確認します：

```bash
rustc --version
cargo --version
```

### 2. プロジェクトのインストール

```bash
git clone https://github.com/rintarop/tiny-gto.git
cd tiny-gto
```

## 実行方法

```bash
cargo run
```

## テストの実行

```bash
cargo test
```

## ライセンス

MIT
