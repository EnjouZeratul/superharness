# 场景：Git 完整工作流

> ID: scenario_git_workflow  
> 依赖: T2 P1.3 Git深度集成  
> 优先级: P0

---

## 目标

验证 Git 工作流完整可用：
1. git status 显示
2. git diff 分析
3. git commit 集成
4. git branch 管理
5. PR 创建和管理

---

## 前置条件

- [ ] Git 已安装并配置
- [ ] 项目为 Git 仓库
- [ ] Git 用户信息已配置
- [ ] 有远程仓库写入权限（PR 测试需要）

---

## 测试步骤

### 步骤 1：创建文件变更

**操作**：
```bash
# 添加新文件
echo "def hello(): return 'Hello'" > app.py

# 修改现有文件
echo "\n## New Feature" >> README.md
```

**验证点**：
- [ ] 文件成功创建
- [ ] 文件成功修改

### 步骤 2：Git Status 显示

**Agent 行为**：
1. 执行 `git status`
2. 显示变更文件列表
3. 按状态分类（新文件/修改/删除）

**预期输出**：
```
Changes:
  A  app.py (new file)
  M  README.md (modified)
```

**验证点**：
- [ ] 正确显示文件状态
- [ ] 状态图标正确
- [ ] 文件路径正确

### 步骤 3：Git Diff 分析

**Agent 行为**：
1. 执行 `git diff` 查看未暂存变更
2. 执行 `git diff --cached` 查看已暂存变更
3. 分析变更内容摘要

**预期输出**：
```
README.md:
  +1 addition
  -0 deletions
  Added: "## New Feature"

app.py:
  +3 additions (new file)
```

**验证点**：
- [ ] Diff 正确解析
- [ ] 变更统计准确
- [ ] 新文件正确标记

### 步骤 4：Git Commit 集成

**Agent 行为**：
1. 自动生成 commit 消息
2. 消息格式遵循规范
3. 执行 commit

**预期 Commit 消息**：
```
Add hello function and update README

- Add app.py with hello function
- Add new feature section to README
```

**验证点**：
- [ ] 消息自动生成
- [ ] 消息格式正确
- [ ] Commit 成功

### 步骤 5：Git Branch 管理

**Agent 行为**：
1. 创建新分支
2. 切换到新分支
3. 在新分支上提交

**操作序列**：
```bash
git checkout -b feature/hello-function
# 进行变更
git add .
git commit -m "Add hello function"
git checkout main
```

**验证点**：
- [ ] 分支创建成功
- [ ] 分支切换成功
- [ ] 分支列表正确

### 步骤 6：PR 创建

**Agent 行为**：
1. 推送分支到远程
2. 使用 `gh` 创建 PR
3. 生成 PR 标题和描述

**预期 PR 内容**：
```
Title: Add hello function

Description:
- Add new hello function in app.py
- Update README with new feature section

Test plan:
- [ ] Manual review
- [ ] Function test
```

**验证点**：
- [ ] PR 创建成功
- [ ] 标题描述准确
- [ ] 包含测试计划

---

## 成功标准

| 指标 | 预期值 |
|------|--------|
| Status 显示准确率 | 100% |
| Diff 解析准确率 | 100% |
| Commit 消息质量 | 良好 |
| 分支操作成功率 | 100% |
| PR 创建成功率 | 100% |

---

## 边界条件

### 边界 1：空仓库

**输入**：无任何变更  
**预期**：显示 "nothing to commit"

### 边界 2：大量变更

**输入**：100+ 文件变更  
**预期**：分页显示或摘要

### 边界 3：合并冲突

**输入**：存在合并冲突  
**预期**：正确识别并提示用户

### 边界 4：无权限

**输入**：无远程写入权限  
**预期**：本地操作成功，远程失败有明确提示

---

## 错误恢复场景

### 错误 1：Commit 失败

**触发**：pre-commit hook 失败  
**预期**：显示 hook 输出，提供修复建议

### 错误 2：Push 被拒绝

**触发**：远程有新提交  
**预期**：提示 pull 后重试

### 错误 3：PR 创建失败

**触发**：GitHub API 错误  
**预期**：显示错误，提供重试选项

---

## 检查清单

执行前确认：
- [ ] Git 已配置用户信息
- [ ] 有远程仓库访问权限
- [ ] GitHub CLI (gh) 已安装

执行后验证：
- [ ] 变更已提交
- [ ] 分支正确
- [ ] PR 已创建（如适用）
- [ ] 无遗留临时文件

---

*Continuum User Scenario - Git Workflow*