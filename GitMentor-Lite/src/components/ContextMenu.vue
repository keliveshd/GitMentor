<template>
  <div v-if="visible" ref="menuRef" class="context-menu" :style="menuStyle" @click.stop>
    <div class="context-menu-item" v-for="item in menuItems" :key="item.id"
      :class="{ disabled: item.disabled, separator: item.separator }" @click="handleItemClick(item)">
      <div v-if="item.separator" class="menu-separator"></div>
      <div v-else class="menu-item-content">
        <span class="menu-icon">{{ item.icon }}</span>
        <span class="menu-text">{{ item.text }}</span>
        <span v-if="item.shortcut" class="menu-shortcut">{{ item.shortcut }}</span>
      </div>
    </div>
  </div>
</template>

<script setup lang="ts">
import { ref, computed, onMounted, onUnmounted, nextTick, watch } from 'vue'

// 菜单项接口定义
export interface ContextMenuItem {
  id: string
  text: string
  icon: string
  action: string
  disabled?: boolean
  separator?: boolean
  shortcut?: string
}

// Props
interface Props {
  visible: boolean
  position: { x: number; y: number }
  menuItems: ContextMenuItem[]
}

const props = defineProps<Props>()

// Emits
const emit = defineEmits<{
  itemClick: [action: string]
  close: []
}>()

// 菜单DOM引用
const menuRef = ref<HTMLElement>()

// 响应式位置状态
const adjustedPosition = ref({ x: 0, y: 0 })

// 计算菜单位置，防止超出窗口
const menuStyle = computed(() => {
  if (!props.visible) return { display: 'none' }

  return {
    left: adjustedPosition.value.x + 'px',
    top: adjustedPosition.value.y + 'px'
  }
})

// 调整菜单位置的函数
const adjustMenuPosition = async () => {
  if (!props.visible || !menuRef.value) return

  await nextTick() // 确保DOM已更新

  let { x, y } = props.position
  const menuRect = menuRef.value.getBoundingClientRect()
  const windowWidth = window.innerWidth
  const windowHeight = window.innerHeight

  // 预估菜单尺寸（如果还没有实际尺寸）
  const menuWidth = menuRect.width || 180
  const menuHeight = menuRect.height || Math.min(props.menuItems.length * 32 + 8, 400)

  // 水平方向调整：如果菜单会超出右边界，向左偏移
  if (x + menuWidth > windowWidth) {
    x = Math.max(10, windowWidth - menuWidth - 10)
  }

  // 垂直方向调整：如果菜单会超出下边界，向上偏移
  if (y + menuHeight > windowHeight) {
    y = Math.max(10, windowHeight - menuHeight - 10)
  }

  // 确保不会超出左边界和上边界
  x = Math.max(10, x)
  y = Math.max(10, y)

  adjustedPosition.value = { x, y }
}

// 处理菜单项点击
const handleItemClick = (item: ContextMenuItem) => {
  if (item.disabled || item.separator) return

  emit('itemClick', item.action)
  emit('close')
}

// 处理点击外部关闭菜单
const handleClickOutside = (event: MouseEvent) => {
  if (props.visible) {
    emit('close')
  }
}

// 处理ESC键关闭菜单
const handleKeydown = (event: KeyboardEvent) => {
  if (event.key === 'Escape' && props.visible) {
    emit('close')
  }
}

// 监听菜单显示状态和位置变化
watch(() => props.visible, async (newVisible) => {
  if (newVisible) {
    // 初始设置位置
    adjustedPosition.value = props.position
    // 等待DOM渲染后调整位置
    await nextTick()
    adjustMenuPosition()
  }
})

watch(() => props.position, () => {
  if (props.visible) {
    adjustedPosition.value = props.position
    nextTick(() => adjustMenuPosition())
  }
}, { deep: true })

onMounted(() => {
  document.addEventListener('click', handleClickOutside)
  document.addEventListener('keydown', handleKeydown)
})

onUnmounted(() => {
  document.removeEventListener('click', handleClickOutside)
  document.removeEventListener('keydown', handleKeydown)
})
</script>

<style scoped>
.context-menu {
  position: fixed;
  background: white;
  border: 1px solid #e1e4e8;
  border-radius: 6px;
  box-shadow: 0 8px 24px rgba(0, 0, 0, 0.12);
  z-index: 9999;
  min-width: 180px;
  max-width: 280px;
  max-height: 400px;
  padding: 4px 0;
  font-size: 13px;
  user-select: none;
  overflow-y: auto;
  overflow-x: hidden;
}

.context-menu-item {
  cursor: pointer;
  transition: background-color 0.15s ease;
}

.context-menu-item:hover:not(.disabled):not(.separator) {
  background-color: #f6f8fa;
}

.context-menu-item.disabled {
  opacity: 0.5;
  cursor: not-allowed;
}

.context-menu-item.separator {
  cursor: default;
}

.menu-separator {
  height: 1px;
  background-color: #e1e4e8;
  margin: 4px 0;
}

.menu-item-content {
  display: flex;
  align-items: center;
  padding: 6px 12px;
  gap: 8px;
}

.menu-icon {
  font-size: 14px;
  width: 16px;
  text-align: center;
  flex-shrink: 0;
}

.menu-text {
  flex: 1;
  color: #24292f;
  font-weight: 400;
}

.menu-shortcut {
  color: #656d76;
  font-size: 11px;
  font-family: 'Consolas', 'Monaco', monospace;
}

.context-menu-item.disabled .menu-text {
  color: #8c959f;
}

/* 滚动条样式优化 */
.context-menu::-webkit-scrollbar {
  width: 6px;
}

.context-menu::-webkit-scrollbar-track {
  background: transparent;
}

.context-menu::-webkit-scrollbar-thumb {
  background: #d0d7de;
  border-radius: 3px;
}

.context-menu::-webkit-scrollbar-thumb:hover {
  background: #8c959f;
}
</style>
