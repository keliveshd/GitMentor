#!/usr/bin/env node

/**
 * Git二进制文件设置脚本
 * 作者：Evilek
 * 编写日期：2025-08-06
 *
 * 用法：
 * node setup-git.cjs [git-path]
 *
 * 示例：
 * node setup-git.cjs                    # 使用系统Git
 * node setup-git.cjs /usr/bin/git       # 指定Git路径
 * node setup-git.cjs C:\Git\bin\git.exe # Windows指定路径
 */

const { execSync } = require("child_process");
const fs = require("fs");
const path = require("path");

console.log("🔧 GitMentor 内置Git设置脚本");
console.log("================================");

// 获取目标平台
function getTargetTriple() {
  try {
    const rustInfo = execSync("rustc -Vv", { encoding: "utf8" });
    const match = /host: (\S+)/g.exec(rustInfo);
    if (!match) {
      throw new Error("无法解析target triple");
    }
    return match[1];
  } catch (error) {
    console.error("❌ 错误：无法获取Rust target triple");
    console.error("请确保已安装Rust工具链");
    process.exit(1);
  }
}

// 检测Git路径
function detectGitPath() {
  const gitPaths =
    process.platform === "win32"
      ? [
          "git.exe",
          "C:\\Program Files\\Git\\bin\\git.exe",
          "C:\\Git\\bin\\git.exe",
        ]
      : ["git", "/usr/bin/git", "/usr/local/bin/git", "/opt/homebrew/bin/git"];

  for (const gitPath of gitPaths) {
    try {
      execSync(`"${gitPath}" --version`, { stdio: "ignore" });
      return gitPath;
    } catch (error) {
      // 继续尝试下一个路径
    }
  }

  throw new Error("未找到Git可执行文件");
}

// 主函数
function main() {
  try {
    // 获取Git路径
    const gitPath = process.argv[2] || detectGitPath();
    console.log(`📍 Git路径: ${gitPath}`);

    // 验证Git可用性
    try {
      const version = execSync(`"${gitPath}" --version`, {
        encoding: "utf8",
      }).trim();
      console.log(`✅ Git版本: ${version}`);
    } catch (error) {
      throw new Error(`Git不可用: ${gitPath}`);
    }

    // 获取目标平台
    const targetTriple = getTargetTriple();
    console.log(`🎯 目标平台: ${targetTriple}`);

    // 构建目标文件名
    const extension = process.platform === "win32" ? ".exe" : "";
    const targetFileName = `git-${targetTriple}${extension}`;
    const targetPath = path.join(__dirname, targetFileName);

    console.log(`📦 目标文件: ${targetFileName}`);

    // 复制文件
    if (fs.existsSync(targetPath)) {
      console.log("⚠️  目标文件已存在，将覆盖");
    }

    fs.copyFileSync(gitPath, targetPath);
    console.log("✅ 文件复制完成");

    // 设置执行权限（Unix系统）
    if (process.platform !== "win32") {
      try {
        execSync(`chmod +x "${targetPath}"`);
        console.log("✅ 执行权限设置完成");
      } catch (error) {
        console.warn("⚠️  警告：无法设置执行权限");
      }
    }

    // 验证目标文件
    try {
      const stats = fs.statSync(targetPath);
      const sizeMB = (stats.size / 1024 / 1024).toFixed(2);
      console.log(`📊 文件大小: ${sizeMB} MB`);
    } catch (error) {
      console.warn("⚠️  警告：无法获取文件信息");
    }

    console.log("");
    console.log("🎉 设置完成！");
    console.log("");
    console.log("📝 下一步：");
    console.log("1. 运行应用测试内置Git功能");
    console.log("2. 检查调试日志确认Git检测结果");
    console.log("3. 如需支持其他平台，请为每个平台重复此过程");
  } catch (error) {
    console.error(`❌ 错误: ${error.message}`);
    console.error("");
    console.error("💡 解决方案：");
    console.error("1. 确保已安装Git");
    console.error("2. 确保已安装Rust工具链");
    console.error("3. 手动指定Git路径：node setup-git.js /path/to/git");
    process.exit(1);
  }
}

// 显示帮助信息
if (process.argv.includes("--help") || process.argv.includes("-h")) {
  console.log("用法：node setup-git.js [git-path]");
  console.log("");
  console.log("参数：");
  console.log("  git-path    Git可执行文件路径（可选，默认自动检测）");
  console.log("");
  console.log("示例：");
  console.log("  node setup-git.js                    # 使用系统Git");
  console.log("  node setup-git.js /usr/bin/git       # 指定Git路径");
  console.log("  node setup-git.js C:\\Git\\bin\\git.exe # Windows指定路径");
  process.exit(0);
}

main();
