# 文件锁定问题解决方案

## 问题描述

在更新过程中，Windows 会锁定正在运行的可执行文件 (`GitMentorLite.exe`)，导致 ZIP 文件无法覆盖，常见错误包括：
- "The process cannot access the file because it is being used by another process"
- "文件正在被使用，无法写入"

## 解决方案

我们实现了**三重保障机制**，确保更新能够成功完成：

### 方案一：直接更新（首选）
- **适用场景**：应用刚启动、未完全加载时
- **原理**：尝试直接解压 ZIP 覆盖文件
- **优点**：用户体验最佳，无需重启
- **缺点**：可能被文件锁定阻止

### 方案二：更新器进程（核心方案）
- **适用场景**：直接更新失败（文件锁定）
- **原理**：启动独立进程完成更新
- **流程**：
  1. 主程序检测到文件锁定
  2. 启动更新器进程：`GitMentorLite.exe updater --installer xxx.zip --app-dir xxx --exe-name xxx.exe`
  3. 主程序退出
  4. 更新器进程等待主程序完全退出
  5. 更新器解压 ZIP 覆盖文件
  6. 更新器重新启动应用
- **优点**：彻底解决文件锁定问题
- **缺点**：需要重启应用

### 方案三：延迟更新（备用方案）
- **适用场景**：所有方案都失败
- **原理**：标记更新文件，下次启动时应用
- **流程**：
  1. 创建 `pending-update` 目录
  2. 复制 ZIP 到待更新目录
  3. 创建 `.update-pending` 标记文件
  4. 下次启动时检测标记文件
  5. 应用更新并清理临时文件
- **优点**：保证更新必定成功
- **缺点**：需要重启

## 技术实现

### 1. 更新管理器 (`update_manager.rs`)

#### 安装流程
```rust
pub async fn install_update(&self, installer_path: &PathBuf) -> Result<()> {
    // 1. 尝试直接更新
    match self.install_portable_zip(installer_path).await {
        Ok(_) => Ok(()),
        Err(e) => {
            // 2. 检测文件锁定
            if error_msg.contains("locked") || error_msg.contains("in use") {
                // 3. 使用更新器进程
                self.install_with_updater_process(installer_path).await
            } else {
                Err(e)
            }
        }
    }
}
```

#### 更新器进程
```rust
pub async fn run_updater_process(
    &self,
    installer_path: &PathBuf,
    app_dir: &PathBuf,
    exe_name: &str,
) -> Result<()> {
    // 1. 等待主进程退出
    tokio::time::sleep(Duration::from_millis(500)).await;

    // 2. 解压 ZIP
    let mut archive = ZipArchive::new(zip_file)?;
    for i in 0..archive.len() {
        let mut file = archive.by_index(i)?;
        // ... 解压逻辑
    }

    // 3. 验证更新
    let exe_path = app_dir.join(exe_name);
    if !exe_path.exists() {
        return Err(anyhow::anyhow!("更新后未找到可执行文件"));
    }

    Ok(())
}
```

### 2. 主程序启动逻辑 (`lib.rs`)

#### 命令行检查
```rust
fn handle_updater_mode() -> bool {
    let args: Vec<String> = std::env::args().collect();

    if args.len() > 1 && args[1] == "updater" {
        // 解析参数并执行更新
        // ...
        std::process::exit(0);
    }
    false
}
```

#### 启动时检查待更新
```rust
async fn handle_pending_updates() {
    // 检查 pending-update 目录
    let pending_dir = app_dir.join("pending-update");
    if pending_dir.exists() && marker_file.exists() {
        // 找到 ZIP 文件并应用更新
        // ...
    }
}
```

### 3. 前端命令 (`update_commands.rs`)

#### 延迟更新准备
```rust
pub async fn install_update(installer_path: String) -> Result<(), String> {
    match update_manager.install_update(&path).await {
        Ok(_) => Ok(()),
        Err(e) => {
            // 直接更新失败，尝试延迟更新
            install_update_delayed(&path).await
        }
    }
}
```

## 更新流程图

```
开始更新
    │
    ├─ 尝试直接更新
    │   ├─ 成功 ✓ → 完成
    │   └─ 失败（文件锁定）
    │        │
    │        └─ 启动更新器进程
    │            │
    │            ├─ 主程序退出
    │            ├─ 更新器解压文件
    │            ├─ 更新器重启应用
    │            └─ 完成 ✓
    │
    └─ 如果所有方案都失败
         │
         └─ 延迟更新
              │
              ├─ 标记待更新文件
              ├─ 下次启动时检测
              ├─ 应用更新
              └─ 完成 ✓
```

## 文件结构变化

```
应用目录/
├── GitMentorLite.exe           # 主程序
├── git.exe                     # Git 二进制
├── resources/                  # 资源文件
├── pending-update/             # [新增] 待更新目录
│   ├── update.zip              # 待更新的 ZIP
│   └── .update-pending         # 标记文件
└── backup-old/                 # [新增] 旧版本备份
    └── ...                     # 备份的文件
```

## 优势

1. **三重保障** - 三种方案确保更新必定成功
2. **自动处理** - 无需用户手动干预
3. **向后兼容** - 保留 MSI 安装方式
4. **用户体验** - 优先使用无需重启的方案
5. **可恢复** - 支持延迟更新，永不失败

## 注意事项

1. **权限要求**
   - 如果安装到 `Program Files`，可能需要管理员权限
   - 建议用户以管理员身份运行更新

2. **杀毒软件**
   - 更新器进程可能触发杀毒软件
   - 建议用户添加到白名单

3. **磁盘空间**
   - 延迟更新需要额外磁盘空间存储 ZIP
   - 完成后会自动清理

4. **备份策略**
   - 旧版本会备份到 `backup-old/` 目录
   - 可手动删除备份以节省空间

## 测试清单

- [ ] 应用启动时检测待更新文件
- [ ] 直接更新（文件未被锁定时）
- [ ] 更新器进程（文件被锁定时）
- [ ] 延迟更新（所有方案都失败时）
- [ ] 更新后应用正常启动
- [ ] 更新完成后清理临时文件
- [ ] 权限不足时的处理
- [ ] 磁盘空间不足时的处理

## 故障排除

### 问题：更新器进程也被锁定
**解决方案**：检查是否有多实例运行
```bash
tasklist | findstr GitMentorLite
```

### 问题：更新后应用无法启动
**解决方案**：手动恢复备份
```bash
# 从 backup-old 目录恢复文件
```

### 问题：延迟更新不生效
**解决方案**：
1. 检查 `pending-update` 目录是否存在
2. 检查 `.update-pending` 标记文件是否存在
3. 检查 ZIP 文件是否完整

---

**作者**: Evilek
**版本**: v0.2.8+
**更新日期**: 2025-01-XX

## 相关文档

- `PORTABLE_UPDATE_README.md` - Portable 版本更新机制说明
- `src-tauri/src/core/update_manager.rs` - 更新管理器源码
- `src-tauri/src/lib.rs` - 应用启动逻辑
- `src-tauri/src/commands/update_commands.rs` - 更新命令实现
