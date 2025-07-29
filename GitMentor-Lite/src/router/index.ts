import { createRouter, createWebHistory } from "vue-router";
import GitPanel from "../components/GitPanel.vue";
import DiffViewerPage from "../pages/DiffViewerPage.vue";
import AISettingsPage from "../pages/AISettingsPage.vue";

/**
 * 路由配置
 * 作者：Evilek
 * 编写日期：2025-07-23
 * 更新日期：2025-01-29 (添加根路由和AI设置页面路由)
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
];

const router = createRouter({
  history: createWebHistory(),
  routes,
});

export default router;
