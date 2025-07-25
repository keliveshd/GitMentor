import { WebviewWindow } from "@tauri-apps/api/webviewWindow";

/**
 * 窗口管理工具类
 * 作者：Evilek
 * 编写日期：2025-07-23
 */
export class WindowManager {
  private static openWindows = new Map<string, WebviewWindow>();

  /**
   * 打开差异查看器窗口
   * @param filePath 文件路径
   * @param diffType 差异类型
   */
  static async openDiffViewer(
    filePath: string,
    diffType: "WorkingTree" | "Staged" | "HeadToWorking" = "WorkingTree"
  ) {
    try {
      console.log(`🚀 [WindowManager] 开始创建差异查看器窗口:`, {
        filePath,
        diffType,
      });

      // 生成唯一的窗口标识 - 使用安全的字符替换
      const safeFilePath = filePath
        .replace(/[^a-zA-Z0-9\-\/_:]/g, "_") // 替换不安全字符为下划线
        .replace(/_{2,}/g, "_"); // 合并多个连续下划线
      const windowLabel = `diff-viewer-${safeFilePath}-${diffType}`;
      console.log(`🏷️ [WindowManager] 窗口标识: ${windowLabel}`);

      // 检查窗口是否已经存在
      if (this.openWindows.has(windowLabel)) {
        const existingWindow = this.openWindows.get(windowLabel);
        if (existingWindow) {
          console.log(
            `♻️ [WindowManager] 窗口已存在，聚焦到现有窗口: ${windowLabel}`
          );
          // 聚焦到已存在的窗口
          await existingWindow.setFocus();
          return existingWindow;
        }
      }

      // 构建URL参数
      const params = new URLSearchParams({
        filePath,
        diffType,
      });
      const fullUrl = `/diff-viewer?${params.toString()}`;
      console.log(`🔗 [WindowManager] 构建的URL: ${fullUrl}`);

      // 创建新窗口
      const window = new WebviewWindow(windowLabel, {
        url: fullUrl,
        title: `差异查看 - ${this.getFileName(filePath)}`,
        width: 1200,
        height: 800,
        center: true,
        resizable: true,
        minimizable: true,
        maximizable: true,
        closable: true,
        skipTaskbar: false,
        alwaysOnTop: false,
      });

      console.log(
        `🪟 [WindowManager] WebviewWindow 实例已创建: ${windowLabel}`
      );

      // 监听窗口关闭事件
      window.once("tauri://close-requested", () => {
        this.openWindows.delete(windowLabel);
      });

      // 监听窗口创建完成事件
      window.once("tauri://created", () => {
        console.log(
          `✅ [WindowManager] 差异查看器窗口创建成功: ${windowLabel}`
        );
      });

      // 监听窗口错误事件
      window.once("tauri://error", (error) => {
        console.error(`❌ [WindowManager] 窗口创建失败: ${windowLabel}`, error);
        this.openWindows.delete(windowLabel);
      });

      // 保存窗口引用
      this.openWindows.set(windowLabel, window);

      return window;
    } catch (error) {
      console.error("❌ [WindowManager] 创建差异查看器窗口失败:", error);
      throw error;
    }
  }

  /**
   * 关闭指定的差异查看器窗口
   * @param filePath 文件路径
   * @param diffType 差异类型
   */
  static async closeDiffViewer(
    filePath: string,
    diffType: "WorkingTree" | "Staged" | "HeadToWorking" = "WorkingTree"
  ) {
    // 生成安全的窗口标识 - 与openDiffViewer保持一致
    const safeFilePath = filePath
      .replace(/[^a-zA-Z0-9\-\/_:]/g, "_") // 替换不安全字符为下划线
      .replace(/_{2,}/g, "_"); // 合并多个连续下划线
    const windowLabel = `diff-viewer-${safeFilePath}-${diffType}`;
    const window = this.openWindows.get(windowLabel);

    if (window) {
      try {
        await window.close();
        this.openWindows.delete(windowLabel);
        console.log(`✅ [WindowManager] 窗口已关闭: ${windowLabel}`);
      } catch (error) {
        console.error(`❌ [WindowManager] 关闭窗口失败: ${windowLabel}`, error);
      }
    }
  }

  /**
   * 关闭所有差异查看器窗口
   */
  static async closeAllDiffViewers() {
    const promises = Array.from(this.openWindows.entries()).map(
      async ([label, window]) => {
        if (label.startsWith("diff-viewer-")) {
          try {
            await window.close();
            this.openWindows.delete(label);
          } catch (error) {
            console.error(`❌ [WindowManager] 关闭窗口失败: ${label}`, error);
          }
        }
      }
    );

    await Promise.all(promises);
    console.log("✅ [WindowManager] 所有差异查看器窗口已关闭");
  }

  /**
   * 获取当前打开的差异查看器窗口数量
   */
  static getDiffViewerCount(): number {
    return Array.from(this.openWindows.keys()).filter((label) =>
      label.startsWith("diff-viewer-")
    ).length;
  }

  /**
   * 打开AI服务设置窗口
   * 作者：Evilek
   * 编写日期：2025-07-25
   */
  static async openAISettings() {
    try {
      console.log(`🚀 [WindowManager] 开始创建AI服务设置窗口`);

      const windowLabel = "ai-settings";

      // 检查窗口是否已经存在
      if (this.openWindows.has(windowLabel)) {
        const existingWindow = this.openWindows.get(windowLabel);
        if (existingWindow) {
          console.log(`♻️ [WindowManager] AI设置窗口已存在，聚焦到现有窗口`);
          // 聚焦到已存在的窗口
          await existingWindow.setFocus();
          return existingWindow;
        }
      }

      // 构建URL
      const fullUrl = "/ai-settings";
      console.log(`🔗 [WindowManager] 构建的URL: ${fullUrl}`);

      // 创建新窗口
      const window = new WebviewWindow(windowLabel, {
        url: fullUrl,
        title: "AI服务设置",
        width: 800,
        height: 600,
        center: true,
        resizable: true,
        minimizable: true,
        maximizable: true,
        closable: true,
        skipTaskbar: false,
        alwaysOnTop: false,
      });

      console.log(`🪟 [WindowManager] AI设置窗口实例已创建`);

      // 监听窗口关闭事件
      window.once("tauri://close-requested", () => {
        this.openWindows.delete(windowLabel);
      });

      // 监听窗口创建完成事件
      window.once("tauri://created", () => {
        console.log(`✅ [WindowManager] AI设置窗口创建成功`);
      });

      // 监听窗口错误事件
      window.once("tauri://error", (error) => {
        console.error(`❌ [WindowManager] AI设置窗口创建失败`, error);
        this.openWindows.delete(windowLabel);
      });

      // 保存窗口引用
      this.openWindows.set(windowLabel, window);

      return window;
    } catch (error) {
      console.error("❌ [WindowManager] 创建AI设置窗口失败:", error);
      throw error;
    }
  }

  /**
   * 关闭AI服务设置窗口
   * 作者：Evilek
   * 编写日期：2025-07-25
   */
  static async closeAISettings() {
    const windowLabel = "ai-settings";
    const window = this.openWindows.get(windowLabel);

    if (window) {
      try {
        await window.close();
        this.openWindows.delete(windowLabel);
        console.log(`✅ [WindowManager] AI设置窗口已关闭`);
      } catch (error) {
        console.error(`❌ [WindowManager] 关闭AI设置窗口失败`, error);
      }
    }
  }

  /**
   * 从文件路径提取文件名
   * @param filePath 文件路径
   */
  private static getFileName(filePath: string): string {
    return filePath.split(/[/\\]/).pop() || filePath;
  }
}

export default WindowManager;
