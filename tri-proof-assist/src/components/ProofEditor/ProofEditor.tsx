import React, { useState } from 'react';
import { Plus, Trash2, Sparkles, BookOpen, CheckCircle, HelpCircle, Download } from 'lucide-react';
import { invoke } from '@tauri-apps/api/tauri';

interface ProofStep {
    id: string;
    because: string;
    therefore: string;
}

interface GeometrySymbol {
    symbol: string;
    name: string;
    latex: string;
}

const ProofEditor: React.FC = () => {
    const [steps, setSteps] = useState<ProofStep[]>([
        { id: '1', because: '', therefore: '' }
    ]);
    const [topic, setTopic] = useState<string>('三角形');
    const [difficulty, setDifficulty] = useState<string>('简单');
    const [apiKey, setApiKey] = useState<string>('');
    const [isGenerating, setIsGenerating] = useState<boolean>(false);
    const [selectedSymbol, setSelectedSymbol] = useState<string>('');
    const [activeTextarea, setActiveTextarea] = useState<{
        stepId: string;
        field: 'because' | 'therefore';
    } | null>(null);

    // 几何符号库
    const geometrySymbols: GeometrySymbol[] = [
        { symbol: '∠', name: '角', latex: '\\angle' },
        { symbol: '△', name: '三角形', latex: '\\triangle' },
        { symbol: '⊥', name: '垂直', latex: '\\perp' },
        { symbol: '∥', name: '平行', latex: '\\parallel' },
        { symbol: '°', name: '度', latex: '^{\\circ}' },
        { symbol: '∵', name: '因为', latex: '\\because' },
        { symbol: '∴', name: '所以', latex: '\\therefore' },
        { symbol: 'AB', name: '线段', latex: '\\overline{AB}' },
        { symbol: '→', name: '推出', latex: '\\Rightarrow' },
        { symbol: '=', name: '等于', latex: '=' },
        { symbol: '≠', name: '不等于', latex: '\\neq' },
        { symbol: '≈', name: '约等于', latex: '\\approx' },
        { symbol: 'π', name: '圆周率', latex: '\\pi' },
    ];

    // 证明模板
    const proofTemplates = [
        {
            name: '三角形内角和',
            steps: [
                { because: '三角形三个角加起来是180°', therefore: '∠A + ∠B + ∠C = 180°' },
                { because: '∠A = 60°，∠B = 70°', therefore: '∠C = 180° - 60° - 70° = 50°' }
            ]
        },
        {
            name: '长方形周长',
            steps: [
                { because: '长8厘米，宽5厘米', therefore: '长方形周长 = (长 + 宽) × 2' },
                { because: '(8 + 5) × 2 = 26', therefore: '周长是26厘米' }
            ]
        },
        {
            name: '正方形面积',
            steps: [
                { because: '边长是6厘米', therefore: '正方形面积 = 边长 × 边长' },
                { because: '6 × 6 = 36', therefore: '面积是36平方厘米' }
            ]
        }
    ];

    const addStep = () => {
        setSteps([...steps, {
            id: Date.now().toString(),
            because: '',
            therefore: ''
        }]);
    };

    const removeStep = (id: string) => {
        if (steps.length > 1) {
            setSteps(steps.filter(s => s.id !== id));
        }
    };

    const updateStep = (id: string, field: 'because' | 'therefore', value: string) => {
        setSteps(steps.map(s =>
            s.id === id ? { ...s, [field]: value } : s
        ));
    };

    const insertSymbol = (symbol: string) => {
        setSelectedSymbol(symbol);
        
        // 如果当前有活跃的输入框，直接插入符号
        if (activeTextarea) {
            const { stepId, field } = activeTextarea;
            const step = steps.find(s => s.id === stepId);
            if (step) {
                const currentValue = step[field];
                const newValue = currentValue + symbol;
                updateStep(stepId, field, newValue);
            }
        }
    };

    const handleTextareaFocus = (stepId: string, field: 'because' | 'therefore') => {
        setActiveTextarea({ stepId, field });
    };

    const handleTextareaBlur = () => {
        setActiveTextarea(null);
    };

    const loadTemplate = (template: any) => {
        const newSteps = template.steps.map((step: any, index: number) => ({
            id: (index + 1).toString(),
            because: step.because,
            therefore: step.therefore
        }));
        setSteps(newSteps);
    };

    const generateProof = async () => {
        setIsGenerating(true);
        try {
            const result = await invoke('generate_problem', { 
                topic, 
                difficulty, 
                apiKey 
            }) as string;
            
            // 解析AI返回的结果
            const data = JSON.parse(result);
            
            // 显示成功消息
            alert(`✅ AI生成成功!\n\n标题: ${data.title}\n描述: ${data.description}`);
            
            // 如果AI返回了步骤，直接加载到编辑器中
            if (data.steps && Array.isArray(data.steps)) {
                const newSteps = data.steps.map((step: any, index: number) => ({
                    id: (Date.now() + index).toString(),
                    because: step.because || '',
                    therefore: step.therefore || ''
                }));
                setSteps(newSteps);
            } else {
                // 如果没有步骤，至少保留一个空步骤
                setSteps([{ id: '1', because: '', therefore: '' }]);
            }
            
        } catch (error) {
            console.error('生成失败:', error);
            alert('❌ 生成失败，请检查API Key或网络连接\n\n如果没有API Key，程序会使用示例数据。');
        } finally {
            setIsGenerating(false);
        }
    };

    const exportProof = () => {
        const proofText = steps.map((step, index) => 
            `${index + 1}. 因为: ${step.because}\n   所以: ${step.therefore}`
        ).join('\n\n');
        
        const blob = new Blob([proofText], { type: 'text/plain' });
        const url = URL.createObjectURL(blob);
        const a = document.createElement('a');
        a.href = url;
        a.download = `几何证明_${new Date().toISOString().split('T')[0]}.txt`;
        a.click();
        URL.revokeObjectURL(url);
    };

    return (
        <div className="h-full flex flex-col p-4 max-w-6xl mx-auto">
            {/* 标题区域 */}
            <div className="mb-6 text-center">
                <h1 className="text-3xl font-bold text-blue-800 mb-2 flex items-center justify-center gap-2">
                    <BookOpen size={32} />
                    几何证明小助手
                </h1>
                <p className="text-slate-600">帮助小朋友学习几何证明，一步一步来！</p>
            </div>

            <div className="flex-1 grid grid-cols-1 lg:grid-cols-3 gap-6">
                {/* 左侧：符号库和模板 */}
                <div className="lg:col-span-1 space-y-6">
                    {/* 几何符号库 */}
                    <div className="bg-white rounded-2xl p-5 shadow-lg border border-blue-100">
                        <h3 className="text-lg font-bold text-blue-700 mb-4 flex items-center gap-2">
                            <HelpCircle size={20} />
                            几何符号库
                        </h3>
                        <div className="grid grid-cols-4 gap-3">
                            {geometrySymbols.map((symbol) => (
                                <button
                                    key={symbol.symbol}
                                    onClick={() => insertSymbol(symbol.symbol)}
                                    className="p-3 bg-blue-50 hover:bg-blue-100 rounded-xl border border-blue-200 transition-all hover:scale-105 active:scale-95"
                                    title={`${symbol.name} (${symbol.latex})`}
                                >
                                    <div className="text-2xl font-bold text-blue-700">{symbol.symbol}</div>
                                    <div className="text-xs text-blue-600 mt-1">{symbol.name}</div>
                                </button>
                            ))}
                        </div>
                        {selectedSymbol && (
                            <div className="mt-4 p-3 bg-green-50 rounded-lg border border-green-200">
                                <div className="text-sm text-green-700">
                                    已选择: <span className="font-bold text-xl">{selectedSymbol}</span>
                                    <span className="ml-2 text-green-600">
                                        {activeTextarea ? '已自动插入到当前输入框' : '点击输入框后再次点击符号插入'}
                                    </span>
                                </div>
                            </div>
                        )}
                    </div>

                    {/* 证明模板 */}
                    <div className="bg-white rounded-2xl p-5 shadow-lg border border-green-100">
                        <h3 className="text-lg font-bold text-green-700 mb-4">常用证明模板</h3>
                        <div className="space-y-3">
                            {proofTemplates.map((template, index) => (
                                <button
                                    key={index}
                                    onClick={() => loadTemplate(template)}
                                    className="w-full p-3 text-left bg-green-50 hover:bg-green-100 rounded-lg border border-green-200 transition"
                                >
                                    <div className="font-semibold text-green-800">{template.name}</div>
                                    <div className="text-sm text-green-600 mt-1">
                                        {template.steps.length} 个步骤
                                    </div>
                                </button>
                            ))}
                        </div>
                    </div>

                    {/* AI辅助生成 */}
                    <div className="bg-white rounded-2xl p-5 shadow-lg border border-purple-100">
                        <h3 className="text-lg font-bold text-purple-700 mb-4 flex items-center gap-2">
                            <Sparkles size={20} />
                            AI智能辅助
                        </h3>
                        <div className="space-y-3">
                            <div>
                                <label className="block text-sm font-medium text-slate-700 mb-1">
                                    证明主题
                                </label>
                                <select
                                    value={topic}
                                    onChange={(e) => setTopic(e.target.value)}
                                    className="w-full p-2 border border-slate-300 rounded-lg"
                                >
                                    <option value="三角形">三角形</option>
                                    <option value="长方形">长方形</option>
                                    <option value="正方形">正方形</option>
                                    <option value="平行四边形">平行四边形</option>
                                    <option value="圆">圆</option>
                                </select>
                            </div>
                            <div>
                                <label className="block text-sm font-medium text-slate-700 mb-1">
                                    难度级别
                                </label>
                                <select
                                    value={difficulty}
                                    onChange={(e) => setDifficulty(e.target.value)}
                                    className="w-full p-2 border border-slate-300 rounded-lg"
                                >
                                    <option value="简单">简单</option>
                                    <option value="中等">中等</option>
                                    <option value="困难">困难</option>
                                </select>
                            </div>
                            <div>
                                <label className="block text-sm font-medium text-slate-700 mb-1">
                                    DeepSeek API Key (可选)
                                </label>
                                <input
                                    type="password"
                                    value={apiKey}
                                    onChange={(e) => setApiKey(e.target.value)}
                                    placeholder="输入API Key使用AI功能"
                                    className="w-full p-2 border border-slate-300 rounded-lg"
                                />
                            </div>
                            <button
                                onClick={generateProof}
                                disabled={isGenerating}
                                className="w-full py-3 px-4 rounded-xl bg-gradient-to-r from-purple-500 to-pink-500 hover:from-purple-600 hover:to-pink-600 text-white font-semibold flex items-center justify-center gap-2 shadow-md transition disabled:opacity-50"
                            >
                                <Sparkles size={20} />
                                {isGenerating ? '正在生成...' : 'AI生成证明思路'}
                            </button>
                        </div>
                    </div>
                </div>

                {/* 中间：证明步骤编辑器 */}
                <div className="lg:col-span-2">
                    <div className="bg-white rounded-2xl p-6 shadow-lg border border-orange-100 h-full flex flex-col">
                        <div className="flex items-center justify-between mb-6">
                            <h2 className="text-2xl font-bold text-orange-700">证明步骤编辑器</h2>
                            <div className="flex gap-2">
                                <button
                                    onClick={exportProof}
                                    className="px-4 py-2 bg-blue-500 hover:bg-blue-600 text-white rounded-lg flex items-center gap-2"
                                >
                                    <Download size={16} />
                                    导出
                                </button>
                                <button
                                    onClick={addStep}
                                    className="px-4 py-2 bg-green-500 hover:bg-green-600 text-white rounded-lg flex items-center gap-2"
                                >
                                    <Plus size={16} />
                                    添加步骤
                                </button>
                            </div>
                        </div>

                        <div className="flex-1 overflow-y-auto space-y-4">
                            {steps.map((step, index) => (
                                <div key={step.id} className="p-5 bg-gradient-to-r from-blue-50 to-indigo-50 rounded-xl border-2 border-blue-200">
                                    <div className="flex items-center justify-between mb-4">
                                        <div className="flex items-center gap-3">
                                            <div className="w-8 h-8 bg-blue-500 text-white rounded-full flex items-center justify-center font-bold">
                                                {index + 1}
                                            </div>
                                            <div className="text-lg font-bold text-blue-700">步骤 {index + 1}</div>
                                        </div>
                                        {steps.length > 1 && (
                                            <button
                                                onClick={() => removeStep(step.id)}
                                                className="p-2 hover:bg-red-100 rounded-lg transition"
                                            >
                                                <Trash2 size={18} className="text-red-500" />
                                            </button>
                                        )}
                                    </div>

                                    <div className="grid grid-cols-1 md:grid-cols-2 gap-4">
                                        {/* 因为 */}
                                        <div>
                                            <div className="flex items-center gap-2 mb-2">
                                                <div className="w-6 h-6 bg-green-100 text-green-700 rounded-full flex items-center justify-center">
                                                    ∵
                                                </div>
                                                <label className="text-sm font-bold text-green-700">
                                                    因为 (已知条件)
                                                </label>
                                            </div>
                                            <textarea
                                                value={step.because}
                                                onChange={(e) => updateStep(step.id, 'because', e.target.value)}
                                                onFocus={() => handleTextareaFocus(step.id, 'because')}
                                                onBlur={handleTextareaBlur}
                                                className="w-full h-32 p-3 border-2 border-green-300 rounded-lg resize-none focus:border-green-500 focus:outline-none bg-white"
                                                placeholder="输入已知条件，如：AB = CD, ∠A = 90°"
                                            />
                                        </div>

                                        {/* 所以 */}
                                        <div>
                                            <div className="flex items-center gap-2 mb-2">
                                                <div className="w-6 h-6 bg-red-100 text-red-700 rounded-full flex items-center justify-center">
                                                    ∴
                                                </div>
                                                <label className="text-sm font-bold text-red-700">
                                                    所以 (推导结论)
                                                </label>
                                            </div>
                                            <textarea
                                                value={step.therefore}
                                                onChange={(e) => updateStep(step.id, 'therefore', e.target.value)}
                                                onFocus={() => handleTextareaFocus(step.id, 'therefore')}
                                                onBlur={handleTextareaBlur}
                                                className="w-full h-32 p-3 border-2 border-red-300 rounded-lg resize-none focus:border-red-500 focus:outline-none bg-white"
                                                placeholder="输入推导结论，如：∠C = 50°, AB ∥ CD"
                                            />
                                        </div>
                                    </div>
                                </div>
                            ))}
                        </div>

                        {/* 底部提示 */}
                        <div className="mt-6 p-4 bg-yellow-50 rounded-lg border border-yellow-200">
                            <div className="flex items-start gap-3">
                                <CheckCircle className="text-yellow-600 mt-1" size={20} />
                                <div>
                                    <div className="font-semibold text-yellow-800">小贴士</div>
                                    <div className="text-sm text-yellow-700 mt-1">
                                        1. 使用符号库快速插入几何符号<br/>
                                        2. 每个步骤都要有明确的"因为"和"所以"<br/>
                                        3. 可以使用模板快速开始<br/>
                                        4. 点击"导出"保存你的证明
                                    </div>
                                </div>
                            </div>
                        </div>
                    </div>
                </div>
            </div>

            {/* 底部统计 */}
            <div className="mt-6 text-center text-slate-600 text-sm">
                已创建 {steps.length} 个证明步骤 • 几何证明小助手 v1.0
            </div>
        </div>
    );
};

export default ProofEditor;
