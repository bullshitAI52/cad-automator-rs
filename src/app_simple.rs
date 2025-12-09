use crate::excel_export::export_to_excel;
use crate::image_info::{scan_folder, BatchImageInfo};
use crate::utils::{batch_convert_color_mode, batch_rename_by_size};
use eframe::egui;
use egui_extras::{Column, TableBuilder};
#[allow(unused_imports)]
use log::{error, info};
use std::collections::VecDeque;
use std::path::PathBuf;
use std::sync::mpsc;
use std::thread;

#[derive(Clone, PartialEq)]
enum AppState {
    Idle,
    Loading,
    Renaming,
    Converting,
    Exporting,
    Undoing,
}

// 撤销操作类型
#[derive(Clone)]
enum UndoAction {
    Rename {
        original_name: String,
        new_name: String,
    },
    // 可以添加其他操作类型
}

enum AppMessage {
    FolderSelected(PathBuf),
    LoadComplete(Result<BatchImageInfo, String>),
    RenameRequested,
    RenameComplete(Result<Vec<(String, String)>, String>), // 修改为返回重命名结果
    ExportRequested,
    ExportComplete(Result<(), String>),
    ConvertRequested(String),
    ConvertComplete(Result<(), String>),
    RefreshRequested,
    UndoRequested,
    UndoComplete(Result<(), String>),
    Error(String),
}

pub struct ImageToolApp {
    folder_path: Option<PathBuf>,
    batch_info: Option<BatchImageInfo>,
    selected_image: Option<usize>,
    preview_texture: Option<egui::TextureHandle>,
    state: AppState,
    status_message: String,
    progress: f32,
    error_message: Option<String>,
    // 编辑状态
    editing_row: Option<usize>,
    edit_buffer: String,
    // 撤销栈
    undo_stack: VecDeque<UndoAction>,
    max_undo_steps: usize,
    // 清除对话框状态
    show_clear_dialog: bool,
    // 消息通道
    tx: mpsc::Sender<AppMessage>,
    rx: mpsc::Receiver<AppMessage>,
}

impl ImageToolApp {
    pub fn new(_cc: &eframe::CreationContext<'_>) -> Self {
        let (tx, rx) = mpsc::channel();

        Self {
            folder_path: None,
            batch_info: None,
            selected_image: None,
            preview_texture: None,
            state: AppState::Idle,
            status_message: "欢迎使用！请先选择一个包含图片的文件夹。".to_string(),
            progress: 0.0,
            error_message: None,
            // 编辑状态
            editing_row: None,
            edit_buffer: String::new(),
            // 撤销栈
            undo_stack: VecDeque::new(),
            max_undo_steps: 10, // 最多撤销10步
            // 清除对话框状态
            show_clear_dialog: false,
            tx,
            rx,
        }
    }

    fn start_editing(&mut self, row_index: usize) {
        if let Some(batch_info) = &self.batch_info {
            if row_index < batch_info.images.len() {
                let image_info = &batch_info.images[row_index];
                self.editing_row = Some(row_index);
                self.edit_buffer = image_info.file_name.clone();
            }
        }
    }

    fn save_editing(&mut self, ctx: &egui::Context) -> bool {
        if let (Some(row_index), Some(folder_path), Some(batch_info)) =
            (self.editing_row, &self.folder_path, &self.batch_info)
        {
            if row_index < batch_info.images.len() {
                let image_info = &batch_info.images[row_index];
                let original_name = &image_info.file_name;
                let mut new_name = self.edit_buffer.trim().to_string();

                // 检查名称是否有效且不同
                if new_name.is_empty() {
                    self.error_message = Some("文件名不能为空".to_string());
                    self.cancel_editing();
                    return false;
                }

                // 处理扩展名：如果新名称没有扩展名，添加原扩展名
                if !new_name.contains('.') {
                    if let Some(ext) = std::path::Path::new(original_name).extension() {
                        if let Some(ext_str) = ext.to_str() {
                            if !ext_str.is_empty() {
                                new_name.push('.');
                                new_name.push_str(ext_str);
                            }
                        }
                    }
                }

                if new_name == *original_name {
                    self.cancel_editing();
                    return false;
                }

                // 执行重命名
                let original_path = folder_path.join(original_name);
                let new_path = folder_path.join(&new_name);

                if let Err(e) = std::fs::rename(&original_path, &new_path) {
                    self.error_message = Some(format!("重命名失败: {}", e));
                    self.cancel_editing();
                    return false;
                }

                // 保存到撤销栈
                let undo_action = UndoAction::Rename {
                    original_name: original_name.clone(),
                    new_name: new_name.clone(),
                };
                self.undo_stack.push_back(undo_action);
                if self.undo_stack.len() > self.max_undo_steps {
                    self.undo_stack.pop_front();
                }

                // 刷新列表
                self.load_folder(folder_path.clone(), ctx);
                self.cancel_editing();
                return true;
            }
        }
        false
    }

    fn cancel_editing(&mut self) {
        self.editing_row = None;
        self.edit_buffer.clear();
    }

    fn clear_all_filenames(&mut self, ctx: &egui::Context) {
        if let (Some(folder_path), Some(batch_info)) = (&self.folder_path, &self.batch_info) {
            let mut cleared_count = 0;
            let mut errors = Vec::new();

            for image_info in &batch_info.images {
                if image_info.error.is_some() {
                    continue;
                }

                let original_path = folder_path.join(&image_info.file_name);
                if !original_path.exists() {
                    continue;
                }

                // 获取文件扩展名
                let extension = original_path
                    .extension()
                    .and_then(|ext| ext.to_str())
                    .unwrap_or("");

                // 构建新文件名：只保留扩展名
                let new_name = if extension.is_empty() {
                    "未命名".to_string()
                } else {
                    format!("未命名.{}", extension)
                };

                let new_path = folder_path.join(&new_name);

                // 处理文件名冲突
                let final_path = self.handle_name_conflict(&new_path);

                match std::fs::rename(&original_path, &final_path) {
                    Ok(_) => {
                        // 保存到撤销栈
                        let undo_action = UndoAction::Rename {
                            original_name: image_info.file_name.clone(),
                            new_name: final_path
                                .file_name()
                                .and_then(|n| n.to_str())
                                .unwrap_or(&new_name)
                                .to_string(),
                        };
                        self.undo_stack.push_back(undo_action);
                        cleared_count += 1;
                    }
                    Err(e) => {
                        errors.push(format!("{}: {}", image_info.file_name, e));
                    }
                }

                // 保持撤销栈大小限制
                if self.undo_stack.len() > self.max_undo_steps {
                    self.undo_stack.pop_front();
                }
            }

            // 刷新列表
            self.load_folder(folder_path.clone(), ctx);

            // 显示结果
            if errors.is_empty() {
                self.status_message = format!("已清除 {} 个文件名", cleared_count);
            } else {
                self.error_message = Some(format!(
                    "清除完成，但有 {} 个错误:\n{}",
                    errors.len(),
                    errors.join("\n")
                ));
                self.status_message =
                    format!("已清除 {} 个文件名，{} 个失败", cleared_count, errors.len());
            }
        }

        self.show_clear_dialog = false;
    }

    fn handle_name_conflict(&self, path: &std::path::Path) -> std::path::PathBuf {
        if !path.exists() {
            return path.to_path_buf();
        }

        let parent = path.parent().unwrap();
        let file_stem = path.file_stem().unwrap();
        let extension = path.extension().unwrap();

        let mut counter = 1;
        loop {
            let new_name = format!(
                "{}_{}.{}",
                file_stem.to_string_lossy(),
                counter,
                extension.to_string_lossy()
            );
            let new_path = parent.join(new_name);

            if !new_path.exists() {
                return new_path;
            }

            counter += 1;
            if counter > 1000 {
                return path.to_path_buf(); // 返回原路径，让重命名失败
            }
        }
    }

    fn select_folder(&mut self) {
        if self.state != AppState::Idle {
            return;
        }

        let tx = self.tx.clone();

        thread::spawn(move || {
            if let Some(folder_path) = rfd::FileDialog::new().pick_folder() {
                let _ = tx.send(AppMessage::FolderSelected(folder_path));
            }
        });

        self.status_message = "正在打开文件夹选择对话框...".to_string();
    }

    fn process_messages(&mut self, ctx: &egui::Context) {
        while let Ok(msg) = self.rx.try_recv() {
            match msg {
                AppMessage::FolderSelected(folder_path) => {
                    self.load_folder(folder_path, ctx);
                }
                AppMessage::LoadComplete(result) => match result {
                    Ok(batch_info) => {
                        self.batch_info = Some(batch_info.clone());
                        self.state = AppState::Idle;
                        self.status_message = format!(
                            "加载完成：共 {} 张图片，成功 {} 张，失败 {} 张",
                            batch_info.total_count,
                            batch_info.success_count,
                            batch_info.error_count
                        );
                        self.progress = 1.0;
                    }
                    Err(e) => {
                        self.state = AppState::Idle;
                        self.error_message = Some(format!("加载失败: {}", e));
                        self.status_message = "加载失败".to_string();
                    }
                },
                AppMessage::RenameRequested => {
                    self.execute_rename(ctx);
                }
                AppMessage::RenameComplete(result) => {
                    match result {
                        Ok(rename_results) => {
                            self.state = AppState::Idle;
                            self.status_message = "批量重命名完成".to_string();
                            self.progress = 1.0;

                            // 保存撤销信息
                            for (original_name, result_msg) in &rename_results {
                                if result_msg.starts_with("成功 -> ") {
                                    let new_name = result_msg.trim_start_matches("成功 -> ");
                                    let undo_action = UndoAction::Rename {
                                        original_name: original_name.clone(),
                                        new_name: new_name.to_string(),
                                    };
                                    self.undo_stack.push_back(undo_action);

                                    // 保持撤销栈大小限制
                                    if self.undo_stack.len() > self.max_undo_steps {
                                        self.undo_stack.pop_front();
                                    }
                                }
                            }

                            // 刷新列表
                            if let Some(folder_path) = &self.folder_path {
                                self.load_folder(folder_path.clone(), ctx);
                            }
                        }
                        Err(e) => {
                            self.state = AppState::Idle;
                            self.error_message = Some(format!("重命名失败: {}", e));
                            self.status_message = "重命名失败".to_string();
                        }
                    }
                }
                AppMessage::ExportRequested => {
                    self.execute_export(ctx);
                }
                AppMessage::ExportComplete(result) => match result {
                    Ok(_) => {
                        self.state = AppState::Idle;
                        self.status_message = "CSV导出完成".to_string();
                        self.progress = 1.0;
                    }
                    Err(e) => {
                        self.state = AppState::Idle;
                        self.error_message = Some(format!("导出失败: {}", e));
                        self.status_message = "导出失败".to_string();
                    }
                },
                AppMessage::ConvertRequested(target_mode) => {
                    self.execute_convert(&target_mode, ctx);
                }
                AppMessage::ConvertComplete(result) => {
                    match result {
                        Ok(_) => {
                            self.state = AppState::Idle;
                            self.status_message = "色彩转换完成".to_string();
                            self.progress = 1.0;
                            // 刷新列表
                            if let Some(folder_path) = &self.folder_path {
                                self.load_folder(folder_path.clone(), ctx);
                            }
                        }
                        Err(e) => {
                            self.state = AppState::Idle;
                            self.error_message = Some(format!("转换失败: {}", e));
                            self.status_message = "转换失败".to_string();
                        }
                    }
                }
                AppMessage::RefreshRequested => {
                    if let Some(folder_path) = &self.folder_path {
                        self.load_folder(folder_path.clone(), ctx);
                    }
                }
                AppMessage::UndoRequested => {
                    self.execute_undo(ctx);
                }
                AppMessage::UndoComplete(result) => {
                    match result {
                        Ok(_) => {
                            self.state = AppState::Idle;
                            self.status_message = "撤销完成".to_string();
                            self.progress = 1.0;
                            // 刷新列表
                            if let Some(folder_path) = &self.folder_path {
                                self.load_folder(folder_path.clone(), ctx);
                            }
                        }
                        Err(e) => {
                            self.state = AppState::Idle;
                            self.error_message = Some(format!("撤销失败: {}", e));
                            self.status_message = "撤销失败".to_string();
                        }
                    }
                }
                AppMessage::Error(e) => {
                    self.state = AppState::Idle;
                    self.error_message = Some(e);
                }
            }
        }
    }

    fn load_folder(&mut self, folder_path: PathBuf, ctx: &egui::Context) {
        if self.state != AppState::Idle {
            return;
        }

        // 取消任何正在进行的编辑
        self.cancel_editing();

        self.state = AppState::Loading;
        self.folder_path = Some(folder_path.clone());
        self.status_message = format!("正在加载图片信息: {}", folder_path.display());
        self.progress = 0.0;
        self.batch_info = None;
        self.selected_image = None;
        self.preview_texture = None;

        let ctx_clone = ctx.clone();
        let tx = self.tx.clone();

        thread::spawn(move || {
            let result = scan_folder(&folder_path).map_err(|e| e.to_string());

            let _ = tx.send(AppMessage::LoadComplete(result));
            ctx_clone.request_repaint();
        });
    }

    fn execute_rename(&mut self, ctx: &egui::Context) {
        if self.state != AppState::Idle || self.folder_path.is_none() || self.batch_info.is_none() {
            return;
        }

        let folder_path = self.folder_path.clone().unwrap();
        let batch_info = self.batch_info.clone().unwrap();

        self.state = AppState::Renaming;
        self.status_message = "正在批量重命名...".to_string();
        self.progress = 0.0;

        let ctx_clone = ctx.clone();
        let tx = self.tx.clone();

        thread::spawn(move || {
            let result =
                batch_rename_by_size(&folder_path, &batch_info.images).map_err(|e| e.to_string());

            let _ = tx.send(AppMessage::RenameComplete(result));
            ctx_clone.request_repaint();
        });
    }

    fn execute_export(&mut self, ctx: &egui::Context) {
        if self.state != AppState::Idle || self.batch_info.is_none() {
            return;
        }

        self.state = AppState::Exporting;
        self.status_message = "正在选择保存位置...".to_string();
        self.progress = 0.0;

        let ctx_clone = ctx.clone();
        let batch_info = self.batch_info.clone().unwrap();
        let tx = self.tx.clone();

        thread::spawn(move || {
            if let Some(save_path) = rfd::FileDialog::new()
                .add_filter("CSV文件", &["csv"])
                .set_file_name("图片信息.csv")
                .save_file()
            {
                let result = export_to_excel(&batch_info, &save_path).map_err(|e| e.to_string());

                let _ = tx.send(AppMessage::ExportComplete(result));
            } else {
                let _ = tx.send(AppMessage::Error("取消导出".to_string()));
            }
            ctx_clone.request_repaint();
        });
    }

    fn execute_undo(&mut self, ctx: &egui::Context) {
        if self.state != AppState::Idle || self.folder_path.is_none() || self.undo_stack.is_empty()
        {
            return;
        }

        let folder_path = self.folder_path.clone().unwrap();

        self.state = AppState::Undoing;
        self.status_message = "正在撤销操作...".to_string();
        self.progress = 0.0;

        let ctx_clone = ctx.clone();
        let tx = self.tx.clone();

        // 获取最后一个撤销操作
        let undo_action = self.undo_stack.pop_back().unwrap();

        thread::spawn(move || {
            let result = match undo_action {
                UndoAction::Rename {
                    original_name,
                    new_name,
                } => {
                    // 撤销重命名：将new_name改回original_name
                    let new_path = folder_path.join(&new_name);
                    let original_path = folder_path.join(&original_name);

                    if new_path.exists() {
                        match std::fs::rename(&new_path, &original_path) {
                            Ok(_) => Ok(()),
                            Err(e) => Err(format!("撤销重命名失败: {}", e)),
                        }
                    } else {
                        Err(format!("文件不存在: {}", new_path.display()))
                    }
                }
            };

            let _ = tx.send(AppMessage::UndoComplete(result));
            ctx_clone.request_repaint();
        });
    }

    fn execute_convert(&mut self, target_mode: &str, ctx: &egui::Context) {
        if self.state != AppState::Idle || self.folder_path.is_none() || self.batch_info.is_none() {
            return;
        }

        let folder_path = self.folder_path.clone().unwrap();
        let batch_info = self.batch_info.clone().unwrap();

        self.state = AppState::Converting;
        self.status_message = format!("正在转换为 {}...", target_mode);
        self.progress = 0.0;

        let ctx_clone = ctx.clone();
        let target_mode = target_mode.to_string();
        let tx = self.tx.clone();

        thread::spawn(move || {
            // 创建输出文件夹
            let output_folder = folder_path.join(format!("converted_{}", target_mode));

            let result = batch_convert_color_mode(
                &folder_path,
                &output_folder,
                &batch_info.images,
                &target_mode,
            )
            .map(|_| ())
            .map_err(|e| e.to_string());

            let _ = tx.send(AppMessage::ConvertComplete(result));
            ctx_clone.request_repaint();
        });
    }

    fn update_preview(&mut self, ctx: &egui::Context) {
        if let (Some(idx), Some(batch_info)) = (self.selected_image, &self.batch_info) {
            if idx < batch_info.images.len() {
                let image_info = &batch_info.images[idx];

                if let Some(thumbnail_data) = &image_info.thumbnail_data {
                    // 加载缩略图纹理
                    let image = image::load_from_memory(thumbnail_data);
                    if let Ok(image) = image {
                        let size = [image.width() as usize, image.height() as usize];
                        let image_buffer = image.to_rgba8();

                        let texture = ctx.load_texture(
                            "preview",
                            egui::ColorImage::from_rgba_unmultiplied(size, image_buffer.as_raw()),
                            egui::TextureOptions::LINEAR,
                        );

                        self.preview_texture = Some(texture);
                    }
                }
            }
        }
    }
}

impl eframe::App for ImageToolApp {
    fn update(&mut self, ctx: &egui::Context, _frame: &mut eframe::Frame) {
        // 处理消息
        self.process_messages(ctx);

        // 1. 顶部工具栏
        egui::TopBottomPanel::top("toolbar").show(ctx, |ui| {
            ui.horizontal(|ui| {
                let is_busy = self.state != AppState::Idle;

                if ui
                    .add_enabled(!is_busy, egui::Button::new("选择文件夹"))
                    .clicked()
                {
                    self.select_folder();
                }

                ui.add_enabled_ui(self.batch_info.is_some() && !is_busy, |ui| {
                    if ui.button("批量重命名").clicked() {
                        let _ = self.tx.send(AppMessage::RenameRequested);
                    }

                    if ui.button("导出到CSV").clicked() {
                        let _ = self.tx.send(AppMessage::ExportRequested);
                    }

                    if ui.button("转为CMYK").clicked() {
                        let _ = self
                            .tx
                            .send(AppMessage::ConvertRequested("CMYK".to_string()));
                    }

                    if ui.button("转为RGB").clicked() {
                        let _ = self
                            .tx
                            .send(AppMessage::ConvertRequested("RGB".to_string()));
                    }

                    if ui.button("刷新").clicked() {
                        let _ = self.tx.send(AppMessage::RefreshRequested);
                    }

                    // 撤销按钮
                    let can_undo = !self.undo_stack.is_empty();
                    if ui
                        .add_enabled(can_undo && !is_busy, egui::Button::new("撤销"))
                        .clicked()
                    {
                        let _ = self.tx.send(AppMessage::UndoRequested);
                    }

                    // 一键清除文件名标签
                    if ui
                        .add_enabled(
                            self.batch_info.is_some() && !is_busy,
                            egui::Button::new("一键清除文件名"),
                        )
                        .clicked()
                    {
                        self.show_clear_dialog = true;
                    }
                });
            });
        });

        // 2. 底部状态栏
        egui::TopBottomPanel::bottom("status_bar").show(ctx, |ui| {
            ui.horizontal(|ui| {
                ui.label(&self.status_message);

                if self.state != AppState::Idle {
                    ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
                        ui.spinner();
                    });
                }
            });
        });

        // 3. 右侧预览面板
        egui::SidePanel::right("preview_panel")
            .resizable(true)
            .default_width(350.0)
            .min_width(200.0)
            .show(ctx, |ui| {
                ui.vertical(|ui| {
                    ui.label(egui::RichText::new("图片预览").heading().strong());
                    ui.separator();

                    if let Some(texture) = &self.preview_texture {
                        let available_size = ui.available_size();
                        let (rect, _response) =
                            ui.allocate_exact_size(available_size, egui::Sense::hover());

                        ui.allocate_ui_at_rect(rect, |ui| {
                            ui.with_layout(
                                egui::Layout::right_to_left(egui::Align::Center),
                                |ui| {
                                    let texture_size = texture.size_vec2();

                                    // Prevent division by zero
                                    if texture_size.x > 0.0 && texture_size.y > 0.0 {
                                        // Calculate scale factor to fit within available space keeping aspect ratio
                                        let width_scale = rect.width() / texture_size.x;
                                        let height_scale = rect.height() / texture_size.y;
                                        // Shrink by 1/3 (show at ~66% of max fit size)
                                        let scale = width_scale.min(height_scale) * 0.66;

                                        let new_size = texture_size * scale;

                                        // Add some right padding
                                        ui.add_space(30.0);

                                        ui.add(
                                            egui::Image::new(texture).fit_to_exact_size(new_size),
                                        );
                                    }
                                },
                            );
                        });
                    } else {
                        ui.centered_and_justified(|ui| {
                            ui.label("请选择图片预览");
                        });
                    }
                });
            });

        // 4. 中间列表面板
        egui::CentralPanel::default().show(ctx, |ui| {
            ui.heading("图片列表");

            if self.state == AppState::Loading {
                ui.centered_and_justified(|ui| {
                    ui.spinner();
                    ui.label(&self.status_message);
                });
            } else if self.batch_info.is_none() {
                ui.centered_and_justified(|ui| {
                    ui.label("请先选择文件夹加载图片");
                });
            } else if let Some(batch_info) = &self.batch_info {
                let batch_info_clone = batch_info.clone();
                let selected_image = self.selected_image;
                let ctx_clone = ctx.clone();

                let table = TableBuilder::new(ui)
                    .striped(true)
                    .resizable(true)
                    .cell_layout(egui::Layout::left_to_right(egui::Align::Center))
                    .column(Column::auto().at_least(200.0)) // 文件名
                    .column(Column::auto().at_least(80.0)) // 像素尺寸
                    .column(Column::auto().at_least(80.0)) // 物理尺寸
                    .column(Column::auto().at_least(60.0)) // DPI
                    .column(Column::auto().at_least(80.0)) // 文件大小
                    .column(Column::auto().at_least(60.0)) // 文件格式
                    .column(Column::auto().at_least(60.0)) // CMYK/RGB
                    .header(20.0, |mut header| {
                        header.col(|ui| {
                            ui.heading("文件名");
                        });
                        header.col(|ui| {
                            ui.heading("像素尺寸");
                        });
                        header.col(|ui| {
                            ui.heading("物理尺寸(cm)");
                        });
                        header.col(|ui| {
                            ui.heading("DPI");
                        });
                        header.col(|ui| {
                            ui.heading("文件大小");
                        });
                        header.col(|ui| {
                            ui.heading("文件格式");
                        });
                        header.col(|ui| {
                            ui.heading("CMYK/RGB");
                        });
                    });

                table.body(|mut body| {
                    for (idx, image_info) in batch_info_clone.images.iter().enumerate() {
                        let row_selected = selected_image == Some(idx);

                        body.row(30.0, |mut row| {
                            row.set_selected(row_selected);

                            // 文件名列
                            row.col(|ui| {
                                if self.editing_row == Some(idx) {
                                    // 编辑模式
                                    let response = ui.add(
                                        egui::TextEdit::singleline(&mut self.edit_buffer)
                                            .desired_width(200.0)
                                            .frame(true),
                                    );

                                    if response.lost_focus()
                                        || ui.input(|i| i.key_pressed(egui::Key::Enter))
                                    {
                                        if self.save_editing(&ctx_clone) {
                                            self.status_message =
                                                format!("已重命名: {}", self.edit_buffer);
                                        }
                                    }

                                    if ui.input(|i| i.key_pressed(egui::Key::Escape)) {
                                        self.cancel_editing();
                                    }

                                    response.request_focus();
                                } else {
                                    let label_response =
                                        ui.selectable_label(row_selected, &image_info.file_name);

                                    if label_response.clicked() {
                                        self.selected_image = Some(idx);
                                        self.update_preview(&ctx_clone);
                                    }

                                    if label_response.double_clicked() {
                                        self.start_editing(idx);
                                    }

                                    label_response.context_menu(|ui| {
                                        if ui.button("重命名").clicked() {
                                            self.start_editing(idx);
                                            ui.close_menu();
                                        }
                                    });
                                }
                            });

                            // 其他列
                            row.col(|ui| {
                                if image_info.error.is_none() {
                                    ui.label(image_info.pixel_size_str());
                                } else {
                                    ui.label("无法读取");
                                }
                            });

                            row.col(|ui| {
                                if image_info.error.is_none() {
                                    ui.label(image_info.physical_size_str());
                                } else {
                                    ui.label("");
                                }
                            });

                            row.col(|ui| {
                                if image_info.error.is_none() {
                                    ui.label(image_info.dpi_str());
                                } else {
                                    ui.label("");
                                }
                            });

                            row.col(|ui| {
                                if image_info.error.is_none() {
                                    ui.label(image_info.file_size_str());
                                } else {
                                    ui.label("");
                                }
                            });

                            row.col(|ui| {
                                if image_info.error.is_none() {
                                    let extension = image_info.file_extension();
                                    let format = if !extension.is_empty() {
                                        extension
                                    } else {
                                        image_info.format.clone()
                                    };
                                    ui.label(format);
                                } else {
                                    ui.label("");
                                }
                            });

                            row.col(|ui| {
                                if image_info.error.is_none() {
                                    ui.label(image_info.color_mode_simple());
                                } else {
                                    ui.label("");
                                }
                            });
                        });
                    }
                });
            }
        });

        // 显示清除确认对话框
        if self.show_clear_dialog {
            egui::Window::new("确认清除")
                .collapsible(false)
                .resizable(false)
                .anchor(egui::Align2::CENTER_CENTER, [0.0, 0.0])
                .show(ctx, |ui| {
                    ui.heading("⚠️ 警告");
                    ui.label("此操作将清除所有图片的文件名，替换为'未命名'。");
                    ui.label("此操作不可逆，建议先备份文件！");
                    ui.separator();

                    ui.horizontal(|ui| {
                        if ui.button("取消").clicked() {
                            self.show_clear_dialog = false;
                        }

                        if ui.button("确认清除").clicked() {
                            self.clear_all_filenames(ctx);
                        }
                    });
                });
        }

        // 显示错误消息
        if let Some(error_msg) = &self.error_message {
            let error_msg_clone = error_msg.clone();
            egui::Window::new("错误")
                .collapsible(false)
                .resizable(false)
                .anchor(egui::Align2::CENTER_CENTER, [0.0, 0.0])
                .show(ctx, |ui| {
                    ui.label(&error_msg_clone);
                    if ui.button("确定").clicked() {
                        self.error_message = None;
                    }
                });
        }

        // 如果正在处理，请求重绘
        if self.state != AppState::Idle {
            ctx.request_repaint();
        }
    }
}
