# tri-proof-assist —— 小学生几何练习小助手

给小学生练几何用的桌面小应用：画一画图形，再按“因为 → 所以”一步步写出思路。

> 仓库名叫 cad-automator-rs 是历史原因，里面实际只有这一个应用 tri-proof-assist。

## 能干嘛

- **画图板**：在画布上画三角形、四边形、圆这些图形，还能改图形属性。
- **证明编辑器**：按“因为 → 所以”的格式写步骤，数学符号能正常显示。
- **AI 出思路**：输个题目和难度，调 DeepSeek 生成证明思路；没填 Key 就用内置例题。
- **保存和读取**：做好的题目能存下来，下次再打开接着练。

## 内置例题（不用 Key 也能练）

- 三角形内角和
- 长方形周长
- 正方形面积
- 认识平行四边形
- 圆的周长（π 取 3.14）

只用小学数学知识：认识图形、角、周长、面积，不用全等、勾股定理、根号这些初中内容。

## 怎么跑

```bash
cd tri-proof-assist
npm install
npm run tauri dev
```

打包：

```bash
npm run tauri build
```

## DeepSeek Key（可选）

不填也能用内置例题。想让 AI 出新题，设置环境变量：

```bash
DEEPSEEK_API_KEY=你的 Key
```

## 技术

Tauri（Rust）+ React + TypeScript + Vite，画布用 Konva，数学公式用 KaTeX。

## 许可

PolyForm 非商业许可（Noncommercial 1.0.0）：禁止商业使用和售卖，见根目录 LICENSE。
