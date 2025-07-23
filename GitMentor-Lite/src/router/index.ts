import { createRouter, createWebHistory } from "vue-router";
import DiffViewerPage from "../pages/DiffViewerPage.vue";

/**
 * 路由配置
 * 作者：Evilek
 * 编写日期：2025-07-23
 */
const routes = [
  {
    path: "/diff-viewer",
    name: "DiffViewer",
    component: DiffViewerPage,
    props: (route: any) => ({
      filePath: route.query.filePath as string,
      diffType: (route.query.diffType as string) || "WorkingTree",
    }),
  },
];

const router = createRouter({
  history: createWebHistory(),
  routes,
});

export default router;
