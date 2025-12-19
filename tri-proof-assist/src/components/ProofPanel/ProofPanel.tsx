import React from 'react';
import { Plus, Trash2 } from 'lucide-react';

interface ProofStep {
    id: string;
    because: string;
    therefore: string;
}

interface ProofPanelProps {
    steps: ProofStep[];
    onChange: (steps: ProofStep[]) => void;
}

const ProofPanel: React.FC<ProofPanelProps> = ({ steps, onChange }) => {

    const addStep = () => {
        onChange([...steps, {
            id: Date.now().toString(),
            because: '',
            therefore: ''
        }]);
    };

    const removeStep = (id: string) => {
        if (steps.length > 1) {
            onChange(steps.filter(s => s.id !== id));
        }
    };

    const updateStep = (id: string, field: 'because' | 'therefore', value: string) => {
        onChange(steps.map(s =>
            s.id === id ? { ...s, [field]: value } : s
        ));
    };

    return (
        <div className="h-full flex flex-col bg-white">
            {/* 标题 */}
            <div className="p-6 border-b border-slate-200 bg-gradient-to-r from-green-50 to-emerald-50">
                <h2 className="text-xl font-bold text-slate-800">证明步骤</h2>
                <p className="text-sm text-slate-500 mt-1">因为 → 所以</p>
            </div>

            {/* 证明步骤列表 */}
            <div className="flex-1 overflow-y-auto p-4 space-y-4">
                {steps.map((step, index) => (
                    <div key={step.id} className="p-4 bg-slate-50 rounded-xl border border-slate-200">
                        <div className="flex items-center justify-between mb-3">
                            <div className="text-sm font-bold text-slate-600">步骤 {index + 1}</div>
                            {steps.length > 1 && (
                                <button
                                    onClick={() => removeStep(step.id)}
                                    className="p-1 hover:bg-red-100 rounded transition"
                                >
                                    <Trash2 size={16} className="text-red-500" />
                                </button>
                            )}
                        </div>

                        {/* 因为 */}
                        <div className="mb-3">
                            <label className="block text-sm font-semibold text-green-700 mb-1">
                                因为
                            </label>
                            <textarea
                                value={step.because}
                                onChange={(e) => updateStep(step.id, 'because', e.target.value)}
                                className="w-full px-3 py-2 border-2 border-green-200 rounded-lg resize-none focus:border-green-400 focus:outline-none"
                                rows={2}
                                placeholder="输入已知条件或前提..."
                            />
                        </div>

                        {/* 所以 */}
                        <div>
                            <label className="block text-sm font-semibold text-red-700 mb-1">
                                所以
                            </label>
                            <textarea
                                value={step.therefore}
                                onChange={(e) => updateStep(step.id, 'therefore', e.target.value)}
                                className="w-full px-3 py-2 border-2 border-red-200 rounded-lg resize-none focus:border-red-400 focus:outline-none"
                                rows={2}
                                placeholder="输入推导结论..."
                            />
                        </div>
                    </div>
                ))}
            </div>

            {/* 添加步骤按钮 */}
            <div className="p-4 border-t border-slate-200">
                <button
                    onClick={addStep}
                    className="w-full py-3 px-4 rounded-xl bg-gradient-to-r from-green-500 to-emerald-500 hover:from-green-600 hover:to-emerald-600 text-white font-semibold flex items-center justify-center gap-2 shadow-md transition"
                >
                    <Plus size={20} />
                    <span>添加证明步骤</span>
                </button>
            </div>
        </div>
    );
};

export default ProofPanel;
