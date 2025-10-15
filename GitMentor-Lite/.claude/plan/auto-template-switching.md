# GitMentor 智能自动切换提交分析模板功能规划

## 已明确的决策

- **技术栈**：后端使用 Rust + Tauri，前端使用 Vue 3 + TypeScript + Element Plus
- **现有模板系统**：项目已有完整的提示模板管理器，支持两段式提示词处理
- **AI配置**：已配置多种 AI 提供商，包括 OpenAI、Ollama、Anthropic 等
- **分析深度**：支持 Simple、Detailed、Deep 三种分析级别
- **缓存机制**：已有本地缓存系统用于存储提交分析结果

## 整体规划概述

### 项目目标

实现一个智能的提交分析模板自动选择系统，根据提交的特征（如文件数量、代码行数、修改类型、复杂度等）自动选择最合适的分析模板，减少用户手动选择的工作量，提高分析效率和准确性。

### 技术栈

- **后端**：Rust + Tauri
- **前端**：Vue 3 + TypeScript + Element Plus
- **AI集成**：现有AI提供商系统
- **缓存系统**：现有本地缓存机制
- **配置管理**：JSON配置文件

### 主要阶段

1. **阶段一：提交特征提取引擎** - 实现对Git提交的智能分析和特征提取
2. **阶段二：智能判断算法** - 基于规则和机器学习的模板选择逻辑
3. **阶段三：用户配置和优化** - 提供用户自定义规则和性能优化
4. **阶段四：前端界面集成** - 将自动选择功能集成到现有用户界面

### 详细任务分解

#### 阶段一：提交特征提取引擎

- **任务 1.1**：创建提交分析器模块
  - 目标：实现对Git提交的深度分析，提取关键特征
  - 输入：Git提交信息、文件变更列表、代码差异
  - 输出：结构化的提交特征数据
  - 涉及文件：`src-tauri/src/core/commit_analyzer.rs`
  - 预估工作量：3人天

- **任务 1.2**：实现复杂度评分算法
  - 目标：基于多个维度计算提交的复杂度分数
  - 输入：文件数量、代码行数、修改类型分布、文件类型
  - 输出：0-100的复杂度分数
  - 涉及文件：`src-tauri/src/core/complexity_scorer.rs`
  - 预估工作量：2人天

- **任务 1.3**：开发文件类型分类器
  - 目标：智能识别文件的技术栈和重要性级别
  - 输入：文件路径、文件内容、语言类型
  - 输出：技术栈分类、重要性等级
  - 涉及文件：`src-tauri/src/core/file_classifier.rs`
  - 预估工作量：2人天

#### 阶段二：智能判断算法

- **任务 2.1**：创建模板选择引擎
  - 目标：基于提交特征智能选择最合适的分析模板
  - 输入：提交特征数据、复杂度分数、用户配置
  - 输出：推荐的模板ID和置信度
  - 涉及文件：`src-tauri/src/core/template_selector.rs`
  - 预估工作量：3人天

- **任务 2.2**：实现规则引擎
  - 目标：提供灵活的规则配置系统
  - 输入：预定义规则、用户自定义规则
  - 输出：规则匹配结果
  - 涉及文件：`src-tauri/src/core/rule_engine.rs`
  - 预估工作量：2人天

- **任务 2.3**：开发学习反馈机制
  - 目标：记录用户的选择偏好，优化算法准确性
  - 输入：用户手动选择记录、算法预测结果
  - 输出：优化后的选择权重
  - 涉及文件：`src-tauri/src/core/feedback_learner.rs`
  - 预估工作量：2人天

#### 阶段三：用户配置和优化

- **任务 3.1**：创建配置管理系统
  - 目标：提供用户自定义规则和配置界面
  - 输入：用户配置界面、配置项定义
  - 输出：JSON配置文件
  - 涉及文件：`src-tauri/src/core/auto_template_config.rs`
  - 预估工作量：2人天

- **任务 3.2**：实现性能优化
  - 目标：优化算法性能，减少响应时间
  - 输入：性能分析结果、算法优化方案
  - 输出：优化后的算法实现
  - 涉及文件：现有核心模块的优化
  - 预估工作量：2人天

- **任务 3.3**：开发缓存机制
  - 目标：缓存特征提取结果，提高响应速度
  - 输入：提交ID、特征数据
  - 输出：缓存文件系统
  - 涉及文件：扩展现有的缓存系统
  - 预估工作量：1人天

#### 阶段四：前端界面集成

- **任务 4.1**：更新AI分析界面
  - 目标：在现有AI分析界面中集成自动选择功能
  - 输入：后端API接口、用户界面设计
  - 输出：增强的用户界面
  - 涉及文件：`src/pages/TemplateConfigPage.vue`, `src/components/GitPanel.vue`
  - 预估工作量：3人天

- **任务 4.2**：实现实时预览功能
  - 目标：展示模板选择的理由和置信度
  - 输入：选择结果数据、界面设计
  - 输出：实时预览组件
  - 涉及文件：`src/components/AutoTemplatePreview.vue`
  - 预估工作量：2人天

- **任务 4.3**：开发用户反馈界面
  - 目标：允许用户对自动选择结果进行评价
  - 输入：反馈界面设计、用户交互逻辑
  - 输出：反馈收集组件
  - 涉及文件：`src/components/TemplateFeedback.vue`
  - 预估工作量：1人天

## 技术方案

### 提交复杂度评分算法

```rust
// 复杂度评分算法示例
pub fn calculate_complexity_score(commit: &CommitData) -> u32 {
    let mut score = 0;
    
    // 文件数量权重 (30%)
    score += (commit.files_changed.len() as u32 * 5).min(30);
    
    // 代码行数权重 (25%)
    let total_lines = commit.insertions + commit.deletions;
    score += (total_lines / 10).min(25);
    
    // 文件类型权重 (20%)
    let type_score = calculate_file_type_score(&commit.files);
    score += type_score.min(20);
    
    // 修改类型权重 (15%)
    let change_score = calculate_change_type_score(&commit.files);
    score += change_score.min(15);
    
    // 路径深度权重 (10%)
    let path_score = calculate_path_depth_score(&commit.files);
    score += path_score.min(10);
    
    score.min(100)
}
```

### 自动选择触发条件

1. **基于复杂度的自动选择**：
   - 0-30分：使用简单模板 (commit_simple)
   - 31-70分：使用详细模板 (commit_detailed)
   - 71-100分：使用深度模板 (commit_deep)

2. **基于文件类型的智能选择**：
   - 配置文件变更：优先使用详细模板
   - 测试文件变更：使用简单模板
   - 核心业务代码：使用深度模板

3. **基于修改模式的选择**：
   - 重构操作：使用深度模板
   - 新功能开发：使用详细模板
   - Bug修复：根据影响范围选择

### 退回机制

1. **智能退回策略**：
   - 当置信度低于70%时，提供多个选项供用户选择
   - 当特征提取失败时，使用默认模板
   - 当用户连续否定建议时，记录反馈并调整算法

2. **用户覆盖机制**：
   - 允许用户手动选择任何模板
   - 记录用户的手动选择作为学习数据
   - 提供一键恢复默认设置的选项

## 验收标准

### 准确率目标

- **初始版本准确率**：≥75%
- **优化后准确率**：≥85%
- **用户满意度**：≥80%（通过用户反馈收集）

### 性能要求

- **响应时间**：特征提取 + 模板选择 ≤ 500ms
- **内存占用**：新增功能内存占用 ≤ 50MB
- **CPU占用**：后台分析过程CPU占用 ≤ 30%

### 用户体验指标

- **使用率**：自动选择功能使用率 ≥ 60%
- **手动干预率**：用户手动修改选择的比例 ≤ 25%
- **学习效果**：算法随使用时间提升准确率

## 数据结构设计

### 提交特征数据
```rust
pub struct CommitFeatures {
    pub commit_id: String,
    pub repo_path: String,
    pub files_changed: usize,
    pub insertions: u32,
    pub deletions: u32,
    pub file_types: Vec<FileTypeInfo>,
    pub change_types: Vec<ChangeTypeInfo>,
    pub complexity_score: u32,
    pub recommended_template: String,
    pub confidence: f32,
    pub timestamp: i64,
}
```

### 用户配置数据
```rust
pub struct AutoTemplateConfig {
    pub enabled: bool,
    pub rules: Vec<SelectionRule>,
    pub feedback_enabled: bool,
    pub learning_enabled: bool,
    pub default_template: String,
    pub custom_weights: Option<ComplexityWeights>,
}
```

### 选择规则数据
```rust
pub struct SelectionRule {
    pub id: String,
    pub name: String,
    pub description: String,
    pub conditions: Vec<Condition>,
    pub action: RuleAction,
    pub priority: u32,
    pub enabled: bool,
}
```

## 风险评估与缓解

### 技术风险

1. **算法准确性风险**
   - 缓解措施：建立完善的测试数据集，持续优化算法
   - 备选方案：提供手动选择功能作为后备

2. **性能影响风险**
   - 缓解措施：实现缓存机制，优化算法复杂度
   - 备选方案：提供性能开关，允许用户禁用自动选择

3. **兼容性风险**
   - 缓解措施：保持与现有模板系统的完全兼容
   - 备选方案：提供独立的配置选项

### 用户体验风险

1. **学习成本风险**
   - 缓解措施：提供清晰的使用说明和示例
   - 备选方案：保持现有手动选择功能不变

2. **信任度风险**
   - 缓解措施：透明的选择理由展示，允许用户干预
   - 备选方案：提供详细的算法解释文档

## 后续优化方向

1. **机器学习增强**：基于用户反馈数据训练更智能的选择模型
2. **团队协作**：支持团队级别的模板选择规则共享
3. **API扩展**：提供REST API供外部工具调用
4. **可视化分析**：提供模板选择效果的可视化分析报告
5. **多语言支持**：扩展到更多编程语言的特定规则

## 用户反馈区域

请在此区域补充您对整体规划的意见和建议：

```
用户补充内容：

---

---

---

```

---

**文档创建时间**：2025-09-17  
**版本**：v1.0  
**预期完成时间**：3-4周