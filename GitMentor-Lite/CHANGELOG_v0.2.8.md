# 更新日志 v0.2.8 - Portable 版本更新机制

## 新功能

### 1. Portable ZIP 更新机制
- 新增 Portable ZIP 包构建和发布
- 避免 PowerShell 脚本被杀毒软件拦截
- 支持覆盖式更新

### 2. 文件锁定问题解决
实现了**三重保障机制**：
- **方案一：直接更新** - 优先尝试，无需重启
- **方案二：更新器进程** - 自动处理文件锁定
- **方案三：延迟更新** - 备用方案，下次启动应用

### 3. GitHub Actions 自动化
- 自动构建 MSI、ZIP 和 Portable ZIP
- 自动发布到 GitHub Release
- 自动生成 CHANGELOG

## 修改的文件

### 核心文件

1. **`src-tauri/tauri.conf.json`**
   - 添加 ZIP 到 bundle 目标列表
   - 配置便携版构建

2. **`src-tauri/Cargo.toml`**
   - 添加 `zip = "1.1"` 依赖
   - 用于 ZIP 文件处理

3. **`src-tauri/src/core/update_manager.rs`** ⭐ 核心修改
   - 新增 `find_windows_installer()` - 优先查找 portable ZIP
   - 新增 `install_update()` - 智能选择安装方式
   - 新增 `install_portable_zip()` - ZIP 覆盖更新
   - 新增 `install_with_updater_process()` - 更新器进程处理
   - 新增 `run_updater_process()` - 独立进程更新逻辑
   - 保留 MSI 安装逻辑（向后兼容）

4. **`src-tauri/src/commands/update_commands.rs`**
   - 修改默认文件名为 ZIP
   - 新增 `install_update_delayed()` - 延迟更新逻辑

5. **`src-tauri/src/lib.rs`** ⭐ 核心修改
   - 新增 `handle_updater_mode()` - 命令行参数处理
   - 新增 `handle_pending_updates()` - 启动时检查待更新
   - 修改 `run()` 函数集成更新检查

### 构建脚本

6. **`scripts/build-portable.ps1`** ⭐ 新文件
   - 本地构建便携版脚本
   - 自动复制文件、创建启动脚本
   - 生成 ZIP 包

7. **`.github/workflows/release.yml`** ⭐ 新文件
   - GitHub Actions 自动构建流程
   - 构建 MSI、ZIP 和 Portable ZIP
   - 自动上传到 Release
   - 生成 CHANGELOG

8. **`package.json`**
   - 添加 `tauri:build:portable` 命令

### 文档

9. **`PORTABLE_UPDATE_README.md`** ⭐ 新文档
   - Portable 版本更新机制说明
   - 构建和使用指南
   - 常见问题解答

10. **`FILE_LOCK_SOLUTION_README.md`** ⭐ 新文档
    - 文件锁定问题详细解决方案
    - 三重保障机制说明
    - 技术实现细节

## 技术实现亮点

### 1. 智能更新流程
```
用户点击更新
    ↓
下载 Portable ZIP
    ↓
尝试直接更新（无需重启）
    ↓ [失败]
启动更新器进程（自动重启）
    ↓ [也失败]
标记延迟更新（下次启动）
    ↓
完成！
```

### 2. 更新器进程机制
```
主程序：
  检测到文件锁定
  → 启动: GitMentorLite.exe updater --installer xxx.zip
  → 退出

更新器进程：
  等待主程序完全退出
  → 解压 ZIP 覆盖文件
  → 重新启动应用
  → 退出
```

### 3. 延迟更新机制
```
更新失败时：
  → 创建 pending-update/ 目录
  → 复制 ZIP 文件
  → 创建 .update-pending 标记

下次启动时：
  → 检测标记文件
  → 应用更新
  → 清理临时文件
```

## 优势

✅ **避免杀毒软件拦截** - 不使用 PowerShell
✅ **解决文件锁定** - 三重保障机制
✅ **零更新失败** - 永远有备用方案
✅ **向后兼容** - 保留 MSI 安装
✅ **自动化构建** - GitHub Actions 全自动
✅ **详细文档** - 完整的说明和故障排除

## 兼容性

- ✅ Windows 10/11
- ✅ 现有 MSI 安装用户
- ✅ Portable 版本用户
- ✅ 企业批量部署

## 测试建议

1. **本地测试**
   ```bash
   npm run tauri:build:portable
   ```

2. **更新测试**
   - 测试直接更新
   - 测试更新器进程
   - 测试延迟更新

3. **权限测试**
   - 普通权限更新
   - 管理员权限更新

## 下一步计划

- [ ] 添加自动增量更新
- [ ] 支持回滚功能 UI
- [ ] 添加更新进度显示
- [ ] 支持差分更新

---

**贡献者**: Evilek
**版本**: v0.2.8
**日期**: 2025-01-XX
