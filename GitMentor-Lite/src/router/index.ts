import { createRouter, createWebHistory } from "vue-router";
import GitPanel from "../components/GitPanel.vue";
import DiffViewerPage from "../pages/DiffViewerPage.vue";
import AISettingsPage from "../pages/AISettingsPage.vue";
import TemplateConfigPage from "../pages/TemplateConfigPage.vue";

/**
 * 路由配置
 * 作者：Evilek
 * 编写日期：2025-07-23
 * 更新日期：2025-01-29 (添加根路由、AI设置页面路由和模板配置页面路由)
 */
const routes = [
  {
    path: "/",
    name: "Home",
    component: GitPanel,
  },
  {
    path: "/diff-viewer",
    name: "DiffViewer",
    component: DiffViewerPage,
    props: (route: any) => ({
      filePath: route.query.filePath as string,
      diffType: (route.query.diffType as string) || "WorkingTree",
    }),
  },
  {
    path: "/ai-settings",
    name: "AISettings",
    component: AISettingsPage,
  },
  {
    path: "/template-config",
    name: "TemplateConfig",
    component: TemplateConfigPage,
  },
];

const router = createRouter({
  history: createWebHistory(),
  routes,
});

export default router;
