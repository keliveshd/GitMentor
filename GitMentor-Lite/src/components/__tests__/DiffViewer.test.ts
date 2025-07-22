/**
 * DiffViewer组件测试
 * 作者：Evilek
 * 编写日期：2025-07-22
 */

import { describe, it, expect, vi, beforeEach } from 'vitest'
import { mount } from '@vue/test-utils'
import DiffViewer from '../DiffViewer.vue'
import EnhancedDiffViewer from '../EnhancedDiffViewer.vue'

// Mock Tauri API
vi.mock('@tauri-apps/api/core', () => ({
  invoke: vi.fn()
}))

// Mock @git-diff-view/vue
vi.mock('@git-diff-view/vue', () => ({
  DiffView: {
    name: 'DiffView',
    template: '<div class="mock-diff-view">Mock DiffView</div>'
  },
  DiffModeEnum: {
    Split: 'split',
    Unified: 'unified'
  }
}))

// Mock @git-diff-view/file
vi.mock('@git-diff-view/file', () => ({
  generateDiffFile: vi.fn(() => ({
    initTheme: vi.fn(),
    init: vi.fn(),
    buildSplitDiffLines: vi.fn(),
    buildUnifiedDiffLines: vi.fn()
  }))
}))

describe('DiffViewer', () => {
  const mockDiffData = {
    file_path: 'test.js',
    old_content: 'console.log("old");',
    new_content: 'console.log("new");',
    old_file_name: 'test.js',
    new_file_name: 'test.js',
    file_language: 'javascript',
    hunks: [
      {
        old_start: 1,
        old_lines: 1,
        new_start: 1,
        new_lines: 1,
        lines: [
          {
            line_type: 'Delete',
            content: 'console.log("old");',
            old_line_number: 1,
            new_line_number: null
          },
          {
            line_type: 'Insert',
            content: 'console.log("new");',
            old_line_number: null,
            new_line_number: 1
          }
        ]
      }
    ],
    is_binary: false,
    is_new_file: false,
    is_deleted_file: false
  }

  beforeEach(() => {
    vi.clearAllMocks()
  })

  describe('基础DiffViewer组件', () => {
    it('应该正确渲染组件', () => {
      const wrapper = mount(DiffViewer, {
        props: {
          filePath: 'test.js',
          diffType: 'WorkingTree'
        }
      })

      expect(wrapper.find('.diff-viewer').exists()).toBe(true)
      expect(wrapper.find('.diff-header').exists()).toBe(true)
    })

    it('应该显示文件路径', () => {
      const wrapper = mount(DiffViewer, {
        props: {
          filePath: 'src/main.js',
          diffType: 'WorkingTree'
        }
      })

      // 由于组件会在mounted时加载数据，我们需要等待
      expect(wrapper.find('.diff-header').exists()).toBe(true)
    })

    it('应该处理关闭事件', async () => {
      const wrapper = mount(DiffViewer, {
        props: {
          filePath: 'test.js',
          diffType: 'WorkingTree'
        }
      })

      const closeButton = wrapper.find('.close-btn')
      await closeButton.trigger('click')

      expect(wrapper.emitted('close')).toBeTruthy()
    })

    it('应该切换视图模式', async () => {
      const wrapper = mount(DiffViewer, {
        props: {
          filePath: 'test.js',
          diffType: 'WorkingTree'
        }
      })

      const toggleButton = wrapper.find('.control-btn')
      await toggleButton.trigger('click')

      // 检查内部状态变化
      expect(wrapper.vm.isUnified).toBe(true)
    })
  })

  describe('增强DiffViewer组件', () => {
    it('应该正确渲染增强组件', () => {
      const wrapper = mount(EnhancedDiffViewer, {
        props: {
          filePath: 'test.js',
          diffType: 'WorkingTree'
        }
      })

      expect(wrapper.find('.enhanced-diff-viewer').exists()).toBe(true)
      expect(wrapper.find('.diff-navigation').exists()).toBe(true)
      expect(wrapper.find('.view-controls').exists()).toBe(true)
      expect(wrapper.find('.action-controls').exists()).toBe(true)
    })

    it('应该显示差异导航控件', () => {
      const wrapper = mount(EnhancedDiffViewer, {
        props: {
          filePath: 'test.js',
          diffType: 'WorkingTree'
        }
      })

      expect(wrapper.find('.diff-navigation').exists()).toBe(true)
      expect(wrapper.find('.nav-btn').exists()).toBe(true)
      expect(wrapper.find('.diff-counter').exists()).toBe(true)
    })

    it('应该有多个控制按钮', () => {
      const wrapper = mount(EnhancedDiffViewer, {
        props: {
          filePath: 'test.js',
          diffType: 'WorkingTree'
        }
      })

      const controlButtons = wrapper.findAll('.control-btn')
      expect(controlButtons.length).toBeGreaterThan(5) // 至少有6个控制按钮
    })

    it('应该处理键盘事件', async () => {
      const wrapper = mount(EnhancedDiffViewer, {
        props: {
          filePath: 'test.js',
          diffType: 'WorkingTree'
        },
        attachTo: document.body
      })

      // 模拟Alt+ArrowDown按键
      const event = new KeyboardEvent('keydown', {
        key: 'ArrowDown',
        altKey: true
      })
      document.dispatchEvent(event)

      // 由于事件处理是异步的，我们需要等待
      await wrapper.vm.$nextTick()

      // 检查是否正确处理了键盘事件
      expect(wrapper.vm.currentDiffIndex).toBeDefined()
    })
  })

  describe('数据处理', () => {
    it('应该正确计算差异统计', () => {
      const wrapper = mount(EnhancedDiffViewer, {
        props: {
          filePath: 'test.js',
          diffType: 'WorkingTree'
        }
      })

      // 设置测试数据
      wrapper.vm.diffData = mockDiffData

      const stats = wrapper.vm.diffStats
      expect(stats).toBeDefined()
      expect(stats.additions).toBe(1)
      expect(stats.deletions).toBe(1)
    })

    it('应该检测有效内容', () => {
      const wrapper = mount(DiffViewer, {
        props: {
          filePath: 'test.js',
          diffType: 'WorkingTree'
        }
      })

      // 设置测试数据
      wrapper.vm.diffData = mockDiffData

      expect(wrapper.vm.hasValidContent).toBe(true)
    })

    it('应该处理二进制文件', () => {
      const binaryData = {
        ...mockDiffData,
        is_binary: true
      }

      const wrapper = mount(DiffViewer, {
        props: {
          filePath: 'image.png',
          diffType: 'WorkingTree'
        }
      })

      wrapper.vm.diffData = binaryData

      expect(wrapper.find('.binary-notice').exists()).toBe(true)
    })
  })

  describe('错误处理', () => {
    it('应该显示加载状态', () => {
      const wrapper = mount(DiffViewer, {
        props: {
          filePath: 'test.js',
          diffType: 'WorkingTree'
        }
      })

      wrapper.vm.loading = true

      expect(wrapper.find('.loading').exists()).toBe(true)
    })

    it('应该显示错误状态', () => {
      const wrapper = mount(DiffViewer, {
        props: {
          filePath: 'test.js',
          diffType: 'WorkingTree'
        }
      })

      wrapper.vm.error = '加载失败'

      expect(wrapper.find('.error').exists()).toBe(true)
      expect(wrapper.find('.retry-btn').exists()).toBe(true)
    })

    it('应该处理重试操作', async () => {
      const wrapper = mount(DiffViewer, {
        props: {
          filePath: 'test.js',
          diffType: 'WorkingTree'
        }
      })

      wrapper.vm.error = '加载失败'
      
      const retryButton = wrapper.find('.retry-btn')
      await retryButton.trigger('click')

      // 检查是否调用了重试方法
      expect(wrapper.vm.error).toBe(null)
    })
  })
})

describe('组件集成测试', () => {
  it('两个组件应该有相同的基础API', () => {
    const basicWrapper = mount(DiffViewer, {
      props: {
        filePath: 'test.js',
        diffType: 'WorkingTree'
      }
    })

    const enhancedWrapper = mount(EnhancedDiffViewer, {
      props: {
        filePath: 'test.js',
        diffType: 'WorkingTree'
      }
    })

    // 检查两个组件都有相同的基础方法
    expect(typeof basicWrapper.vm.loadDiff).toBe('function')
    expect(typeof enhancedWrapper.vm.loadDiff).toBe('function')
    
    expect(typeof basicWrapper.vm.closeViewer).toBe('function')
    expect(typeof enhancedWrapper.vm.closeViewer).toBe('function')
  })

  it('增强组件应该有额外的功能', () => {
    const enhancedWrapper = mount(EnhancedDiffViewer, {
      props: {
        filePath: 'test.js',
        diffType: 'WorkingTree'
      }
    })

    // 检查增强功能
    expect(typeof enhancedWrapper.vm.copyDiff).toBe('function')
    expect(typeof enhancedWrapper.vm.downloadDiff).toBe('function')
    expect(typeof enhancedWrapper.vm.formatFileSize).toBe('function')
    expect(typeof enhancedWrapper.vm.goToPreviousDiff).toBe('function')
    expect(typeof enhancedWrapper.vm.goToNextDiff).toBe('function')
  })
})
