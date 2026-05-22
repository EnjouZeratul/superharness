# T1 任务：builtin.py 测试

## 基本信息

| 项目 | 内容 |
|------|------|
| 终端 | T1 |
| 模块 | `python/continuum_sdk/tools/builtin.py` |
| 当前覆盖率 | 0% |
| 目标覆盖率 | 80% |
| 优先级 | P0 |
| 参考规范 | `docs/TESTING_STANDARDS.md` |

---

## 任务目标

为内置工具集编写完整测试，确保每个工具的实际执行逻辑正确。

---

## 需测试的工具

### 1. ReadTool（文件读取）
- [ ] 正常读取文件
- [ ] 读取大文件（分页）
- [ ] 读取不存在的文件（错误处理）
- [ ] 读取无权限文件
- [ ] offset/limit 参数测试

### 2. WriteTool（文件写入）
- [ ] 写入新文件
- [ ] 覆盖已有文件
- [ ] 追加模式
- [ ] 创建目录（create_dirs=True）
- [ ] 备份功能（backup=True）

### 3. BashTool（命令执行）
- [ ] 正常命令执行
- [ ] 命令超时处理
- [ ] 命令不存在
- [ ] 非零退出码
- [ ] 危险命令拦截

### 4. GrepTool（内容搜索）
- [ ] 正则搜索
- [ ] 文件过滤
- [ ] case insensitive
- [ ] output_mode: content/files_with_matches/count

### 5. GlobTool（文件匹配）
- [ ] 通配符匹配
- [ ] 递归搜索
- [ ] 返回文件列表

---

## 测试要求

### 禁止行为
```python
# ❌ 不要这样写
def test_read(self):
    reader = ReadTool()
    assert True  # 无用断言

def test_write(self):
    pass  # 空测试
```

### 必须这样写
```python
# ✅ 正确示例
def test_read_file_success(self):
    """测试正常读取文件"""
    reader = ReadTool()
    with tempfile.NamedTemporaryFile(mode='w', delete=False) as f:
        f.write("test content")
        filepath = f.name
    
    try:
        result = reader.read(filepath)
        assert result.is_error is False
        assert "test content" in result.content
    finally:
        os.unlink(filepath)

def test_read_nonexistent_file_raises_error(self):
    """测试读取不存在的文件"""
    reader = ReadTool()
    with pytest.raises(ToolError):
        reader.read("/nonexistent/path/file.txt")
```

---

## 测试文件位置

```
python/tests/test_builtin.py
```

---

## 验收标准

1. 覆盖率 ≥ 80%
2. 所有工具均有成功/失败测试
3. 边界条件已覆盖
4. 无占位测试、无 pass、无 assert True
5. CI 通过

---

## 提交方式

```bash
git checkout -b test/builtin-coverage
git add python/tests/test_builtin.py
git commit -m "test: add comprehensive tests for builtin tools"
git push origin test/builtin-coverage
# 创建 PR，标题格式：test: builtin.py coverage to 80%
```
