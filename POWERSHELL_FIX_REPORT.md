# PowerShell 脚本语法错误修复报告

## 问题概述

GitHub Actions 的 release 工作流在创建便携版 ZIP 时失败，错误信息如下：

```
ParserError: D:\a\_temp\9497ca64-e0c2-4787-bce0-de086c701055.ps1:42
```

## 根本原因

**错误位置**: `.github/workflows/release.yml` 第 128 行

**问题代码**:
```powershell
$readme += [char]10 + [char]10 + "使用方法：双击 `启动 GitMentor Lite.bat`"
```

**问题分析**:
- 反引号（`）在 PowerShell 中是转义字符（Escape Character）
- 当字符串中包含反引号时，PowerShell 解析器会将其解释为转义指令
- 这导致后续代码无法正确解析，出现语法错误

## 解决方案

将反引号替换为单引号：

**修复后代码**:
```powershell
$readme += [char]10 + [char]10 + "使用方法：双击 '启动 GitMentor Lite.bat'"
```

**为什么这个修复有效**:
- 单引号字符串在 PowerShell 中不会解析转义序列
- 单引号和双引号在中文语境下都可以表示引用
- 保持了原始文本的含义和格式

## 其他潜在解决方案

除了替换为单引号，还可以考虑以下方案：

1. **使用 `-replace 操作符替换反引号**:
   ```powershell
   $readme += [char]10 + [char]10 + ("使用方法：双击 `启动 GitMentor Lite.bat`" -replace '`', "''")
   ```

2. **使用 here-string 避免转义**:
   ```powershell
   $readme = @"
   # GitMentor Lite Portable v$Version

   使用方法：双击 '启动 GitMentor Lite.bat'
   "@
   ```

3. **使用不同的标点符号**:
   ```powershell
   $readme += [char]10 + [char]10 + "使用方法：双击『启动 GitMentor Lite.bat』"
   ```

## 验证步骤

修复后，GitHub Actions 应该能够：

1. ✅ 成功创建便携版目录
2. ✅ 复制可执行文件和资源文件
3. ✅ 创建启动脚本
4. ✅ 创建 README.txt 文件（不再出现语法错误）
5. ✅ 成功生成 ZIP 包
6. ✅ 将构建产物上传到 GitHub Release

## 建议

1. **代码审查**: 在 PowerShell 脚本中避免使用反引号作为普通字符
2. **测试**: 在本地或测试环境中验证 PowerShell 脚本
3. **文档**: 在项目文档中说明 PowerShell 脚本的编写规范

## 影响范围

- ✅ 仅影响 GitHub Actions 的 release 工作流
- ✅ 不影响本地开发流程
- ✅ 不影响其他构建步骤
- ✅ 修复后应能成功生成 v0.4.16 及后续版本的便携版 ZIP

---

## 第二次修复：Here-Document 语法错误

**发现问题**: 2025-11-16 19:45
**错误位置**: `.github/workflows/release.yml` 第 199 行
**错误类型**: Shell 脚本语法错误

### 问题概述

在第二次构建时，第二个 job（create-changelog）失败，错误信息：

```
/home/runner/work/_temp/0b12bad6-9c7d-4f09-8f43-94010209fca4.sh: line 27: warning: here-document at line 11 delimited by end-of-file (wanted `EOF')
/home/runner/work/_temp/0b12bad6-9c7d-4f09-8f43-94010209fca4.sh: line 28: syntax error: unexpected end of file
```

### 根本原因

在 shell 脚本的 here-document (`cat << EOF`) 中，EOF 结束标记前面有前导空格，导致解析失败。

### 解决方案

移除 CHANGELOG 内容中所有行的前导空格：

**修复前**（第 183-199 行）:
```yaml
cat > CHANGELOG.md << EOF
            # Changelog
            ...
            EOF
```

**修复后**:
```yaml
cat > CHANGELOG.md << EOF
# Changelog
...
EOF
```

### 验证

修复后，GitHub Actions 应该能够：
- ✅ 成功生成 CHANGELOG.md 文件
- ✅ 完成 create-changelog job
- ✅ 完整执行所有构建和发布步骤

---

## 第三次修复：YAML 变量语法错误

**发现问题**: 2025-11-16 19:48
**错误位置**: `.github/workflows/release.yml` 第 187 行
**错误类型**: YAML 语法错误

### 问题概述

GitHub Actions 工作流验证失败：

```
Invalid workflow file: .github/workflows/release.yml#L187
You have an error in your yaml syntax on line 187
```

### 根本原因

在 YAML 多行字符串（here-document）中直接使用 GitHub Actions 变量语法 `${{ github.ref_name }}` 导致 YAML 解析器语法错误。YAML 解析器无法正确处理 `${{` 序列，认为这是无效语法。

### 解决方案

将 GitHub Actions 变量和 shell 命令替换为 shell 变量：

**修复前**:
```yaml
cat > CHANGELOG.md << EOF
## [${{ github.ref_name }}] - $(date +%Y-%m-%d)
EOF
```

**修复后**:
```yaml
VERSION="${{ github.ref_name }}"
DATE=$(date +%Y-%m-%d)
cat > CHANGELOG.md << EOF
## [$VERSION] - $DATE
EOF
```

### 执行流程

1. GitHub Actions 先替换 `${{ github.ref_name }}` 为实际版本号（如 "v0.4.17"）
2. Shell 执行脚本，设置 `VERSION="v0.4.17"` 和 `DATE="2025-11-16"`
3. Here-document 中引用 `$VERSION` 和 `$DATE` 变量

### 验证

修复后，GitHub Actions 应该能够：
- ✅ 通过 YAML 语法验证
- ✅ 成功执行脚本
- ✅ 正确生成包含版本号和日期的 CHANGELOG.md

---

**修复时间**: 2025-11-16 19:48
**修复文件**: `.github/workflows/release.yml`
**修复行数**: 第 183-191 行（变量定义和引用）
**总修复次数**: 3 次
