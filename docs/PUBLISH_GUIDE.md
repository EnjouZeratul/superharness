# crates.io 发布指南

## 发布顺序

按依赖关系，必须按以下顺序发布：

```
1. sh-layer0 (无内部依赖)
2. sh-layer1 (依赖 sh-layer0)
3. sh-layer2 (依赖 sh-layer0, sh-layer1)
4. sh-layer3 (依赖 sh-layer0, sh-layer1, sh-layer2)
5. sh-layer4 (依赖 sh-layer0, sh-layer1, sh-layer2, sh-layer3)
6. sh-core    (依赖所有 layer)
7. superharness-cli (依赖 sh-core, sh-layer4)
```

## 发布命令

### 前置准备

1. 确保已登录 crates.io:
```bash
cargo login YOUR_API_TOKEN
```

2. 确保所有更改已提交:
```bash
git add .
git commit -m "chore: prepare for v0.1.0 release"
git tag v0.1.0
git push origin main --tags
```

### 发布步骤

```bash
# Step 1: 发布 layer0
cd rust/layer0
cargo publish
cd ../..

# 等待 30 秒让 crates.io 索引更新
sleep 30

# Step 2: 发布 layer1
cd rust/layer1
cargo publish
cd ../..
sleep 30

# Step 3: 发布 layer2
cd rust/layer2
cargo publish
cd ../..
sleep 30

# Step 4: 发布 layer3
cd rust/layer3
cargo publish
cd ../..
sleep 30

# Step 5: 发布 layer4
cd rust/layer4
cargo publish
cd ../..
sleep 30

# Step 6: 发布 sh-core
cd rust/sh-core
cargo publish
cd ../..
sleep 30

# Step 7: 发布 CLI
cd cli
cargo publish
cd ..
```

## 发布前检查

```bash
# 检查所有 Cargo.toml 配置
cargo check

# 运行所有测试
cargo test --all

# 检查 clippy
cargo clippy --all

# 检查格式
cargo fmt --all -- --check

# Dry-run 测试 (layer0 可以先测试)
cd rust/layer0
cargo publish --dry-run
```

## 发布后验证

```bash
# 验证包已发布
cargo search sh-layer0
cargo search sh-layer1
cargo search sh-layer2
cargo search sh-layer3
cargo search sh-layer4
cargo search sh-core
cargo search superharness-cli

# 安装测试
cargo install superharness-cli
superharness --version
```

## 回滚说明

如果发布失败，需要：
1. 修复问题
2. 更新版本号（如 0.1.0 -> 0.1.1）
3. 重新发布

注意：crates.io 不允许删除已发布的版本，只能 yank：
```bash
cargo yank sh-layer0@0.1.0
```

## 常见问题

### Q: 提示依赖找不到
A: 等待 30-60 秒让 crates.io 索引更新，或手动更新索引：
```bash
cargo search sh-layer0
```

### Q: 提示版本已存在
A: 更新 Cargo.toml 中的版本号

### Q: 发布卡住
A: 检查网络连接，或使用代理：
```bash
export CARGO_HTTP_PROXY=http://127.0.0.1:7890
```
