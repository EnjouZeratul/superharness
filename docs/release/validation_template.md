# 发布验证模板

> 用户执行发布后验证时填写此报告

## 验证信息

- **验证人**: [用户名]
- **验证时间**: [日期时间]
- **验证环境**: [操作系统, Python版本, Rust版本]

---

## PyPI 安装验证

### 安装命令
```bash
pip install continuum
```

### 验证命令
```bash
python -c "from continuum_sdk import Agent; print('✓ SDK 导入成功')"
python -c "from continuum_sdk import Session; print('✓ Session 导入成功')"
python -c "from continuum_sdk.tools import ToolRegistry; print('✓ ToolRegistry 导入成功')"
```

### 结果
- [ ] 安装成功
- [ ] SDK 导入成功
- [ ] 版本正确

**错误信息** (如有):
```
[粘贴错误信息]
```

---

## crates.io 安装验证

### 安装命令
```bash
cargo install continuum
```

### 验证命令
```bash
sh --version
sh config --help
```

### 结果
- [ ] 安装成功
- [ ] CLI 可执行
- [ ] 命令正常

**错误信息** (如有):
```
[粘贴错误信息]
```

---

## 端到端验证

### 配置初始化
```bash
sh config init
```

**结果**:
- [ ] 配置文件创建成功
- [ ] 默认配置正确

### 提供商配置
```bash
# 使用真实 API key
export ANTHROPIC_API_KEY=your-key

# 或配置自定义提供商
sh config add-provider custom --api-key $KEY --url $URL --model glm-5
sh config use custom
```

**结果**:
- [ ] API key 设置成功
- [ ] 提供商切换成功

### Agent 运行
```bash
sh run "你好，请介绍一下你自己"
```

**预期输出**:
```
Agent: [AI 响应]
```

**实际输出**:
```
[粘贴实际输出]
```

### Python SDK 运行
```python
from continuum_sdk import Agent

agent = Agent()
result = agent.run("hello")
print(result)
```

**实际输出**:
```
[粘贴实际输出]
```

---

## 问题记录

| # | 问题 | 状态 | 备注 |
|---|------|------|------|
| 1 | | | |
| 2 | | | |

---

## 总体评价

- [ ] ✅ 全部通过
- [ ] ⚠️ 部分问题
- [ ] ❌ 主要功能失败

**备注**:
```
[用户备注]
```

---

*验证报告模板 - Continuum v1.0.0*