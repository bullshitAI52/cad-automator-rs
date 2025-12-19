import React, { useState } from 'react';
import { Stage, Layer, Image as KonvaImage, Text } from 'react-konva';
import { save, open } from '@tauri-apps/api/dialog';
import { invoke } from '@tauri-apps/api/tauri';
import { Upload, Save, FolderOpen, Trash2, Eraser, Sun, Moon, ZoomIn, ZoomOut } from 'lucide-react';
import ProofPanel from '../ProofPanel/ProofPanel';

interface TextAnnotation {
    id: string;
    x: number;
    y: number;
    text: string;
    color: string;
    fontSize: number;
}

interface ProofStep {
    id: string;
    because: string;
    therefore: string;
}

const Board: React.FC = () => {
    const [backgroundImage, setBackgroundImage] = useState<HTMLImageElement | null>(null);
    const [imagePath, setImagePath] = useState<string>('');
    const [annotations, setAnnotations] = useState<TextAnnotation[]>([]);
    const [selectedId, setSelectedId] = useState<string | null>(null);
    const [currentColor, setCurrentColor] = useState<string>('#FF0000');
    const [pendingText, setPendingText] = useState<string>('');
    const [canvasSize, setCanvasSize] = useState({ width: 800, height: 600 });
    const [imageScale, setImageScale] = useState(1);
    const [imageOffset, setImageOffset] = useState({ x: 0, y: 0 });
    const [fontSize, setFontSize] = useState(28);
    const [isDarkMode, setIsDarkMode] = useState(false);
    const [proofSteps, setProofSteps] = useState<ProofStep[]>([]);
    const canvasRef = React.useRef<HTMLDivElement>(null);

    // 响应式画布尺寸
    React.useEffect(() => {
        const updateCanvasSize = () => {
            if (canvasRef.current) {
                const rect = canvasRef.current.getBoundingClientRect();
                setCanvasSize({ width: rect.width, height: rect.height });
            }
        };

        updateCanvasSize();
        window.addEventListener('resize', updateCanvasSize);
        return () => window.removeEventListener('resize', updateCanvasSize);
    }, []);

    // 预设文字模板 - 扩展版
    const textTemplates = [
        'A', 'B', 'C', 'D',
        'AB', 'BC', 'AC',
        '∠___', '∠A', '∠B',
        '△ABC', '△___',
        '___°', '90°',
        '≅', '⊥', '∥'
    ];

    const handleImageImport = async () => {
        try {
            const path = await open({
                filters: [{
                    name: 'Images',
                    extensions: ['png', 'jpg', 'jpeg', 'gif', 'svg']
                }]
            });

            if (typeof path === 'string') {
                const base64Data = await invoke('read_image_as_base64', { path }) as string;
                const img = new window.Image();
                img.onload = () => {
                    // 计算缩放比例以适应画布
                    const scaleX = canvasSize.width / img.width;
                    const scaleY = canvasSize.height / img.height;
                    const scale = Math.min(scaleX, scaleY, 1); // 最大不超过原始尺寸

                    setBackgroundImage(img);
                    setImagePath(path);
                    setImageScale(scale);
                    setAnnotations([]);
                };
                img.src = base64Data;
            }
        } catch (err) {
            console.error(err);
            alert('导入图片失败: ' + err);
        }
    };

    const handleCanvasClick = (e: any) => {
        if (!pendingText) return;

        const stage = e.target.getStage();
        const pointerPos = stage.getPointerPosition();

        if (!pointerPos) return;

        const newAnnotation: TextAnnotation = {
            id: `text-${Date.now()}`,
            x: pointerPos.x,
            y: pointerPos.y,
            text: pendingText,
            color: currentColor,
            fontSize: fontSize,
        };

        setAnnotations([...annotations, newAnnotation]);
        setSelectedId(newAnnotation.id);
        setPendingText(''); // 清除待添加文字
    };

    const clearCanvas = () => {
        if (confirm('确定要清空所有标注吗？')) {
            setAnnotations([]);
            setSelectedId(null);
        }
    };

    const zoomIn = () => {
        setImageScale(prev => Math.min(prev * 1.2, 3));
    };

    const zoomOut = () => {
        setImageScale(prev => Math.max(prev / 1.2, 0.1));
    };

    const deleteSelectedAnnotation = () => {
        if (selectedId) {
            setAnnotations(annotations.filter(a => a.id !== selectedId));
            setSelectedId(null);
        }
    };

    const updateAnnotationText = (id: string, newText: string) => {
        setAnnotations(annotations.map(a =>
            a.id === id ? { ...a, text: newText } : a
        ));
    };

    const handleSave = async () => {
        try {
            const projectState = JSON.stringify({
                imagePath,
                annotations,
                proofSteps,
                fontSize,
                isDarkMode
            }, null, 2);
            const path = await save({
                filters: [{ name: 'Proof Project', extensions: ['proof', 'json'] }]
            });
            if (path) {
                await invoke('save_file', { path, content: projectState });
                alert('项目已保存!');
            }
        } catch (err) {
            console.error(err);
            alert('保存失败');
        }
    };

    const handleLoad = async () => {
        try {
            const path = await open({
                filters: [{ name: 'Proof Project', extensions: ['proof', 'json'] }]
            });
            if (typeof path === 'string') {
                const content = await invoke('read_file', { path }) as string;
                const data = JSON.parse(content);

                if (data.imagePath) {
                    const base64Data = await invoke('read_image_as_base64', { path: data.imagePath }) as string;
                    const img = new window.Image();
                    img.onload = () => {
                        setBackgroundImage(img);
                        setImagePath(data.imagePath);
                    };
                    img.src = base64Data;
                }

                setAnnotations(data.annotations || []);
                setProofSteps(data.proofSteps || []);
                setFontSize(data.fontSize || 28);
                setIsDarkMode(data.isDarkMode || false);
                setSelectedId(null);
            }
        } catch (err) {
            console.error(err);
            alert('加载失败');
        }
    };

    const selectedAnnotation = annotations.find(a => a.id === selectedId);

    // 初始化proofSteps如果为空
    React.useEffect(() => {
        if (proofSteps.length === 0) {
            setProofSteps([{ id: '1', because: '', therefore: '' }]);
        }
    }, []);

    return (
        <div className={`flex h-screen w-full ${isDarkMode ? 'bg-slate-900' : 'bg-gradient-to-br from-slate-50 to-blue-50'}`}>
            {/* 左侧工具栏 */}
            <div className={`w-72 flex flex-col border-r ${isDarkMode ? 'border-slate-700 bg-slate-800' : 'border-slate-200 bg-white'} shadow-lg`}>
                {/* 标题 */}
                <div className="p-6 border-b border-slate-200 bg-gradient-to-r from-blue-50 to-indigo-50">
                    <h2 className="text-xl font-bold text-slate-800">几何标注</h2>
                    <p className="text-sm text-slate-500 mt-1">导入图片 · 添加文字</p>
                </div>

                {/* 导入图片 */}
                <div className="p-4 border-b border-slate-200">
                    <button
                        className="w-full py-3 px-4 rounded-xl bg-gradient-to-r from-blue-500 to-indigo-500 hover:from-blue-600 hover:to-indigo-600 text-white font-semibold flex items-center justify-center gap-2 shadow-md transition"
                        onClick={handleImageImport}
                    >
                        <Upload size={20} />
                        <span>导入几何图</span>
                    </button>
                </div>

                {/* 文字模板 */}
                <div className="p-4 border-b border-slate-200">
                    <div className="text-sm font-semibold text-slate-700 mb-3">快速添加文字</div>
                    <div className="grid grid-cols-3 gap-2">
                        {textTemplates.map((template) => (
                            <button
                                key={template}
                                className={`py-3 px-2 rounded-lg border-2 transition font-semibold text-lg ${pendingText === template
                                    ? 'bg-blue-100 border-blue-500 text-blue-700'
                                    : 'bg-white border-slate-300 text-slate-700 hover:border-blue-400'
                                    }`}
                                onClick={() => setPendingText(pendingText === template ? '' : template)}
                            >
                                {template}
                            </button>
                        ))}
                    </div>
                    {pendingText && (
                        <div className="mt-3 p-3 bg-blue-50 rounded-lg text-sm text-blue-700 border border-blue-200">
                            💡 点击画布添加 "<span className="font-bold">{pendingText}</span>"
                        </div>
                    )}
                </div>

                {/* 颜色选择 */}
                <div className="p-4 border-b border-slate-200">
                    <div className="text-sm font-semibold text-slate-700 mb-3">文字颜色</div>
                    <div className="flex gap-2">
                        {['#FF0000', '#0000FF', '#000000'].map((color) => (
                            <button
                                key={color}
                                className={`flex-1 h-12 rounded-lg border-2 transition ${currentColor === color ? 'border-blue-500 scale-105' : 'border-slate-300'
                                    }`}
                                style={{ backgroundColor: color }}
                                onClick={() => setCurrentColor(color)}
                            />
                        ))}
                    </div>
                </div>

                {/* 字体大小 */}
                <div className="p-4 border-b border-slate-200">
                    <div className="text-sm font-semibold text-slate-700 mb-2">文字大小: {fontSize}px</div>
                    <input
                        type="range"
                        min="16"
                        max="48"
                        value={fontSize}
                        onChange={(e) => setFontSize(Number(e.target.value))}
                        className="w-full"
                    />
                </div>

                {/* 工具按钮 */}
                <div className="p-4 border-b border-slate-200">
                    <div className="grid grid-cols-2 gap-2">
                        <button
                            onClick={clearCanvas}
                            className="py-2 px-3 rounded-lg bg-red-50 hover:bg-red-100 border border-red-200 text-red-700 flex items-center justify-center gap-2 transition"
                        >
                            <Eraser size={16} />
                            <span className="text-sm font-semibold">清空</span>
                        </button>
                        <button
                            onClick={() => setIsDarkMode(!isDarkMode)}
                            className="py-2 px-3 rounded-lg bg-slate-100 hover:bg-slate-200 border border-slate-300 text-slate-700 flex items-center justify-center gap-2 transition"
                        >
                            {isDarkMode ? <Sun size={16} /> : <Moon size={16} />}
                            <span className="text-sm font-semibold">主题</span>
                        </button>
                        <button
                            onClick={zoomIn}
                            className="py-2 px-3 rounded-lg bg-green-50 hover:bg-green-100 border border-green-200 text-green-700 flex items-center justify-center gap-2 transition"
                        >
                            <ZoomIn size={16} />
                            <span className="text-sm font-semibold">放大</span>
                        </button>
                        <button
                            onClick={zoomOut}
                            className="py-2 px-3 rounded-lg bg-blue-50 hover:bg-blue-100 border border-blue-200 text-blue-700 flex items-center justify-center gap-2 transition"
                        >
                            <ZoomOut size={16} />
                            <span className="text-sm font-semibold">缩小</span>
                        </button>
                    </div>
                </div>

                {/* 编辑选中文字 */}
                {selectedAnnotation && (
                    <div className="p-4 border-b border-slate-200 bg-amber-50">
                        <div className="text-sm font-semibold text-amber-800 mb-2">编辑文字</div>
                        <input
                            type="text"
                            value={selectedAnnotation.text}
                            onChange={(e) => updateAnnotationText(selectedAnnotation.id, e.target.value)}
                            className="w-full mb-3 px-3 py-2 border-2 border-amber-300 rounded-lg text-base font-semibold focus:border-amber-500 focus:outline-none"
                            placeholder="输入文字..."
                        />
                        <button
                            className="w-full py-2 px-3 rounded-lg bg-red-500 hover:bg-red-600 text-white flex items-center justify-center gap-2 transition"
                            onClick={deleteSelectedAnnotation}
                        >
                            <Trash2 size={16} />
                            <span className="font-semibold">删除</span>
                        </button>
                    </div>
                )}

                {/* 文件操作 */}
                <div className="p-4 mt-auto border-t border-slate-200">
                    <div className="flex gap-2 mb-3">
                        <button
                            className="flex-1 py-2 px-3 rounded-lg bg-slate-100 hover:bg-slate-200 text-slate-700 flex items-center justify-center gap-2 transition"
                            onClick={handleSave}
                        >
                            <Save size={16} />
                            <span className="text-sm font-semibold">保存</span>
                        </button>
                        <button
                            className="flex-1 py-2 px-3 rounded-lg bg-slate-100 hover:bg-slate-200 text-slate-700 flex items-center justify-center gap-2 transition"
                            onClick={handleLoad}
                        >
                            <FolderOpen size={16} />
                            <span className="text-sm font-semibold">打开</span>
                        </button>
                    </div>
                    <div className="text-xs text-slate-500 text-center">
                        已添加 {annotations.length} 个文字标注
                    </div>
                </div>
            </div>

            {/* 主工作区 */}
            <div className="flex-1 flex">
                {/* 画布区域 */}
                <div ref={canvasRef} className="flex-1 relative bg-white">
                    {!backgroundImage ? (
                        <div className="absolute inset-0 flex items-center justify-center">
                            <div className="text-center">
                                <Upload size={64} className="mx-auto text-slate-300 mb-4" />
                                <h3 className="text-xl font-bold text-slate-400 mb-2">
                                    点击"导入几何图"开始
                                </h3>
                                <p className="text-sm text-slate-500">
                                    支持 PNG, JPG, SVG 等图片格式
                                </p>
                            </div>
                        </div>
                    ) : (
                        <Stage
                            width={canvasSize.width}
                            height={canvasSize.height}
                            onClick={handleCanvasClick}
                            onMouseDown={(e) => {
                                if (e.target === e.target.getStage()) {
                                    setSelectedId(null);
                                }
                            }}
                        >
                            <Layer>
                                <KonvaImage
                                    image={backgroundImage}
                                    width={backgroundImage.width * imageScale}
                                    height={backgroundImage.height * imageScale}
                                />
                            </Layer>
                            <Layer>
                                {annotations.map((annotation) => (
                                    <Text
                                        key={annotation.id}
                                        x={annotation.x}
                                        y={annotation.y}
                                        text={annotation.text}
                                        fontSize={annotation.fontSize}
                                        fontStyle="bold"
                                        fill={annotation.color}
                                        stroke={selectedId === annotation.id ? '#FFD700' : undefined}
                                        strokeWidth={selectedId === annotation.id ? 2 : 0}
                                        shadowColor={selectedId === annotation.id ? '#FFD700' : undefined}
                                        shadowBlur={selectedId === annotation.id ? 10 : 0}
                                        onClick={() => setSelectedId(annotation.id)}
                                        draggable
                                        onDragEnd={(e) => {
                                            setAnnotations(annotations.map(a =>
                                                a.id === annotation.id
                                                    ? { ...a, x: e.target.x(), y: e.target.y() }
                                                    : a
                                            ));
                                        }}
                                    />
                                ))}
                            </Layer>
                        </Stage>
                    )}
                </div>

                {/* 右侧证明面板 */}
                <div className={`w-96 border-l ${isDarkMode ? 'border-slate-700 bg-slate-800' : 'border-slate-200 bg-white'} overflow-y-auto`}>
                    <ProofPanel steps={proofSteps} onChange={setProofSteps} />
                </div>
            </div>
        </div>
    );
};

export default Board;
