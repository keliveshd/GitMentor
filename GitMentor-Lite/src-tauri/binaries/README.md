# 内置Git二进制文件配置

## 概述

GitMentor支持三种Git执行方式：
1. **SystemGit** - 使用系统安装的Git命令（优先级最高）
2. **BundledGit** - 使用应用内置的Git可执行文件（备选方案）
3. **Git2Api** - 使用Git2库API（最后备选）

## 内置Git设置

### 1. 获取Git可执行文件

你需要为每个目标平台准备对应的Git可执行文件：

**Windows平台：**
- 从 [Git for Windows](https://git-scm.com/download/win) 下载便携版
- 提取 `git.exe` 文件

**Linux平台：**
- 使用包管理器安装Git：`sudo apt install git` 或 `sudo yum install git`
- 复制 `/usr/bin/git` 文件

**macOS平台：**
- 使用Homebrew安装：`brew install git`
- 复制 `/usr/local/bin/git` 或 `/opt/homebrew/bin/git` 文件

### 2. 重命名二进制文件

将Git可执行文件按照目标平台重命名并放置在此目录下：

```
src-tauri/binaries/
├── git-x86_64-pc-windows-msvc.exe     # Windows 64位
├── git-i686-pc-windows-msvc.exe       # Windows 32位
├── git-x86_64-unknown-linux-gnu       # Linux 64位
├── git-aarch64-unknown-linux-gnu      # Linux ARM64
├── git-x86_64-apple-darwin            # macOS Intel
└── git-aarch64-apple-darwin           # macOS Apple Silicon
```

### 3. 获取当前平台的Target Triple

运行以下命令获取当前平台的target triple：

```bash
# 获取完整信息
rustc -Vv

# 仅获取target triple（Unix系统）
rustc -Vv | grep host | cut -f2 -d' '

# 仅获取target triple（Windows PowerShell）
rustc -Vv | Select-String "host:" | ForEach-Object {$_.Line.split(" ")[1]}
```

### 4. 设置文件权限（Unix系统）

确保Git二进制文件具有执行权限：

```bash
chmod +x src-tauri/binaries/git-*
```

## 工作原理

1. **检测阶段**：GitEngine会按优先级检测可用的Git执行方式
2. **系统Git优先**：如果系统安装了Git，优先使用系统Git
3. **内置Git备选**：如果系统没有Git，使用内置的Git可执行文件
4. **API降级**：如果都不可用，降级到Git2库API（功能有限）

## 注意事项

- 内置Git文件会增加应用包的大小（通常20-50MB）
- 确保Git版本兼容性（建议使用2.30+版本）
- 在生产环境中，Tauri会自动处理sidecar的路径解析
- 开发环境中，函数会检查 `src-tauri/binaries/` 目录

## 调试

启用调试日志查看Git检测过程：

```rust
debug_log!("[DEBUG] 检测Git执行方式...");
debug_log!("[DEBUG] 当前目标平台: {}", target_triple);
debug_log!("[DEBUG] 找到内置Git: {}", binary_path.display());
```

## 常见问题

**Q: 为什么需要内置Git？**
A: 确保应用在没有安装Git的系统上也能正常工作。

**Q: 如何减小应用包大小？**
A: 只为目标平台提供对应的Git二进制文件，不需要包含所有平台的版本。

**Q: 内置Git和系统Git有什么区别？**
A: 功能完全相同，只是调用方式不同。内置Git通过Tauri的sidecar机制调用。
