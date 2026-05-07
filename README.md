# mdls2mmd

Markdown リストから Mermaid フローチャート**のコード**を生成する CLI ツール. 

## ビルド

```bash
cargo build --release
```

または

```bash
nix build
```

## 使い方

```bash
# ファイルを変換
mdls2mmd input.md

# 方向を指定 (LR / TD / RL / BT)
mdls2mmd -d TD input.md

# ファイルへ出力
mdls2mmd input.md -o diagram.mmd

# stdin からパイプ
cat list.md | mdls2mmd
```

NixOS であれば

```bash
nix run github:espeon011/mdls2mmd -- input.md
```

のようにインストールなしで実行可能. 
