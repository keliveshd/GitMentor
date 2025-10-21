// 测试统一模板管理的脚本
// 可以在浏览器开发者工具中运行

async function testUnifiedTemplateManagement() {
    console.log('🧪 开始测试统一模板管理功能...\n');

    try {
        // 1. 获取所有提交模板
        console.log('1️⃣ 获取所有提交模板');
        const commitTemplates = await window.__TAURI__.invoke('get_all_commit_templates');
        console.log('提交模板数量:', commitTemplates.length);
        commitTemplates.forEach(t => {
            console.log(`- ${t.name} (${t.id}): ${t.description}`);
        });

        // 2. 获取提交模板版本历史
        console.log('\n2️⃣ 获取提交模板版本历史');
        const templateId = 'commit_standard';
        const versions = await window.__TAURI__.invoke('get_commit_template_version_history', {
            templateId: templateId
        });
        console.log(`${templateId} 的版本历史:`, versions.length, '个版本');
        versions.forEach(v => {
            console.log(`- ${v.version}: ${v.name} (${v.created_at})`);
        });

        // 3. 检查系统模板更新
        console.log('\n3️⃣ 检查系统模板更新');
        const commitUpdates = await window.__TAURI__.invoke('check_commit_template_updates');
        console.log('提交模板更新数量:', commitUpdates.length);
        commitUpdates.forEach(u => {
            console.log(`- ${u.system_template_id}: ${u.update_description}`);
        });

        const allUpdates = await window.__TAURI__.invoke('check_unified_system_updates');
        console.log('所有模板更新数量:', allUpdates.length);

        // 4. 获取统一模板列表
        console.log('\n4️⃣ 获取统一模板列表');
        const unifiedTemplates = await window.__TAURI__.invoke('get_all_templates_unified');
        console.log('提交模板:', unifiedTemplates.commit_templates.length, '个');
        console.log('版本化模板:', unifiedTemplates.versioned_templates.length, '个');

        // 5. 创建自定义模板测试
        console.log('\n5️⃣ 创建自定义版本化模板');
        const customTemplateId = await window.__TAURI__.invoke('create_unified_custom_template', {
            name: '测试自定义模板',
            description: '用于测试版本管理的自定义模板',
            templateType: 'test_template',
            content: '这是一个测试模板的内容\n\n变量: {{test_var}}',
            baseTemplateId: null
        });
        console.log('创建的自定义模板ID:', customTemplateId);

        // 6. 更新模板并创建版本
        console.log('\n6️⃣ 更新模板并创建新版本');
        const versionId = await window.__TAURI__.invoke('update_commit_template_with_version', {
            templateId: 'commit_chinese',
            content: '这是更新后的中文提交模板内容\n\n变更的文件：{{staged_files}}\n代码差异：{{diff}}',
            versionName: '测试更新版本',
            versionDescription: '用于测试版本更新功能'
        });
        console.log('创建的新版本ID:', versionId);

        // 7. 获取更新后的版本历史
        console.log('\n7️⃣ 验证版本创建');
        const updatedVersions = await window.__TAURI__.invoke('get_commit_template_version_history', {
            templateId: 'commit_chinese'
        });
        console.log('更新后的版本历史:', updatedVersions.length, '个版本');
        console.log('最新版本:', updatedVersions[0].name);

        console.log('\n✅ 统一模板管理功能测试完成！');
        return true;

    } catch (error) {
        console.error('❌ 测试失败:', error);
        return false;
    }
}

// 导出测试函数
if (typeof window !== 'undefined') {
    window.testUnifiedTemplateManagement = testUnifiedTemplateManagement;
    console.log('💡 测试函数已加载，请在控制台运行: testUnifiedTemplateManagement()');
}