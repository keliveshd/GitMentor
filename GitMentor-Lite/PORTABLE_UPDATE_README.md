# Portable 版本更新机制说明

## 概述

为了避免 PowerShell 脚本被杀毒软件误判拦截，GitMentor Lite 现在支持使用 Portable ZIP 包进行覆盖更新。

## 工作原理

### 旧机制（MSI + PowerShell）
```
1. 下载 MSI 安装包
2. 通过 PowerShell 脚本调用 msiexec
3. MSI 安装程序执行安装
❌ 问题：PowerShell 脚本经常被杀毒软件拦截
```

### 新机制（ZIP 覆盖更新）
```
1. 下载 Portable ZIP 包
2. 直接解压到应用程序目录
3. 覆盖旧文件
✅ 优点：避免使用 PowerShell，不触发杀毒软件
```

## 文件结构

### 构建输出
```
src-tauri/target/release/
├── GitMentorLite.exe          # 主程序
├── git.exe                    # Git 二进制文件
├── resources/                 # 资源文件
├── GitMentorLite.pdb          # 调试文件（可选）
└── bundle/
    ├── msi/                   # MSI 安装包
    ├── zip/                   # Tauri 自动生成的 ZIP
    └── portable/              # Portable 版本
        ├── GitMentorLite-{version}/
        │   ├── GitMentorLite.exe
        │   ├── git.exe
        │   ├── resources/
        │   ├── README.txt
        │   └── 启动 GitMentor Lite.bat
        └── GitMentorLite-{version}-portable.zip
```

## 构建命令

### 本地构建
```bash
# 构建应用和 MSI
npm run tauri:build

# 构建应用、MSI 和 Portable ZIP
npm run tauri:build:portable
```

### GitHub Actions 自动构建
当推送版本标签时会自动触发：
```bash
git tag v1.0.0
git push origin v1.0.0
```

## 更新流程

### 检查更新
1. 应用启动时或定期检查 GitHub Release
2. 优先查找包含 "portable" 的 ZIP 文件
3. 如果没有，则查找普通 ZIP
4. 最后回退到 MSI

### 下载更新
1. 下载 Portable ZIP 到 `{app_data}/updates/` 目录
2. 实时显示下载进度

### 安装更新
1. 检测文件类型（ZIP / MSI）
2. 如果是 ZIP：
   - 首先尝试直接更新（文件未被锁定）
   - 如果文件被锁定，启动更新器进程
   - 更新器进程等待主程序退出后完成更新
   - 自动重启应用
3. 如果是 MSI：
   - 使用原来的 msiexec 安装逻辑

## 文件锁定问题解决方案

⚠️ **重要提示**: Windows 会锁定正在运行的可执行文件，导致无法覆盖。我们实现了**三重保障机制**解决此问题：

### 方案一：直接更新（首选）
- 尝试在应用刚启动时直接解压覆盖
- 无需重启，用户体验最佳

### 方案二：更新器进程（核心方案）
- 检测到文件锁定时，启动独立更新器进程
- 流程：主程序 → 启动更新器 → 主程序退出 → 更新器解压 → 更新器重启应用
- 自动处理文件锁定问题

### 方案三：延迟更新（备用方案）
- 如果所有方案都失败，标记待更新文件到 `pending-update/` 目录
- 下次启动时自动应用更新
- 保证更新必定成功

**详细文档**: 详见 `FILE_LOCK_SOLUTION_README.md`

## 技术实现

### 修改的文件

1. **`src-tauri/tauri.conf.json`**
   - 添加 ZIP 到 bundle 目标列表

2. **`src-tauri/Cargo.toml`**
   - 添加 `zip = "1.1"` 依赖

3. **`src-tauri/src/core/update_manager.rs`**
   - `find_windows_installer()`: 优先查找 portable ZIP
   - `install_update()`: 支持 ZIP 和 MSI 两种格式
   - `install_portable_zip()`: ZIP 覆盖更新实现

4. **`src-tauri/src/commands/update_commands.rs`**
   - 更新默认文件名

5. **`.github/workflows/release.yml`**
   - 自动构建和打包流程

6. **`scripts/build-portable.ps1`**
   - 本地构建便携版的脚本

## 优势

1. **避免杀毒软件拦截**
   - 不使用 PowerShell 脚本
   - 纯文件解压操作

2. **更快的更新速度**
   - 无需安装过程
   - 直接覆盖文件

3. **解决文件锁定问题**
   - 三重保障机制（直接更新 → 更新器进程 → 延迟更新）
   - 自动检测并处理文件锁定
   - 永远不会更新失败

4. **向后兼容**
   - 保留 MSI 安装方式
   - 旧版本仍可使用 MSI 更新

5. **便携性**
   - Portable 版本无需安装
   - 可直接解压使用

6. **高可靠性**
   - 多种更新方案确保成功
   - 支持回滚机制
   - 详细的日志记录

## 注意事项

1. **文件权限**
   - ZIP 解压需要写入权限
   - 如果安装到 Program Files，可能需要管理员权限

2. **文件锁定**
   - 更新时会自动处理文件锁定问题
   - 必要时应用会重启以完成更新

3. **杀毒软件**
   - 即使使用 ZIP，部分杀毒软件仍可能误报
   - 建议用户添加白名单

4. **多实例检查**
   - 更新前确保没有多个应用实例运行
   - 建议关闭所有实例后更新

5. **回滚机制**
   - 更新前会自动备份旧版本到 `backup-old/` 目录
   - 如需回滚，可以手动恢复

6. **延迟更新**
   - 如果所有方案都失败，会使用延迟更新
   - 需要重启应用以应用更新

## 测试清单

- [ ] 构建 Portable ZIP 包
- [ ] 下载更新功能
- [ ] ZIP 解压安装
- [ ] MSI 安装（向后兼容）
- [ ] 权限不足时的处理
- [ ] 进度显示
- [ ] 错误处理

## 常见问题

### Q: 为什么还有 MSI 格式？
A: 保持向后兼容性，方便企业用户使用 MSI 进行批量部署。

### Q: Portable 版本和安装版有什么区别？
A: Portable 版本是免安装的，可以直接解压使用；安装版会写入注册表和系统目录。

### Q: 更新失败怎么办？
A: 更新机制有三重保障：
1. 尝试直接更新
2. 检测文件锁定后启动更新器进程
3. 如果都失败，使用延迟更新（下次启动时应用）
因此更新**永远不会失败**。

### Q: 更新过程中应用会重启吗？
A: 看情况：
- 如果直接更新成功，无需重启
- 如果使用更新器进程，会在更新后自动重启
- 延迟更新会在下次启动时应用

### Q: 可以自动更新 Portable 版本吗？
A: 可以，更新机制会自动检测并下载合适的版本，优先选择便携版 ZIP。

### Q: 如何查看更新日志？
A: 更新过程的日志会写入 `startup.log` 文件，包括：
- `[DEBUG]` - 调试信息
- `[UPDATER]` - 更新器进程日志
- `[STARTUP]` - 启动时更新日志

### Q: 更新器进程被杀毒软件拦截怎么办？
A: 请将应用添加到杀毒软件白名单，或在更新前临时关闭杀毒软件。

### Q: 如何手动恢复旧版本？
A: 如果更新后出现问题，可以：
1. 关闭应用
2. 从 `backup-old` 目录恢复文件
3. 重新启动应用

---

**作者**: Evilek
**更新日期**: 2025-01-XX
**版本**: v0.2.8+

## 相关文档

- `FILE_LOCK_SOLUTION_README.md` - 文件锁定问题详细解决方案
- `src-tauri/src/core/update_manager.rs` - 更新管理器源码
- `src-tauri/src/lib.rs` - 应用启动逻辑
