// ⭐ 在 Windows 上隐藏命令行窗口
#![windows_subsystem = "windows"]

use eframe::egui;
use egui_plot::{Legend, Line, Plot, PlotPoints};
use hound::WavReader;
use rfd::FileDialog;
use rodio::{Decoder, OutputStream, OutputStreamHandle, Sink};
use std::error::Error;
use std::fs::File;
use std::io::BufReader;
use std::path::PathBuf;
use std::sync::{Arc, Mutex};
use std::thread;
use std::sync::mpsc;
use std::time::Duration;
use chrono::Local;
use csv;


// --- 语言和国际化结构 ---

/// 定义支持的语言
#[derive(PartialEq, Clone, Copy, Debug)]
enum Language {
    Chinese,
    English,
}

/// 包含所有 UI 文本的结构体
struct Lang {
    // ⭐ 新增：导航栏/全局 UI 文本
    nav_lang_label: &'static str,
    nav_zh_label: &'static str,
    nav_en_label: &'static str,
    nav_help_btn: &'static str,

    // 菜单/模式
    mode_single: &'static str,
    mode_compare: &'static str,
    mode_console: &'static str, // 控制台模式

    // 状态栏
    status_loading: &'static str,
    status_ready: &'static str,

    // 单文件模式
    single_open_btn: &'static str,
    single_clear_btn: &'static str,
    single_y_label: &'static str,
    single_x_label: &'static str,

    // 归一化和导出
    export_csv_btn: &'static str,
    normalize_label: &'static str,
    normalize_apply: &'static str,

    // 对比模式
    #[allow(dead_code)]
    compare_heading: &'static str,
    compare_track_a_label: &'static str,
    compare_track_b_label: &'static str,
    compare_select_a: &'static str,
    compare_select_b: &'static str,
    compare_report_title: &'static str,
    compare_plot_raw_label: &'static str,
    compare_plot_diff_label: &'static str,
    compare_conf_label: &'static str,
    // compare_target_diff_label: &'static str, // (这个标签直接在 UI 中硬编码了)

    // 结果字符串格式
    compare_err_duration_fmt: &'static str,
    compare_avg_diff_fmt: &'static str,
    compare_std_dev_fmt: &'static str,
    compare_correlation_fmt: &'static str,
    compare_t_stat_fmt: &'static str,
    compare_t_test_significant: &'static str,
    compare_t_test_not_significant: &'static str,

    compare_max_diff_fmt: &'static str,
    compare_min_diff_fmt: &'static str,

    // 状态结果
    compare_high_match: &'static str,
    compare_mid_diff: &'static str,
    compare_huge_diff: &'static str,

    // --- 新增：帮助文本/悬浮窗 ---
    help_title: &'static str,
    help_desc: &'static str,
    help_monitor_title: &'static str,
    help_console_title: &'static str,
    help_cmd_list: &'static str,
    help_cmd_kill: &'static str,
    help_cmd_clear: &'static str,
    help_cmd_quit: &'static str,

    // --- 新增：控制台硬编码信息
    console_cmd_hint_cn: &'static str,
    console_cmd_label: &'static str,
    help_monitor_desc: &'static str,

    // --- 新增：播放器相关文本 ---
    player_play: &'static str,
    player_pause: &'static str,
    player_stop: &'static str,
    player_mute: &'static str,
    player_solo: &'static str,
    player_volume: &'static str,
    player_position: &'static str,
    #[allow(dead_code)]
    player_duration: &'static str,
    player_no_files: &'static str,

    // --- 新增：图表交互相关文本 ---
    help_plot_interaction_title: &'static str,
    help_plot_drag_desc: &'static str,
    help_plot_seek_desc: &'static str,

    // --- 新增：播放器快捷键文本 ---
    help_player_shortcuts_title: &'static str,
    help_space_key_desc: &'static str,
}

impl Lang {
    /// 根据语言加载字符串
    fn load(lang: Language) -> Self {
        match lang {
            // 中文 (zh_CN)
            Language::Chinese => Lang {
                // ⭐ 新增：导航栏/全局 UI 文本
                nav_lang_label: "语言:",
                nav_zh_label: "中文",
                nav_en_label: "English",
                nav_help_btn: "❓ 帮助",

                mode_single: "🎵 单机批处理模式",
                mode_compare: "⚖️ AB 对比模式",
                mode_console: "💻 控制台/日志",
                status_loading: "正在处理音频数据，请稍候...",
                status_ready: "就绪",
                single_open_btn: "📂 打开文件 (支持多选 WAV/CSV)",
                single_clear_btn: "🗑️ 清空列表",
                single_y_label: "Loudness (dBFS)",
                single_x_label: "Time (s)",
                export_csv_btn: "💾 导出为 CSV",
                normalize_label: "LUFS 归一化目标 (平均 dBFS):",
                normalize_apply: "应用归一化",
                compare_heading: "A/B 动态一致性检验",
                compare_track_a_label: "Track A (Ref):",
                compare_track_b_label: "Track B (Target):",
                compare_select_a: "📂 选择文件 A",
                compare_select_b: "📂 选择文件 B",
                compare_report_title: "分析报告",
                compare_plot_raw_label: "响度曲线对比 (A vs B)",
                compare_plot_diff_label: "差值稳定性 (Track A - Track B)",
                compare_conf_label: "假设检验置信度:",
                compare_err_duration_fmt: "❌ 时间差异过大 ({}s vs {}s)，无法进行逐点对比。",
                compare_avg_diff_fmt: "平均差异: {} dB",
                compare_std_dev_fmt: "动态标准差: {}",
                compare_correlation_fmt: "动态相关系数 (r): {}",
                compare_t_stat_fmt: "均值差值 T-统计量: {}",
                compare_t_test_significant: "❌ 均值差值显著",
                compare_t_test_not_significant: "✅ 均值差值不显著",
                compare_max_diff_fmt: "最大差值: {} dB",
                compare_min_diff_fmt: "最小差值: {} dB",
                compare_high_match: "✅ 动态一致性极高",
                compare_mid_diff: "⚠️ 动态存在差异",
                compare_huge_diff: "❌ 动态差异巨大",

                // 新增：帮助文本
                help_title: "📊 WAV 动态分析器帮助",
                help_desc: "本应用用于分析 WAV/CSV 文件的响度曲线 (LUFS/dBFS) 并进行归一化或动态一致性 (A/B) 比较。",
                help_monitor_title: "进程监视器",
                help_console_title: "控制台命令",
                help_cmd_list: "显示当前所有正在运行或已完成的后台任务。",
                help_cmd_kill: "发送终止信号给指定 ID 的任务。用法: kill <任务ID>",
                help_cmd_clear: "清空控制台日志。",
                help_cmd_quit: "发送关闭信号给工作池，准备退出应用。",

                // ⭐ 新增：控制台硬编码信息
                console_cmd_hint_cn: "可用命令: `tasks` (或 `list`) | `kill <ID>` | `clear` | `quit` (或 `exit`)",
                console_cmd_label: "CMD >",
                help_monitor_desc: "进程监视器（💻 控制台/日志模式）显示后台加载和分析任务的实时状态。",

                // --- 新增：播放器相关文本 ---
                player_play: "▶️ 播放",
                player_pause: "⏸️ 暂停",
                player_stop: "⏹️ 停止",
                player_mute: "M",
                player_solo: "S",
                player_volume: "音量",
                player_position: "位置",
                player_duration: "时长",
                player_no_files: "请加载WAV文件以使用播放器",

                // --- 新增：图表交互相关文本 ---
                help_plot_interaction_title: "📊 图表交互",
                help_plot_drag_desc: "• 左键拖拽：平移图表视图",
                help_plot_seek_desc: "• Shift + 左键拖拽/点击：调整播放进度",

                // --- 新增：播放器快捷键文本 ---
                help_player_shortcuts_title: "🎵 播放器快捷键",
                help_space_key_desc: "• 空格键：播放/暂停",
            },
            // 英文 (en_US)
            Language::English => Lang {
                // ⭐ 新增：导航栏/全局 UI 文本
                nav_lang_label: "Language:",
                nav_zh_label: "Chinese",
                nav_en_label: "English",
                nav_help_btn: "❓ Help",

                mode_single: "🎵 Single Batch Mode",
                mode_compare: "⚖️ A/B Comparison Mode",
                mode_console: "💻 Console/Log",
                status_loading: "Processing audio data, please wait...",
                status_ready: "Ready",
                single_open_btn: "📂 Open Files (WAV/CSV Multi-select)",
                single_clear_btn: "🗑️ Clear List",
                single_y_label: "Loudness (dBFS)",
                single_x_label: "Time (s)",
                export_csv_btn: "💾 Export to CSV",
                normalize_label: "LUFS Normalization Target (Avg dBFS):",
                normalize_apply: "Apply Normalization",
                compare_heading: "A/B Dynamic Consistency Check",
                compare_track_a_label: "Track A (Ref):",
                compare_track_b_label: "Track B (Target):",
                compare_select_a: "📂 Select File A",
                compare_select_b: "📂 Select File B",
                compare_report_title: "Analysis Report",
                compare_plot_raw_label: "Loudness Curve Comparison (A vs B)",
                compare_plot_diff_label: "Difference Stability (Track A - Track B)",
                compare_conf_label: "Hypothesis Test Confidence:",
                compare_err_duration_fmt: "❌ Duration difference too large ({}s vs {}s), unable to perform point-by-point comparison.",
                compare_avg_diff_fmt: "Average Difference: {} dB",
                compare_std_dev_fmt: "Dynamic Std Dev: {}",
                compare_correlation_fmt: "Dynamic Correlation (r): {}",
                compare_t_stat_fmt: "Mean Diff T-Statistic: {}",
                compare_t_test_significant: "❌ Mean Difference is Significant",
                compare_t_test_not_significant: "✅ Mean Difference is Not Significant",
                compare_max_diff_fmt: "Max Difference: {} dB",
                compare_min_diff_fmt: "Min Difference: {} dB",
                compare_high_match: "✅ High Dynamic Consistency",
                compare_mid_diff: "⚠️ Dynamic Differences Exist",
                compare_huge_diff: "❌ Huge Dynamic Difference",

                // 新增：帮助文本
                help_title: "📊 WAV Dynamics Analyzer Help",
                help_desc: "This application is used to analyze loudness curves (LUFS/dBFS) of WAV/CSV files and perform normalization or dynamic consistency (A/B) comparisons.",
                help_monitor_title: "Process Monitor",
                help_console_title: "Console Commands",
                help_cmd_list: "Show all currently running or completed background tasks.",
                help_cmd_kill: "Sends a termination signal to the task with the specified ID. Usage: kill <TaskID>",
                help_cmd_clear: "Clear the console log.",
                help_cmd_quit: "Sends a shutdown signal to the worker pool, preparing to exit the application.",

                // ⭐ 新增：控制台硬编码信息
                console_cmd_hint_cn: "Available commands: `tasks` (or `list`) | `kill <ID>` | `clear` | `quit` (or `exit`)",
                console_cmd_label: "CMD >",
                help_monitor_desc: "The process monitor (💻 Console/Log mode) shows the real-time status of background loading and analysis tasks.",

                // --- 新增：播放器相关文本 ---
                player_play: "▶️ Play",
                player_pause: "⏸️ Pause",
                player_stop: "⏹️ Stop",
                player_mute: "M",
                player_solo: "S",
                player_volume: "Volume",
                player_position: "Position",
                player_duration: "Duration",
                player_no_files: "Please load WAV files to use the player",

                // --- 新增：图表交互相关文本 ---
                help_plot_interaction_title: "📊 Plot Interaction",
                help_plot_drag_desc: "• Left-click drag: Pan chart view",
                help_plot_seek_desc: "• Shift + Left-click drag/click: Adjust playback position",

                // --- 新增：播放器快捷键文本 ---
                help_player_shortcuts_title: "🎵 Player Shortcuts",
                help_space_key_desc: "• Space key: Play/Pause",
            },
        }
    }
}


// --- 核心数据结构 ---

#[derive(Clone, Debug)]
struct AudioCurve {
    name: String,
    // (时间, dBFS)
    points: Vec<[f64; 2]>,
    duration: f64,
    average_dbfs: f64, // 用于计算归一化偏移
    file_path: PathBuf, // 新增：文件路径，用于播放
}

#[derive(Clone, Debug)]
struct Track {
    curve: AudioCurve,
    is_muted: bool,
    is_soloed: bool,
    volume: f32,
}

impl Track {
    fn new(curve: AudioCurve) -> Self {
        Self {
            curve,
            is_muted: false,
            is_soloed: false,
            volume: 1.0,
        }
    }

    fn should_play(&self, has_solo_track: bool) -> bool {
        if has_solo_track {
            self.is_soloed
        } else {
            !self.is_muted
        }
    }
}

struct AudioPlayer {
    tracks: Vec<Track>,
    is_playing: bool,
    current_position: f64,
    duration: f64,
    sinks: Vec<Sink>,
    stream: Option<OutputStream>,
    stream_handle: Option<OutputStreamHandle>,
}

impl AudioPlayer {
    fn new() -> Self {
        Self {
            tracks: Vec::new(),
            is_playing: false,
            current_position: 0.0,
            duration: 0.0,
            sinks: Vec::new(),
            stream: None,
            stream_handle: None,
        }
    }
    
    // ⭐ 新增: dB与线性音量转换函数
    fn db_to_linear(db: f32) -> f32 {
        if db <= -60.0 {
            0.0
        } else {
            10.0f32.powf(db / 20.0)
        }
    }

    fn add_track(&mut self, curve: AudioCurve) -> Result<(), Box<dyn Error>> {
        self.tracks.push(Track::new(curve));
        self.update_duration();
        Ok(())
    }

    fn clear_tracks(&mut self) {
        self.stop();
        self.tracks.clear();
        self.duration = 0.0;
        self.current_position = 0.0;
    }

    fn update_duration(&mut self) {
        self.duration = self.tracks.iter()
            .map(|t| t.curve.duration)
            .fold(0.0, |a, b| a.max(b));
    }

    fn has_solo_track(&self) -> bool {
        self.tracks.iter().any(|t| t.is_soloed)
    }

    fn play(&mut self) -> Result<(), Box<dyn Error>> {
        if self.tracks.is_empty() {
            return Ok(());
        }

        let has_solo = self.has_solo_track();

        let (stream, stream_handle) = OutputStream::try_default()?;
        self.stream = Some(stream);
        self.stream_handle = Some(stream_handle);

        self.sinks.clear();

        for track in &self.tracks {
            if track.should_play(has_solo) {
                let file = File::open(&track.curve.file_path)?;
                let source = Decoder::new(BufReader::new(file))?;
                let sink = Sink::try_new(&self.stream_handle.as_ref().unwrap())?;
                sink.append(source);
                sink.set_volume(track.volume);
                
                // 确保所有音轨从同一位置开始播放
                if self.current_position > 0.0 {
                    let _ = sink.try_seek(std::time::Duration::from_secs_f64(self.current_position));
                }
                
                sink.pause();
                self.sinks.push(sink);
            }
        }

        for sink in &self.sinks {
            sink.play();
        }

        self.is_playing = true;
        Ok(())
    }

    fn pause(&mut self) {
        // 在暂停前保存当前播放位置
        if !self.sinks.is_empty() {
            self.current_position = self.get_position();
        }
        
        for sink in &self.sinks {
            sink.pause();
        }
        self.is_playing = false;
    }

    fn stop(&mut self) {
        for sink in &self.sinks {
            sink.stop();
        }
        self.sinks.clear();
        self.stream = None;
        self.stream_handle = None;
        self.is_playing = false;
        self.current_position = 0.0;
    }

    fn set_track_volume(&mut self, index: usize, volume: f32) {
        if index < self.tracks.len() && index < self.sinks.len() {
            self.tracks[index].volume = volume;
            self.sinks[index].set_volume(volume);
        }
    }
    
    // ⭐ 新增: 使用dB值设置轨道音量
    fn set_track_volume_db(&mut self, index: usize, db_adjustment: f32) {
        if index < self.tracks.len() && index < self.sinks.len() {
            // 将dB增量转换为线性乘数
            let linear_multiplier = Self::db_to_linear(db_adjustment);
            // 应用到当前音量
            let new_volume = (self.tracks[index].volume * linear_multiplier).clamp(0.0, 1.0);
            self.tracks[index].volume = new_volume;
            self.sinks[index].set_volume(new_volume);
        }
    }

    fn set_track_mute(&mut self, index: usize, muted: bool) {
        if index < self.tracks.len() {
            self.tracks[index].is_muted = muted;
        }
    }

    fn set_track_solo(&mut self, index: usize, soloed: bool) {
        if index < self.tracks.len() {
            self.tracks[index].is_soloed = soloed;
        }
    }

    fn get_position(&self) -> f64 {
        if let Some(sink) = self.sinks.first() {
            // 直接获取播放位置
            let position = sink.get_pos();
            position.as_secs_f64()
        } else {
            self.current_position
        }
    }

    fn seek_to(&mut self, position: f64) -> Result<(), Box<dyn Error>> {
        let duration = std::time::Duration::from_secs_f64(position);
        for sink in &self.sinks {
            sink.try_seek(duration)?;
        }
        self.current_position = position;
        Ok(())
    }

    fn update(&mut self) {
        if self.is_playing && !self.sinks.is_empty() {
            // 检查第一个sink是否为空（播放结束）
            if let Some(first_sink) = self.sinks.first() {
                if first_sink.empty() {
                    self.stop();
                    return;
                }
            }
            
            // 更新当前位置（使用第一个sink的位置作为参考）
            self.current_position = self.get_position();
            
            // 确保所有sink保持同步
            let target_position = std::time::Duration::from_secs_f64(self.current_position);
            for sink in &self.sinks {
                // 只有当位置差异超过阈值时才进行同步，避免频繁seek
                let current_pos = sink.get_pos();
                let diff = (current_pos.as_secs_f64() - self.current_position).abs();
                if diff > 0.1 { // 100ms阈值
                    let _ = sink.try_seek(target_position);
                }
            }
        }
    }
}

#[derive(Clone, Debug)]
struct ComparisonResult {
    mean_diff: f64,
    std_dev: f64,
    max_diff: f64,
    min_diff: f64,
    correlation_coefficient: f64, // Pearson r
    t_statistic: f64,             // T-stat for mean difference vs target
    // (时间, 差值)
    diff_points: Vec<[f64; 2]>,
}

#[derive(PartialEq, Clone, Copy)]
enum AppMode {
    Single,
    Compare,
    Console,
}

// --- 日志系统 ---

struct LogEntry {
    time: String,
    message: String,
    level: LogLevel,
}

#[derive(PartialEq, Clone, Copy, Debug)]
enum LogLevel {
    Info,
    Error,
    Debug,
    Command, // 命令行输入或操作
}

struct Logger {
    entries: Arc<Mutex<Vec<LogEntry>>>,
}

impl Logger {
    fn new() -> Self {
        Self { entries: Arc::new(Mutex::new(Vec::new())) }
    }

    /// 记录一条日志，线程安全
    fn log(&self, level: LogLevel, message: String) {
        let time = Local::now().format("%H:%M:%S").to_string();
        let entry = LogEntry { time, message, level };
        if let Ok(mut entries) = self.entries.lock() {
            entries.push(entry);
            // 限制日志条目数量
            if entries.len() > 1000 {
                entries.drain(0..500);
            }
        }
    }
}

// 辅助函数，方便记录日志
fn log_info(logger: &Logger, msg: &str) {
    logger.log(LogLevel::Info, msg.to_string());
}

fn log_error(logger: &Logger, msg: &str) {
    logger.log(LogLevel::Error, msg.to_string());
}

fn log_debug(logger: &Logger, msg: &str) {
    logger.log(LogLevel::Debug, msg.to_string());
}

fn log_command(logger: &Logger, msg: &str) {
    logger.log(LogLevel::Command, msg.to_string());
}

// --- 任务管理结构 ---

#[derive(Clone, Debug, PartialEq)]
enum TaskState {
    Waiting,
    Running(f32), // 0.0 - 1.0 进度
    Completed,
    Killed,
    Error(String),
}

#[derive(Clone, Debug)]
struct AudioTask {
    id: usize,
    name: String,
    state: TaskState,
}

// UI 线程发送给 WorkerPool 主线程的命令
#[derive(Debug)]
enum WorkerCommand {
    Kill(usize), // 杀死指定 ID 的任务
    Shutdown,    // 关闭所有 worker
}

// Worker/Task 线程发送给 UI 线程的消息
enum WorkerMessage {
    Log(LogEntry),
    UpdateTaskState(usize, TaskState),
    NewCurve(AudioCurve, Option<char>), // 专门用于返回处理结果
}

struct WorkerPool {
    tasks: Arc<Mutex<Vec<AudioTask>>>, // 共享任务列表
    next_id: usize,
    command_tx: mpsc::Sender<WorkerCommand>, // UI -> Worker 命令发送端
    _worker_handle: thread::JoinHandle<()>,   // Worker 管理线程句柄
}

impl WorkerPool {
    fn new(ui_tx: mpsc::Sender<WorkerMessage>) -> Self {
        let (command_tx, command_rx) = mpsc::channel();
        let tasks = Arc::new(Mutex::new(Vec::<AudioTask>::new()));
        let tasks_clone = tasks.clone();
        let ui_tx_clone = ui_tx.clone();

        // 启动 WorkerPool 管理线程 (非阻塞)
        let _worker_handle = thread::spawn(move || {
            loop {
                // 1. 检查来自 UI 的命令
                match command_rx.try_recv() {
                    Ok(WorkerCommand::Kill(id)) => {
                        if let Ok(mut tasks_lock) = tasks_clone.lock() {
                            if let Some(task) = tasks_lock.iter_mut().find(|t| t.id == id && t.state != TaskState::Completed && t.state != TaskState::Killed) {
                                // 在任务列表中标记为 Killed
                                task.state = TaskState::Killed;
                                ui_tx_clone.send(WorkerMessage::UpdateTaskState(id, TaskState::Killed)).unwrap_or_default();

                                // 记录到日志
                                ui_tx_clone.send(WorkerMessage::Log(LogEntry {
                                    time: Local::now().format("%H:%M:%S").to_string(),
                                    message: format!("Command: Task {} ({}) marked for kill. (Note: Actual thread termination is not guaranteed in std::thread)", id, task.name),
                                    level: LogLevel::Command,
                                })).unwrap_or_default();
                            }
                        }
                    }
                    Ok(WorkerCommand::Shutdown) => {
                        ui_tx_clone.send(WorkerMessage::Log(LogEntry {
                            time: Local::now().format("%H:%M:%S").to_string(),
                            message: "WorkerPool received Shutdown command. Exiting.".to_string(),
                            level: LogLevel::Debug,
                        })).unwrap_or_default();
                        break;
                    }
                    Err(mpsc::TryRecvError::Empty) => {
                        // 无命令，继续
                    }
                    Err(mpsc::TryRecvError::Disconnected) => break, // 通道断开
                }

                thread::sleep(Duration::from_millis(100));
            }
        });

        Self {
            tasks,
            next_id: 1,
            command_tx,
            _worker_handle,
        }
    }

    /// 启动一个后台任务
    fn spawn_task<F>(&mut self, name: String, f: F, ui_tx: mpsc::Sender<WorkerMessage>, logger: &Logger)
    where
        F: FnOnce(usize, mpsc::Sender<WorkerMessage>, Arc<Mutex<Vec<LogEntry>>>) + Send + 'static,
    {
        let id = self.next_id;
        self.next_id += 1;
        let task_name = name.clone();

        // 传递日志条目 Arc<Mutex<...>> 的克隆给工作线程
        let logger_entries_clone = logger.entries.clone();
        let ui_tx_clone = ui_tx.clone();

        // 1. 记录初始状态
        log_info(logger, &format!("⚙️ 任务 {} 启动: {}", id, task_name));

        let initial_task = AudioTask {
            id,
            name: task_name.clone(),
            state: TaskState::Running(0.0),
        };

        // 2. 启动实际工作线程
        thread::spawn(move || {
            ui_tx_clone.send(WorkerMessage::UpdateTaskState(id, TaskState::Running(0.0))).unwrap_or_default();

            // 执行实际任务
            f(id, ui_tx_clone.clone(), logger_entries_clone.clone());

            // 任务完成，发送最终状态 (这里仅作为兜底，实际应在 f 中发送 Completed/Error/Killed)
            ui_tx_clone.send(WorkerMessage::UpdateTaskState(id, TaskState::Completed)).unwrap_or_default();

            let thread_logger = Logger { entries: logger_entries_clone };
            log_info(&thread_logger, &format!("✔️ 任务 {} 完成: {}", id, task_name));

        });

        // 3. 存储任务信息
        if let Ok(mut tasks_lock) = self.tasks.lock() {
            tasks_lock.push(initial_task);
        }
    }
}


// --- 音频处理逻辑 (更新: 增加 Logger 参数) ---

fn calculate_rms_dbfs(samples: &[f64]) -> f64 {
    if samples.is_empty() { return -120.0; }
    let squared_sum: f64 = samples.iter().map(|s| s * s).sum();
    let rms = (squared_sum / samples.len() as f64).sqrt();
    if rms < 1e-9 { -120.0 } else { 20.0 * rms.log10() }
}

/// 计算 Pearson 相关系数 (r)
fn calculate_correlation(a_vals: &[f64], b_vals: &[f64], len: usize) -> f64 {
    if len <= 1 { return 0.0; }

    let mean_a = a_vals.iter().sum::<f64>() / len as f64;
    let mean_b = b_vals.iter().sum::<f64>() / len as f64;

    let mut numerator = 0.0;
    let mut sum_sq_a = 0.0;
    let mut sum_sq_b = 0.0;

    for i in 0..len {
        let dev_a = a_vals[i] - mean_a;
        let dev_b = b_vals[i] - mean_b;

        numerator += dev_a * dev_b;
        sum_sq_a += dev_a * dev_a;
        sum_sq_b += dev_b * dev_b;
    }

    let denominator = (sum_sq_a * sum_sq_b).sqrt();

    if denominator == 0.0 {
        return 0.0;
    }
    numerator / denominator
}

/// 计算单样本 T 统计量 (检验均值差值是否为 0/C)
/// mean_difference 应该传入 (实际均值差 - 目标差值)
fn calculate_t_statistic(mean_difference: f64, std_dev: f64, n: usize) -> f64 {
    if n <= 1 || std_dev.abs() < f64::EPSILON {
        return 0.0;
    }
    // 标准误差 (SEM) = std_dev / sqrt(n)
    let sem = std_dev / (n as f64).sqrt();

    // T = (Mean - Target) / SEM
    mean_difference / sem
}


/// 【已修复】解析 WAV 文件，支持 16/24/32-bit PCM 和 32-bit Float 格式。
fn parse_wav(path: PathBuf, logger: &Logger) -> Result<AudioCurve, Box<dyn Error + Send + Sync>> {
    let filename = path.file_name().unwrap().to_string_lossy().to_string();
    log_info(logger, &format!("▶️ 开始解析 WAV 文件: {}", filename));

    let mut reader = WavReader::open(&path)?;
    let spec = reader.spec();

    log_debug(logger, &format!("WAV Spec: Rate={}Hz, Channels={}, Bits={}, Format={:?}", spec.sample_rate, spec.channels, spec.bits_per_sample, spec.sample_format));

    // 根据 WAV 文件的格式规范读取并归一化样本
    let samples: Vec<f64> = match (spec.sample_format, spec.bits_per_sample) {
        // 16-bit Integer PCM (Read as i16, max value is 2^15)
        (hound::SampleFormat::Int, 16) => {
            let max_val = 1u32 << 15;
            reader.samples::<i16>()
                .filter_map(|s| s.ok())
                .map(|s| s as f64 / max_val as f64)
                .collect()
        }
        // 24-bit Integer PCM (Read as i32, max value is 2^23)
        (hound::SampleFormat::Int, 24) => {
            let max_val = 1u32 << 23;
            reader.samples::<i32>()
                .filter_map(|s| s.ok())
                .map(|s| s as f64 / max_val as f64)
                .collect()
        }
        // 32-bit Integer PCM (Read as i32, max value is 2^31)
        (hound::SampleFormat::Int, 32) => {
            let max_val = 1u64 << 31;
            reader.samples::<i32>()
                .filter_map(|s| s.ok())
                .map(|s| s as f64 / max_val as f64)
                .collect()
        }
        // 32-bit Float (Read as f32, already normalized [-1.0, 1.0])
        (hound::SampleFormat::Float, 32) => {
            reader.samples::<f32>()
                .filter_map(|s| s.ok())
                .map(|s| s as f64)
                .collect()
        }
        // Fallback for unsupported formats
        _ => {
            let msg = format!(
                "❌ 不支持的 WAV 格式: Format={:?}, Bits={}",
                spec.sample_format, spec.bits_per_sample
            );
            log_error(logger, &msg);
            return Err(Box::new(std::io::Error::new(
                std::io::ErrorKind::InvalidData,
                msg,
            )));
        }
    };

    if samples.is_empty() {
        return Err(Box::new(std::io::Error::new(std::io::ErrorKind::InvalidData, "WAV 文件没有可用的样本数据")));
    }
    log_debug(logger, &format!("总样本数: {}", samples.len()));

    let window_sec = 0.4;
    let step_sec = 0.1;
    let sample_rate = spec.sample_rate as usize;
    let channels = spec.channels as usize;

    let window_size = (window_sec * sample_rate as f64) as usize;
    let step_size = (step_sec * sample_rate as f64) as usize;

    if window_size * channels == 0 || step_size * channels == 0 {
        log_error(logger, "⚠️ 窗口/步进尺寸计算为 0，跳过曲线生成。");
        return Err(Box::new(std::io::Error::new(std::io::ErrorKind::InvalidData, "计算窗口大小错误")));
    }

    let mut points = Vec::new();
    let mut dbfs_sum = 0.0;
    let mut i = 0;
    while i + window_size * channels <= samples.len() {
        let window = &samples[i..i + window_size * channels];
        let db = calculate_rms_dbfs(window);
        let time = (i as f64 + (window_size * channels / 2) as f64) / (sample_rate * channels) as f64;
        points.push([time, db]);
        dbfs_sum += db;
        i += step_size * channels;
    }

    let duration = points.last().map(|p| p[0]).unwrap_or(0.0);
    let average_dbfs = if points.is_empty() { -120.0 } else { dbfs_sum / points.len() as f64 };

    log_info(logger, &format!("✅ 文件解析完成: {} (Duration: {:.2}s, Points: {})", filename, duration, points.len()));

    Ok(AudioCurve { name: filename, points, duration, average_dbfs, file_path: path })
}

/// 解析 CSV 文件
fn parse_csv(path: PathBuf, logger: &Logger) -> Result<AudioCurve, Box<dyn Error + Send + Sync>> {
    let filename = path.file_name().unwrap().to_string_lossy().to_string();
    log_info(logger, &format!("▶️ 开始解析 CSV 文件: {}", filename));

    let file = File::open(&path)?;
    let mut rdr = csv::Reader::from_reader(file);
    let mut points = Vec::new();
    let mut dbfs_sum = 0.0;
    let mut count = 0;

    for (line_num, result) in rdr.records().enumerate() {
        let record = match result {
            Ok(r) => r,
            Err(e) => {
                log_error(logger, &format!("CSV 读取错误 (Line {}): {}", line_num + 2, e));
                continue;
            }
        };

        if record.len() >= 2 {
            let t = match record[0].parse::<f64>() {
                Ok(v) => v,
                Err(e) => {
                    log_error(logger, &format!("CSV 格式错误 (Time, Line {}): {}", line_num + 2, e));
                    continue;
                }
            };
            let v = match record[1].parse::<f64>() {
                Ok(v) => v,
                Err(e) => {
                    log_error(logger, &format!("CSV 格式错误 (Value, Line {}): {}", line_num + 2, e));
                    continue;
                }
            };
            points.push([t, v]);
            dbfs_sum += v;
            count += 1;
        } else {
            log_error(logger, &format!("CSV 格式错误 (列数不足 2, Line {}): {:?}", line_num + 2, record));
        }
    }
    let duration = points.last().map(|p| p[0]).unwrap_or(0.0);
    let average_dbfs = if count == 0 { -120.0 } else { dbfs_sum / count as f64 };

    log_info(logger, &format!("✅ CSV 解析完成: {} (Duration: {:.2}s, Points: {})", filename, duration, points.len()));

    Ok(AudioCurve { name: filename, points, duration, average_dbfs, file_path: path })
}


fn load_file(path: PathBuf, logger: &Logger) -> Result<AudioCurve, Box<dyn Error + Send + Sync>> {
    if let Some(ext) = path.extension() {
        if ext == "csv" {
            return parse_csv(path, logger);
        }
    }
    parse_wav(path, logger)
}

/// 导出 AudioCurve 数据到 CSV 文件
fn export_to_csv(curve: &AudioCurve, target_lufs: f64, logger: &Logger) -> Result<(), Box<dyn Error + Send + Sync>> {
    let default_name = format!("{}.csv", curve.name.replace(".wav", "").replace(".csv", ""));

    // 允许用户选择保存位置
    let path = FileDialog::new()
        .set_file_name(&default_name)
        .add_filter("CSV File", &["csv"])
        .save_file();

    if let Some(path) = path {
        log_info(logger, &format!("▶️ 导出数据到: {}", path.display()));
        let file = File::create(&path)?;
        let mut wtr = csv::Writer::from_writer(file);

        // 写入表头
        wtr.write_record(&["Time (s)", "Loudness (dBFS)", "Normalized Loudness (dBFS)"])?;

        // 计算偏移量
        let offset_val = target_lufs - curve.average_dbfs;
        log_debug(logger, &format!("应用归一化偏移量: {:.2} dB", offset_val));

        // 写入数据点
        for point in &curve.points {
            let normalized_db = point[1] + offset_val;
            wtr.write_record(&[
                format!("{:.3}", point[0]),      // Time
                format!("{:.2}", point[1]),      // Raw dBFS
                format!("{:.2}", normalized_db), // Normalized dBFS
            ])?;
        }

        wtr.flush()?;
        log_info(logger, &format!("✅ CSV 文件导出成功: {}", path.file_name().unwrap_or_default().to_string_lossy()));
    }
    Ok(())
}


// --- GUI 应用程序结构 ---

struct WavLufsApp {
    mode: AppMode,
    lang: Lang,
    current_lang: Language,

    // 全局日志系统
    logger: Logger,

    // 异步工作池
    worker_pool: WorkerPool,
    ui_tx: mpsc::Sender<WorkerMessage>,
    ui_rx: mpsc::Receiver<WorkerMessage>, // Worker -> UI 消息接收端

    // 命令行相关
    cmd_input: String,

    // 单机模式数据
    single_files: Arc<Mutex<Vec<AudioCurve>>>,
    loading: bool,
    error_msg: Option<String>,
    target_lufs: f32,
    show_help_popup: bool, // 新增：控制帮助悬浮窗

    // 对比模式数据
    compare_a: Option<AudioCurve>,
    compare_b: Option<AudioCurve>,
    compare_result: Option<ComparisonResult>,
    confidence_level: f32,
    // ⭐ 新增: 目标平均差值 (Target Mean Difference)
    target_mean_diff: f32,
    // ⭐ 新增: 曲线同步开关
    compare_plots_linked: bool,

    // ⭐ 新增: 独立播放器
    single_player: AudioPlayer,
    compare_player: AudioPlayer,
    
    // ⭐ 新增: 竖线对齐功能
    single_plot_line_position: f64,
    compare_plot_line_position: f64,
    diff_plot_line_position: f64,
    
    // ⭐ 新增: 音量调节功能（支持dB增量）
    compare_track_db_adjustments: Vec<f32>, // 每个轨道的dB增量
    compare_track_db_input_strings: Vec<String>, // 每个轨道的dB输入框字符串
}

impl WavLufsApp {
    fn new(cc: &eframe::CreationContext<'_>) -> Self {
        let current_lang = Language::Chinese;
        let lang = Lang::load(current_lang);

        // --- 修正: 将字体配置逻辑移到 configure_fonts 并调用 ---
        Self::configure_fonts(&cc.egui_ctx, current_lang);
        // --- 字体配置结束 ---

        // 显式关闭调试功能，避免显示 ID 冲突的调试信息

        // 在 egui 0.27 中，该功能已移至 Context 上的 set_debug_on_hover 方法。


        let logger = Logger::new();
        log_info(&logger, "✅ 应用启动成功。");

        // --- 初始化 MPSC 通道和 WorkerPool ---
        let (ui_tx, ui_rx) = mpsc::channel();
        let worker_pool = WorkerPool::new(ui_tx.clone());

        Self {
            mode: AppMode::Single,
            lang,
            current_lang,
            logger,
            worker_pool,
            ui_tx,
            ui_rx,
            cmd_input: String::new(),
            single_files: Arc::new(Mutex::new(Vec::new())),
            loading: false,
            error_msg: None,
            target_lufs: -23.0,
            show_help_popup: false, // 默认关闭
            compare_a: None,
            compare_b: None,
            compare_result: None,
            confidence_level: 0.95,
            // ⭐ 初始化目标差值为 0.0 (默认为检查绝对匹配)
            target_mean_diff: 0.0,
            // ⭐ 初始化曲线同步开关为 true (默认同步)
            compare_plots_linked: true,
            // ⭐ 初始化独立播放器
            single_player: AudioPlayer::new(),
            compare_player: AudioPlayer::new(),
            // ⭐ 初始化竖线位置
            single_plot_line_position: 0.0,
            compare_plot_line_position: 0.0,
            diff_plot_line_position: 0.0,
            // ⭐ 初始化音量调节
            compare_track_db_adjustments: Vec::new(),
            compare_track_db_input_strings: Vec::new(),
        }
    }

    // --- 新增: 字体配置方法 ---
    /// 配置 egui 字体，根据当前语言加载中文字体
    fn configure_fonts(ctx: &egui::Context, lang: Language) {
        let mut fonts = egui::FontDefinitions::default();

        if lang == Language::Chinese {
            // 1. 加载中文字体 (假设项目中存在 chinese_font.ttf)
            // 必须使用 .into() 兼容 egui::FontData::from_static
            fonts.font_data.insert(
                "chinese_font".to_owned(),
                // ⚠️ 警告：因为我没有 chinese_font.ttf，所以这里假设它已在项目中
                // 实际部署时，请确保 chinese_font.ttf 文件在 main.rs 同目录下。
                // 否则编译会失败。
                egui::FontData::from_static(include_bytes!("chinese_font.ttf")).into(),
            );

            // 2. 设置字体为默认，将中文字体放在首位
            fonts.families.get_mut(&egui::FontFamily::Proportional).unwrap().insert(0, "chinese_font".to_owned());
            fonts.families.get_mut(&egui::FontFamily::Monospace).unwrap().push("chinese_font".to_owned());
        }
        // 如果是 English 模式，egui::FontDefinitions::default() 会确保使用默认字体

        // 3. 应用字体配置
        ctx.set_fonts(fonts);
    }
    // ----------------------------

    // 运行对比逻辑
    fn run_comparison(&mut self) {
        if let (Some(a), Some(b)) = (&self.compare_a, &self.compare_b) {
            // 1. 检查时间长度
            let duration_diff = (a.duration - b.duration).abs();
            if duration_diff > 2.0 { // 容忍 2 秒误差
                let a_fmt = format!("{:.2}", a.duration);
                let b_fmt = format!("{:.2}", b.duration);

                let final_err_msg = self.lang.compare_err_duration_fmt
                    .replacen("{}", &a_fmt, 1)
                    .replacen("{}", &b_fmt, 1);

                log_error(&self.logger, &format!("⚠️ 对比失败: {}", final_err_msg));
                self.error_msg = Some(final_err_msg);
                self.compare_result = None;
                return;
            }

            // 2. 计算差值和收集原始数据点
            let len = std::cmp::min(a.points.len(), b.points.len());
            log_debug(&self.logger, &format!("对比点数: {}", len));
            
            // ⭐ 修复关键点：获取当前 UI 设置的 dB 增量
            // 如果数组为空或索引越界，默认为 0.0
            let a_adj = *self.compare_track_db_adjustments.get(0).unwrap_or(&0.0) as f64;
            let b_adj = *self.compare_track_db_adjustments.get(1).unwrap_or(&0.0) as f64;

            let mut diff_vals = Vec::new();
            let mut diff_points = Vec::new();
            let mut a_vals = Vec::new();
            let mut b_vals = Vec::new();
            let mut filtered_count = 0;

            for i in 0..len {
                // ⭐ 修复关键点：在计算前应用增量
                let a_val = a.points[i][1] + a_adj;
                let b_val = b.points[i][1] + b_adj;
                
                // 过滤脏数据：当任一轨道的(应用增量后的)响度值低于 -60 dB 时跳过该点
                if a_val < -60.0 || b_val < -60.0 {
                    filtered_count += 1;
                    continue;
                }
                
                let diff = a_val - b_val;
                diff_vals.push(diff);
                diff_points.push([a.points[i][0], diff]);
                a_vals.push(a_val);
                b_vals.push(b_val);
            }
            
            if filtered_count > 0 {
                // 降低日志级别以免刷屏，或者保持 debug
                // log_debug(&self.logger, &format!("过滤了 {} 个脏数据点", filtered_count));
            }
            
            // 如果过滤后没有数据点，返回错误
            if diff_vals.is_empty() {
                log_error(&self.logger, "⚠️ 对比失败: 过滤后没有有效的数据点");
                self.error_msg = Some("过滤后没有有效的数据点".to_string());
                self.compare_result = None;
                return;
            }

            // 3. 统计
            let filtered_len = diff_vals.len();
            let mean = diff_vals.iter().sum::<f64>() / filtered_len as f64;
            let variance: f64 = diff_vals.iter().map(|v| (v - mean).powi(2)).sum::<f64>() / (filtered_len as f64 - 1.0).max(1.0);
            let std_dev = variance.sqrt();
            let max_diff = diff_vals.iter().fold(f64::NEG_INFINITY, |a, &b| a.max(b));
            let min_diff = diff_vals.iter().fold(f64::INFINITY, |a, &b| a.min(b));

            // 4. 新增统计计算
            let correlation_coefficient = calculate_correlation(&a_vals, &b_vals, filtered_len);

            // T 检验逻辑
            let target_c = self.target_mean_diff as f64;
            // T 统计量现在检验 (实际平均差值 - 目标平均差值) 是否显著不为 0
            let t_statistic = calculate_t_statistic(mean - target_c, std_dev, filtered_len);

            log_info(&self.logger, &format!("✅ 对比完成 (Adj: A{:.1}/B{:.1}). Mean Diff: {:.2} dB", a_adj, b_adj, mean));

            self.compare_result = Some(ComparisonResult {
                mean_diff: mean,
                std_dev,
                max_diff,
                min_diff,
                correlation_coefficient,
                t_statistic,
                diff_points,
            });
            self.error_msg = None;
        } else {
            // log_error(&self.logger, "⚠️ 对比失败: 缺少 Track A 或 Track B。");
        }
    }


    // 允许切换语言，同时更新 UI
    // fn switch_language(&mut self, new_lang: Language, ctx: &egui::Context) {
    //     if self.current_lang != new_lang {
    //         log_info(&self.logger, &format!("切换语言到: {:?}", new_lang));
    //         self.current_lang = new_lang;
    //         self.lang = Lang::load(new_lang);

    //         //  修正: 切换语言时重新配置字体
    //         Self::configure_fonts(ctx, new_lang);

    //         ctx.request_repaint();
    //     }
    // }
}

impl eframe::App for WavLufsApp {
    fn update(&mut self, ctx: &egui::Context, _frame: &mut eframe::Frame) {
        // 全局样式微调
        ctx.set_pixels_per_point(1.2);
        
        // 处理键盘输入 - 空格键控制播放
        if ctx.input(|i| i.key_pressed(egui::Key::Space)) {
            match self.mode {
                AppMode::Single => {
                    if self.single_player.is_playing {
                        self.single_player.pause();
                        log_info(&self.logger, "播放器暂停（空格键）");
                    } else {
                        if let Err(e) = self.single_player.play() {
                            log_error(&self.logger, &format!("播放失败（空格键）: {}", e));
                        } else {
                            log_info(&self.logger, "播放器开始播放（空格键）");
                        }
                    }
                }
                AppMode::Compare => {
                    if self.compare_player.is_playing {
                        self.compare_player.pause();
                        log_info(&self.logger, "播放器暂停（空格键）");
                    } else {
                        if let Err(e) = self.compare_player.play() {
                            log_error(&self.logger, &format!("播放失败（空格键）: {}", e));
                        } else {
                            log_info(&self.logger, "播放器开始播放（空格键）");
                        }
                    }
                }
                AppMode::Console => {
                    // 在控制台模式下，空格键不控制播放
                }
            }
            // 防止空格键重复触发
            ctx.send_viewport_cmd(egui::ViewportCommand::Focus);
        }

        // --- 异步消息处理 (非阻塞循环) ---
        while let Ok(msg) = self.ui_rx.try_recv() {
            match msg {
                WorkerMessage::Log(entry) => {
                    if let Ok(mut entries) = self.logger.entries.lock() {
                        entries.push(entry);
                    }
                    ctx.request_repaint();
                }
                WorkerMessage::UpdateTaskState(id, state) => {
                    if let Ok(mut tasks) = self.worker_pool.tasks.lock() {
                        if let Some(task) = tasks.iter_mut().find(|t| t.id == id) {
                            task.state = state.clone();
                            self.loading = tasks.iter().any(|t| matches!(t.state, TaskState::Running(_)) || t.state == TaskState::Waiting);

                            if let TaskState::Error(e) = state {
                                self.error_msg = Some(format!("Task {} Error: {}", id, e));
                            }
                        }
                    }
                    ctx.request_repaint();
                }
                WorkerMessage::NewCurve(curve, slot_opt) => { // 修正: 接收 slot_opt
                    if let Some(slot) = slot_opt {
                        // 对比模式结果
                        if slot == 'A' {
                            self.compare_a = Some(curve.clone());
                            let _ = self.compare_player.add_track(curve);
                        } else if slot == 'B' {
                            self.compare_b = Some(curve.clone());
                            let _ = self.compare_player.add_track(curve);
                        }

                        // 关键: 尝试运行对比 (必须在 UI 线程上)
                        if self.compare_a.is_some() && self.compare_b.is_some() {
                            self.run_comparison();
                        }
                    } else {
                        // 单机模式结果
                        if let AppMode::Single = self.mode {
                            if let Ok(mut files) = self.single_files.lock() {
                                let _ = self.single_player.add_track(curve.clone());
                                files.push(curve);
                            }
                        }
                    }
                    ctx.request_repaint();
                }
            }
        }

        // --- 更新播放器状态 ---
        self.single_player.update();
        self.compare_player.update();
        
        // 更新竖线位置以对齐当前播放时间
        if self.single_player.is_playing {
            self.single_plot_line_position = self.single_player.current_position;
            ctx.request_repaint();
        }
        if self.compare_player.is_playing {
            self.compare_plot_line_position = self.compare_player.current_position;
            self.diff_plot_line_position = self.compare_player.current_position;
            ctx.request_repaint();
        }

        // --- 顶部导航栏 (I18N & 语言选择) ---
        egui::TopBottomPanel::top("top_panel").show(ctx, |ui| {
            ui.horizontal(|ui| {
                ui.spacing_mut().item_spacing.x = 4.0;
                // 模式选择
                ui.selectable_value(&mut self.mode, AppMode::Single, self.lang.mode_single);
                ui.selectable_value(&mut self.mode, AppMode::Compare, self.lang.mode_compare);
                ui.selectable_value(&mut self.mode, AppMode::Console, self.lang.mode_console);

                ui.separator();

                // 语言选择
                // 修正：使用 I18N 字段替代硬编码的 "语言:"
                ui.label(self.lang.nav_lang_label);
                let old_lang = self.current_lang; // 记录旧语言

                // 中文选项 - 修正：使用 I18N 字段替代硬编码的 "中文"
                ui.selectable_value(&mut self.current_lang, Language::Chinese, self.lang.nav_zh_label);

                // English 选项 - 修正：使用 I18N 字段替代硬编码的 "English"
                ui.selectable_value(&mut self.current_lang, Language::English, self.lang.nav_en_label);

                // 修正语言切换逻辑：在 selectable_value 之外检查并重新加载
                if self.current_lang != old_lang {
                    log_info(&self.logger, &format!("切换语言到: {:?}", self.current_lang));

                    // 核心切换逻辑：重新加载语言数据和字体
                    self.lang = Lang::load(self.current_lang);
                    Self::configure_fonts(ctx, self.current_lang);

                    // 由于 selectable_value 已经点击了，我们不需要 if clicked() 包装
                    ui.ctx().request_repaint();
                }

                ui.separator();

                // --- 新增：帮助按钮 --- 修正：使用 I18N 字段替代硬编码的 "❓ 帮助"
                if ui.button(self.lang.nav_help_btn).clicked() {
                    self.show_help_popup = true;
                }
            });
        });

        // --- 底部状态栏 (I18N) ---
        egui::TopBottomPanel::bottom("bottom_panel").show(ctx, |ui| {
            ui.horizontal(|ui| {
                if self.loading {
                    ui.spinner();
                    ui.label(self.lang.status_loading); // I18N
                    ctx.request_repaint();
                } else if let Some(err) = &self.error_msg {
                    ui.colored_label(egui::Color32::RED, err);
                } else {
                    ui.label(self.lang.status_ready); // I18N
                }
                
                // 添加播放时间显示
                ui.add_space(20.0);
                let (player_name, position, duration, is_playing) = match self.mode {
                    AppMode::Single => ("单机", self.single_player.current_position, self.single_player.duration, self.single_player.is_playing),
                    AppMode::Compare => ("AB对比", self.compare_player.current_position, self.compare_player.duration, self.compare_player.is_playing),
                    AppMode::Console => ("控制台", 0.0, 0.0, false),
                };
                
                if is_playing || position > 0.0 {
                    let status = if is_playing { "播放中" } else { "暂停" };
                    ui.label(format!("{} {}: {:.1}s / {:.1}s", player_name, status, position, duration));
                } else {
                    ui.label(format!("{}: 0.0s / 0.0s", player_name));
                }
            });
        });

        // 中央内容区
        egui::CentralPanel::default().show(ctx, |ui| {
            match self.mode {
                AppMode::Single => self.ui_single_mode(ui, ctx),
                AppMode::Compare => self.ui_compare_mode(ui, ctx),
                AppMode::Console => self.ui_console_mode(ui),
            }
        });

        // --- 新增：帮助悬浮窗口 ---
        self.ui_help_popup(ctx);
    }
}

impl WavLufsApp {

    // --- 新增：帮助悬浮窗口的实现 ---
    fn ui_help_popup(&mut self, ctx: &egui::Context) {
        if self.show_help_popup {
            let lang = &self.lang; // 获取当前语言文本
            // 这里使用一个唯一的 ID 源，以防与任何其他 Window 冲突
            egui::Window::new(lang.help_title)
                .id(egui::Id::new("help_window"))
                .open(&mut self.show_help_popup)
                .resizable(true)
                .default_size([400.0, 400.0])
                .show(ctx, |ui| {
                    ui.label(lang.help_desc);
                    ui.separator();

                    ui.heading(lang.help_monitor_title);
                    // 修正：使用 I18N 字段替代硬编码的中文描述
                    ui.label(lang.help_monitor_desc);
                    ui.separator();

                    ui.heading(lang.help_console_title);
                    ui.vertical(|ui| {
                        // 使用 help_cmd_tasks 来描述 tasks/list 命令
                        ui.label(format!("**`tasks`** 或 **`list`**: {}", lang.help_cmd_list));
                        ui.label(format!("**`kill <ID>`**: {}", lang.help_cmd_kill));
                        ui.label(format!("**`clear`**: {}", lang.help_cmd_clear));
                        ui.label(format!("**`quit`** 或 **`exit`**: {}", lang.help_cmd_quit));
                    });
                    ui.separator();

                    // --- 新增：图表交互说明 ---
                    ui.heading(lang.help_plot_interaction_title);
                    ui.vertical(|ui| {
                        ui.label(lang.help_plot_drag_desc);
                        ui.label(lang.help_plot_seek_desc);
                    });
                    ui.separator();

                    // --- 新增：播放器快捷键说明 ---
                    ui.heading(lang.help_player_shortcuts_title);
                    ui.vertical(|ui| {
                        ui.label(lang.help_space_key_desc);
                    });
                });
        }
    }
    // ---------------------------------

    fn ui_single_mode(&mut self, ui: &mut egui::Ui, _ctx: &egui::Context) {
        // 文件选择区域
        ui.horizontal(|ui| {
            if ui.button(self.lang.single_open_btn).clicked() { // I18N
                log_info(&self.logger, "用户点击: 打开文件对话框");
                let files = FileDialog::new()
                    .add_filter("Audio/Data", &["wav", "csv"])
                    .pick_files();

                if let Some(paths) = files {
                    log_info(&self.logger, &format!("选中文件数: {}", paths.len()));
                    self.loading = true;
                    self.error_msg = None;

                    let logger_ref = &self.logger;
                    let ui_result_tx_base = self.ui_tx.clone();

                    for path in paths {
                        let filename = path.file_name().unwrap().to_string_lossy().to_string();
                        let task_ui_tx = ui_result_tx_base.clone();

                        self.worker_pool.spawn_task(
                            filename.clone(),
                            move |task_id, ui_tx_clone, logger_entries| {
                                let thread_logger = Logger { entries: logger_entries };

                                match load_file(path, &thread_logger) {
                                    Ok(curve) => {
                                        ui_tx_clone.send(WorkerMessage::NewCurve(curve, None)).unwrap_or_default();
                                    }
                                    Err(e) => {
                                        let err_msg = format!("文件加载失败 ({}): {}", filename, e);
                                        log_error(&thread_logger, &err_msg);
                                        ui_tx_clone.send(WorkerMessage::UpdateTaskState(task_id, TaskState::Error(err_msg))).unwrap_or_default();
                                    }
                                }
                            },
                            task_ui_tx,
                            logger_ref
                        );
                    }
                }
            }

            if ui.button(self.lang.single_clear_btn).clicked() { // I18N
                self.single_files.lock().unwrap().clear();
                self.single_player.clear_tracks();
                log_info(&self.logger, "文件列表已清空。");
            }

            let curves = self.single_files.lock().unwrap();
            if !curves.is_empty() {
                if ui.button(self.lang.export_csv_btn).clicked() { // I18N
                    if let Some(curve) = curves.first() {
                        match export_to_csv(curve, self.target_lufs as f64, &self.logger) {
                            Ok(_) => self.error_msg = Some(format!("✅ {} exported successfully!", curve.name)),
                            Err(e) => {
                                let err_msg = format!("❌ Export failed: {}", e);
                                log_error(&self.logger, &err_msg);
                                self.error_msg = Some(err_msg);
                            }
                        }
                    }
                }
            }
            drop(curves);
        });

        ui.separator();

        // 归一化设置
        ui.horizontal(|ui| {
            ui.label(self.lang.normalize_label); // I18N
            ui.add(egui::DragValue::new(&mut self.target_lufs)
                .speed(0.1)
                .range(-60.0..=0.0)
                .suffix(" dBFS")
            );
            if ui.button(self.lang.normalize_apply).clicked() {
                log_info(&self.logger, &format!("归一化目标设定为: {:.1} dBFS", self.target_lufs));
                self.error_msg = Some(format!("已应用归一化目标: {:.1} dBFS", self.target_lufs));
            }
        });

        ui.separator();

        // 轻量化文件列表（集成播放控制）
        ui.heading("📋 文件列表");
        ui.push_id("single_file_list", |ui| {
            egui::ScrollArea::vertical().max_height(150.0).show(ui, |ui| {
                let curves = self.single_files.lock().unwrap();
                if curves.is_empty() {
                    ui.label(self.lang.player_no_files);
                } else {
                    for (index, curve) in curves.iter().enumerate() {
                        ui.push_id(format!("track_{}", index), |ui| {
                            ui.horizontal(|ui| {
                                // 文件名
                                ui.label(&curve.name);

                                ui.add_space(10.0);

                                // 播放/暂停按钮
                                if self.single_player.is_playing {
                                    if ui.add_sized([60.0, 20.0], egui::Button::new(self.lang.player_pause)).clicked() {
                                        self.single_player.pause();
                                        log_info(&self.logger, "播放器暂停");
                                    }
                                } else {
                                    if ui.add_sized([60.0, 20.0], egui::Button::new(self.lang.player_play)).clicked() {
                                        if let Err(e) = self.single_player.play() {
                                            log_error(&self.logger, &format!("播放失败: {}", e));
                                        } else {
                                            log_info(&self.logger, "播放器开始播放");
                                        }
                                    }
                                }

                                // 停止按钮
                                if ui.add_sized([60.0, 20.0], egui::Button::new(self.lang.player_stop)).clicked() {
                                    self.single_player.stop();
                                    log_info(&self.logger, "播放器停止");
                                }

                                ui.add_space(10.0);

                                // 播放进度条
                                let duration = self.single_player.duration;
                                let current_position = self.single_player.current_position;
                                if duration > 0.0 {
                                    let mut position = current_position;
                                    let position_text = format!("{:.1}s / {:.1}s", position, duration);
                                    ui.add_sized([200.0, 20.0], egui::Slider::new(&mut position, 0.0..=duration)
                                        .show_value(false)
                                        .text(position_text));

                                    if position != current_position {
                                        if let Err(e) = self.single_player.seek_to(position) {
                                            log_error(&self.logger, &format!("跳转失败: {}", e));
                                        }
                                    }
                                }
                            });
                        });
                    }
                }
            });
        });

        ui.separator();

        // 绘图区域 - 始终显示
        ui.push_id("single_plot_area", |ui| {
            // 1. 准备变量存储跳转请求
            let mut seek_request: Option<f64> = None;
            let duration = self.single_player.duration;

            // ⭐ 在闭包外部预先读取输入状态，避免借用冲突
            let pointer_down = ui.input(|i| i.pointer.primary_down());

            // 2. 配置 Plot
            let plot = Plot::new("single_plot")
                .legend(Legend::default())
                .y_axis_label(self.lang.single_y_label) // I18N
                .x_axis_label(self.lang.single_x_label) // I18N
                // ⭐ 按住 Shift 键时才允许拖拽进度，否则是平移视图
                .allow_drag(!ui.input(|i| i.modifiers.shift));

            plot.show(ui, |plot_ui| {
                // --- 绘图逻辑 (保持原样) ---
                // 添加播放位置指示线
                if self.single_player.is_playing || self.single_player.current_position > 0.0 {
                    plot_ui.vline(egui_plot::VLine::new("playback_position", self.single_player.current_position)
                        .color(egui::Color32::from_rgb(255, 165, 0)));
                }

                let curves = self.single_files.lock().unwrap();
                let target = self.target_lufs as f64;
                for curve in curves.iter() {
                    // 计算归一化偏移量：目标 - 平均 dBFS
                    let offset = target - curve.average_dbfs;

                    // 应用偏移量到曲线数据
                    let shifted_points: PlotPoints = curve.points.iter()
                        .map(|p| [p[0], p[1] + offset])
                        .collect();

                    let name = format!("{} (Avg: {:.2} dBFS)", curve.name, curve.average_dbfs);

                    plot_ui.line(Line::new(name, shifted_points));
                }

                // --- ⭐ 新增：交互逻辑 ---
                // 检测鼠标是否在图表区域内且按下了左键
                if plot_ui.response().hovered() && pointer_down {
                    // 获取鼠标指针在图表中的坐标
                    if let Some(pos) = plot_ui.pointer_coordinate() {
                        seek_request = Some(pos.x);
                    }
                }
            });

            // 3. 处理跳转请求 (在闭包外部执行，避免所有权冲突)
            if let Some(seek_time) = seek_request {
                // 限制范围在 0 到 音频时长 之间
                let target_time = seek_time.clamp(0.0, duration);
                // 执行跳转 (忽略可能的错误)
                let _ = self.single_player.seek_to(target_time);
                
                // 如果想拖拽时暂停，松开时播放，可以在这里加逻辑。
                // 目前这样写是"刷盘(Scrubbing)"效果。
            }
        });
    }

    #[allow(dead_code)]
    fn ui_player_controls(&mut self, ui: &mut egui::Ui, player: &mut AudioPlayer, _show_tracks: bool) {
        ui.group(|ui| {
            ui.horizontal(|ui| {
                // 播放/暂停/停止按钮
                let is_playing = player.is_playing;
                if is_playing {
                    if ui.button(self.lang.player_pause).clicked() {
                        player.pause();
                        log_info(&self.logger, "播放器暂停");
                    }
                } else {
                    if ui.button(self.lang.player_play).clicked() {
                        if let Err(e) = player.play() {
                            log_error(&self.logger, &format!("播放失败: {}", e));
                        } else {
                            log_info(&self.logger, "播放器开始播放");
                        }
                    }
                }

                if ui.button(self.lang.player_stop).clicked() {
                    player.stop();
                    log_info(&self.logger, "播放器停止");
                }

                ui.separator();

                // 播放进度条
                let duration = player.duration;
                let current_position = player.current_position;
                if duration > 0.0 {
                    let mut position = current_position;
                    let position_text = format!("{:.1}s / {:.1}s", position, duration);
                    ui.add_sized([300.0, 20.0], egui::Slider::new(&mut position, 0.0..=duration)
                        .show_value(false)
                        .text(position_text));

                    if position != current_position {
                        if let Err(e) = player.seek_to(position) {
                            log_error(&self.logger, &format!("跳转失败: {}", e));
                        }
                    }
                } else {
                    ui.label(format!("{}: 0.0s / 0.0s", self.lang.player_position));
                }
            });
        });
    }

    fn ui_compare_mode(&mut self, ui: &mut egui::Ui, ctx: &egui::Context) {
        // 文件选择区 - 可折叠（包含标题）
        egui::CollapsingHeader::new("📂 A/B 文件选择")
            .default_open(true)
            .show(ui, |ui| {
                // 修复 ID 冲突：使用 ui.push_id 隔离文件选择区的列布局
                ui.push_id("compare_selection", |ui| {
                    ui.columns(2, |columns| {
                // Slot A
                columns[0].vertical(|ui| {
                    ui.label(self.lang.compare_track_a_label); // I18N
                    
                    // 检查Track A是否正在加载
                    let is_loading_a = if let Ok(tasks) = self.worker_pool.tasks.lock() {
                        tasks.iter().any(|t| matches!(t.state, TaskState::Running(_)) && t.name.contains("Track A"))
                    } else {
                        false
                    };
                    
                    if is_loading_a {
                        ui.horizontal(|ui| {
                            ui.spinner();
                            ui.label("加载中...");
                        });
                    }
                    
                    if ui.button(self.compare_a.as_ref().map(|c| c.name.as_str()).unwrap_or(self.lang.compare_select_a)).clicked() { // I18N
                        log_info(&self.logger, "选择 Track A");
                        if let Some(path) = FileDialog::new().add_filter("Audio", &["wav", "csv"]).pick_file() {
                            let file_slot = 'A'; // 定义插槽
                            let filename = path.file_name().unwrap().to_string_lossy().to_string();
                            let task_name = format!("Track {} Load: {}", file_slot, filename);
                            let logger_ref = &self.logger;
                            let ui_result_tx_base = self.ui_tx.clone();

                            self.loading = true; // 增加 loading 状态
                            self.error_msg = None;

                            // 启动后台加载任务
                            self.worker_pool.spawn_task(
                                task_name,
                                move |task_id, ui_tx_clone, logger_entries| {
                                    let thread_logger = Logger { entries: logger_entries };
                                    match load_file(path, &thread_logger) {
                                        Ok(curve) => {
                                            // 发送结果和插槽信息
                                            ui_tx_clone.send(WorkerMessage::NewCurve(curve, Some(file_slot))).unwrap_or_default();
                                            ui_tx_clone.send(WorkerMessage::UpdateTaskState(task_id, TaskState::Completed)).unwrap_or_default();
                                        }
                                        Err(e) => {
                                            let err_msg = format!("文件加载失败 ({}): {}", filename, e);
                                            ui_tx_clone.send(WorkerMessage::UpdateTaskState(task_id, TaskState::Error(err_msg))).unwrap_or_default();
                                        }
                                    }
                                },
                                ui_result_tx_base,
                                logger_ref
                            );
                        }
                    }

                    // --- 新增：Track A的Mute/Solo/音量控制 ---
                    // 确保dB调整数组大小与轨道数量一致
                    while self.compare_track_db_adjustments.len() < 2 {
                        self.compare_track_db_adjustments.push(0.0);
                        self.compare_track_db_input_strings.push("0.0".to_string());
                    }
                    
                    let track_info: Option<(bool, bool, f32, String)> = self.compare_player.tracks.get(0).map(|t| (t.is_muted, t.is_soloed, t.volume, t.curve.name.clone()));
                    if let Some((is_muted, is_soloed, volume, name)) = track_info {
                        ui.horizontal(|ui| {
                            // Mute按钮
                            let mute_color = if is_muted {
                                egui::Color32::RED
                            } else {
                                egui::Color32::GRAY
                            };
                            if ui.add(egui::Button::new(self.lang.player_mute)
                                .fill(mute_color)
                                .small()).clicked() {
                                self.compare_player.set_track_mute(0, !is_muted);
                                log_info(&self.logger, &format!("Track A Mute: {}", !is_muted));
                            }

                            // Solo按钮
                            let solo_color = if is_soloed {
                                egui::Color32::YELLOW
                            } else {
                                egui::Color32::GRAY
                            };
                            if ui.add(egui::Button::new(self.lang.player_solo)
                                .fill(solo_color)
                                .small()).clicked() {
                                self.compare_player.set_track_solo(0, !is_soloed);
                                log_info(&self.logger, &format!("Track A Solo: {}", !is_soloed));
                            }

                            // 文件名
                            ui.label(&name);

                            // 音量滑块
                            ui.add_space(10.0);
                            ui.label(self.lang.player_volume);
                            let mut vol = volume;
                            ui.add_sized([100.0, 20.0], egui::Slider::new(&mut vol, 0.0..=1.0)
                                .show_value(false));
                            if vol != volume {
                                self.compare_player.set_track_volume(0, vol);
                            }
                            
                            // dB增量输入
                            ui.add_space(10.0);
                            ui.label("dB增量:");
                            let db_response = ui.add_sized([80.0, 20.0], egui::TextEdit::singleline(&mut self.compare_track_db_input_strings[0]));
                            
                            // 处理dB增量输入
                            if db_response.lost_focus() {
                                if let Ok(parsed_db) = self.compare_track_db_input_strings[0].parse::<f32>() {
                                    self.compare_track_db_adjustments[0] = parsed_db;
                                    self.compare_player.set_track_volume_db(0, parsed_db);
                                    log_info(&self.logger, &format!("Track A dB增量: {:.1}", parsed_db));
                                    // dB增量变化时重新运行对比
                                    if self.compare_a.is_some() && self.compare_b.is_some() {
                                        self.run_comparison();
                                        ctx.request_repaint();
                                    }
                                }
                            }
                        });
                    }
                });
                // Slot B
                columns[1].vertical(|ui| {
                    ui.label(self.lang.compare_track_b_label); // I18N
                    
                    // 检查Track B是否正在加载
                    let is_loading_b = if let Ok(tasks) = self.worker_pool.tasks.lock() {
                        tasks.iter().any(|t| matches!(t.state, TaskState::Running(_)) && t.name.contains("Track B"))
                    } else {
                        false
                    };
                    
                    if is_loading_b {
                        ui.horizontal(|ui| {
                            ui.spinner();
                            ui.label("加载中...");
                        });
                    }
                    
                    if ui.button(self.compare_b.as_ref().map(|c| c.name.as_str()).unwrap_or(self.lang.compare_select_b)).clicked() { // I18N
                        log_info(&self.logger, "选择 Track B");
                        if let Some(path) = FileDialog::new().add_filter("Audio", &["wav", "csv"]).pick_file() {
                            let file_slot = 'B'; // 定义插槽
                            let filename = path.file_name().unwrap().to_string_lossy().to_string();
                            let task_name = format!("Track {} Load: {}", file_slot, filename);
                            let logger_ref = &self.logger;
                            let ui_result_tx_base = self.ui_tx.clone();

                            self.loading = true; // 增加 loading 状态
                            self.error_msg = None;

                            // 启动后台加载任务
                            self.worker_pool.spawn_task(
                                task_name,
                                move |task_id, ui_tx_clone, logger_entries| {
                                    let thread_logger = Logger { entries: logger_entries };
                                    match load_file(path, &thread_logger) {
                                        Ok(curve) => {
                                            // 发送结果和插槽信息
                                            ui_tx_clone.send(WorkerMessage::NewCurve(curve, Some(file_slot))).unwrap_or_default();
                                            ui_tx_clone.send(WorkerMessage::UpdateTaskState(task_id, TaskState::Completed)).unwrap_or_default();
                                        }
                                        Err(e) => {
                                            let err_msg = format!("文件加载失败 ({}): {}", filename, e);
                                            ui_tx_clone.send(WorkerMessage::UpdateTaskState(task_id, TaskState::Error(err_msg))).unwrap_or_default();
                                        }
                                    }
                                },
                                ui_result_tx_base,
                                logger_ref
                            );
                        }
                    }

                    // --- 新增：Track B的Mute/Solo/音量控制 ---
                    let track_info: Option<(bool, bool, f32, String)> = self.compare_player.tracks.get(1).map(|t| (t.is_muted, t.is_soloed, t.volume, t.curve.name.clone()));
                    if let Some((is_muted, is_soloed, volume, name)) = track_info {
                        ui.horizontal(|ui| {
                            // Mute按钮
                            let mute_color = if is_muted {
                                egui::Color32::RED
                            } else {
                                egui::Color32::GRAY
                            };
                            if ui.add(egui::Button::new(self.lang.player_mute)
                                .fill(mute_color)
                                .small()).clicked() {
                                self.compare_player.set_track_mute(1, !is_muted);
                                log_info(&self.logger, &format!("Track B Mute: {}", !is_muted));
                            }

                            // Solo按钮
                            let solo_color = if is_soloed {
                                egui::Color32::YELLOW
                            } else {
                                egui::Color32::GRAY
                            };
                            if ui.add(egui::Button::new(self.lang.player_solo)
                                .fill(solo_color)
                                .small()).clicked() {
                                self.compare_player.set_track_solo(1, !is_soloed);
                                log_info(&self.logger, &format!("Track B Solo: {}", !is_soloed));
                            }

                            // 文件名
                            ui.label(&name);

                            // 音量滑块
                            ui.add_space(10.0);
                            ui.label(self.lang.player_volume);
                            let mut vol = volume;
                            ui.add_sized([100.0, 20.0], egui::Slider::new(&mut vol, 0.0..=1.0)
                                .show_value(false));
                            if vol != volume {
                                self.compare_player.set_track_volume(1, vol);
                            }
                            
                            // dB增量输入
                            ui.add_space(10.0);
                            ui.label("dB增量:");
                            let db_response = ui.add_sized([80.0, 20.0], egui::TextEdit::singleline(&mut self.compare_track_db_input_strings[1]));
                            
                            // 处理dB增量输入
                            if db_response.lost_focus() {
                                if let Ok(parsed_db) = self.compare_track_db_input_strings[1].parse::<f32>() {
                                    self.compare_track_db_adjustments[1] = parsed_db;
                                    self.compare_player.set_track_volume_db(1, parsed_db);
                                    log_info(&self.logger, &format!("Track B dB增量: {:.1}", parsed_db));
                                    // dB增量变化时重新运行对比
                                    if self.compare_a.is_some() && self.compare_b.is_some() {
                                        self.run_comparison();
                                        ctx.request_repaint();
                                    }
                                }
                            }
                        });
                    }
                });
            });
                });
            }); // 关闭 CollapsingHeader

        // --- 新增：播放器控件 ---
        ui.group(|ui| {
            ui.horizontal(|ui| {
                // 播放/暂停/停止按钮
                let is_playing = self.compare_player.is_playing;
                if is_playing {
                    if ui.button(self.lang.player_pause).clicked() {
                        self.compare_player.pause();
                        log_info(&self.logger, "播放器暂停");
                    }
                } else {
                    if ui.button(self.lang.player_play).clicked() {
                        if let Err(e) = self.compare_player.play() {
                            log_error(&self.logger, &format!("播放失败: {}", e));
                        } else {
                            log_info(&self.logger, "播放器开始播放");
                        }
                    }
                }

                if ui.button(self.lang.player_stop).clicked() {
                    self.compare_player.stop();
                    log_info(&self.logger, "播放器停止");
                }

                ui.separator();

                // 播放进度条
                let duration = self.compare_player.duration;
                let current_position = self.compare_player.current_position;
                if duration > 0.0 {
                    let mut position = current_position;
                    let position_text = format!("{:.1}s / {:.1}s", position, duration);
                    ui.add_sized([300.0, 20.0], egui::Slider::new(&mut position, 0.0..=duration)
                        .show_value(false)
                        .text(position_text));

                    if position != current_position {
                        if let Err(e) = self.compare_player.seek_to(position) {
                            log_error(&self.logger, &format!("跳转失败: {}", e));
                        }
                    }
                } else {
                    ui.label(format!("{}: 0.0s / 0.0s", self.lang.player_position));
                }
            });
        });

        // ⭐ FIX E0500: 先克隆结果，让后续的 'res' 引用不再阻塞对 'self' 的可变访问。
        let comparison_result_clone = self.compare_result.clone();

        // ⭐ 一致性检验分析 - 可折叠区域（只在有结果时显示）
        if let Some(res) = &comparison_result_clone {
            egui::CollapsingHeader::new("📊 一致性检验分析")
                .default_open(false)
                .show(ui, |ui| {
                    // --- 置信度选择 + T检验中心值 (同一行) ---
                    ui.horizontal(|ui| {
                        ui.label(self.lang.compare_conf_label); // I18N

                        // 检查是否有按钮被点击，并存储标志
                        let mut clicked = false;
                        if ui.selectable_value(&mut self.confidence_level, 0.90, "90%").clicked() { clicked = true; }
                        if ui.selectable_value(&mut self.confidence_level, 0.95, "95%").clicked() { clicked = true; }
                        if ui.selectable_value(&mut self.confidence_level, 0.99, "99%").clicked() { clicked = true; }

                        // 只有在点击后才调用 &mut self 的方法
                        if clicked {
                            log_debug(&self.logger, &format!("置信度设置为 {:.0}%", self.confidence_level * 100.0));
                            self.run_comparison();
                            ctx.request_repaint();
                        }

                        ui.separator();

                        // T检验中心值放在右侧
                        ui.label("T检验中心:");
                        let response = ui.add(egui::DragValue::new(&mut self.target_mean_diff)
                            .speed(0.1)
                            .range(-20.0..=20.0)
                            .suffix(" dB")
                        );
                        if response.changed() || (response.lost_focus() && ui.input(|i| i.key_pressed(egui::Key::Enter))) {
                            self.run_comparison();
                            ctx.request_repaint();
                        }
                    });
                    ui.separator();
                    // ------------------------------------

                    // 统计数据面板
                    ui.horizontal(|ui| {
                // ⭐ 修复 ID 冲突：使用 ui.push_id 隔离 group
                ui.push_id("compare_stats", |ui| {
                    ui.group(|ui| {
                        ui.vertical(|ui| {
                            ui.label(egui::RichText::new(self.lang.compare_report_title).strong()); // I18N

                            // 平均差异
                            let avg_diff_fmt = format!("{:.2}", res.mean_diff);
                            ui.label(self.lang.compare_avg_diff_fmt.replacen("{}", &avg_diff_fmt, 1)); // I18N

                            // 标准差
                            let std_dev_fmt = format!("{:.4}", res.std_dev);
                            ui.label(self.lang.compare_std_dev_fmt.replacen("{}", &std_dev_fmt, 1));    // I18N

                            // 动态相关系数 (r) - 衡量曲线形状相似度
                            let corr_fmt = format!("{:.4}", res.correlation_coefficient);
                            ui.label(self.lang.compare_correlation_fmt.replacen("{}", &corr_fmt, 1)); // I18N

                            // 状态结果 (基于标准差)
                            if res.std_dev < 1.0 {
                                ui.colored_label(egui::Color32::GREEN, self.lang.compare_high_match); // I18N
                            } else if res.std_dev < 3.0 {
                                ui.colored_label(egui::Color32::YELLOW, self.lang.compare_mid_diff); // I18N
                            } else {
                                ui.colored_label(egui::Color32::RED, self.lang.compare_huge_diff); // I18N
                            }
                        });
                    });
                });

                ui.vertical(|ui| {
                    // 最大差值
                    let max_diff_fmt = format!("{:.2}", res.max_diff);
                    ui.label(self.lang.compare_max_diff_fmt.replacen("{}", &max_diff_fmt, 1)); // I18N

                    // 最小差值
                    let min_diff_fmt = format!("{:.2}", res.min_diff);
                    ui.label(self.lang.compare_min_diff_fmt.replacen("{}", &min_diff_fmt, 1)); // I18N

                    // ⭐ 新增: 报告 T 检验目标
                    ui.label(format!("T 检验目标: {:.2} dB", self.target_mean_diff));

                    // 均值差值 T-统计量
                    let t_stat_fmt = format!("{:.2}", res.t_statistic);
                    ui.label(self.lang.compare_t_stat_fmt.replacen("{}", &t_stat_fmt, 1)); // I18N

                    // --- 假设检验结果 (根据置信度动态判断) ---
                    let critical_value = match self.confidence_level {
                        0.90 => 1.645,
                        0.95 => 1.960,
                        0.99 => 2.576,
                        _ => 1.960,
                    };

                    // 检验原假设 H0: Mean(Diff) = target_mean_diff
                    if res.t_statistic.abs() > critical_value {
                        // T 检验失败：实际平均差值与目标差值存在显著差异
                        ui.colored_label(egui::Color32::RED, self.lang.compare_t_test_significant); // I18N
                    } else {
                        // T 检验通过：实际平均差值与目标差值不存在显著差异
                        ui.colored_label(egui::Color32::GREEN, self.lang.compare_t_test_not_significant); // I18N
                    }
                    // ------------------------------------
                });
            });
                }); // 关闭 CollapsingHeader "一致性检验分析"
        }

        // ⭐ 同步曲线复选框 - 始终显示
        ui.horizontal(|ui| {
            ui.checkbox(&mut self.compare_plots_linked, "🔗 同步曲线缩放/位移");
            ui.label(self.lang.compare_plot_raw_label); // I18N
        });

        // 双图表显示 - 始终显示
        let link_group_id = egui::Id::new("compare_plots_link_group");
        let height = ui.available_height() / 2.0 - 20.0;

        // 定义变量捕获跳转意图
        let mut compare_seek_request: Option<f64> = None;
        let compare_duration = self.compare_player.duration;

        // ⭐ 在闭包外部预先读取输入状态，避免借用冲突
        let pointer_down = ui.input(|i| i.pointer.primary_down());

        // 上图：原始曲线对比 - 始终显示
        // ⭐ 修复 ID 冲突：为 Plot 控件提供唯一的 ID 源
        ui.push_id("compare_raw_plot", |ui| {
            let mut plot = Plot::new("compare_raw")
                .height(height)
                .legend(Legend::default())
                // ⭐ 按住 Shift 键时才允许拖拽进度，否则是平移视图
                .allow_drag(!ui.input(|i| i.modifiers.shift));

            // 如果启用同步，链接轴
            if self.compare_plots_linked {
                plot = plot.link_axis(link_group_id, true)
                    .link_cursor(link_group_id, true);
            }

            plot.show(ui, |plot_ui| {
                // 添加播放位置指示线
                if self.compare_player.is_playing || self.compare_player.current_position > 0.0 {
                    plot_ui.vline(egui_plot::VLine::new("playback_position", self.compare_player.current_position)
                        .color(egui::Color32::from_rgb(255, 165, 0)));
                }
                
                if let Some(a) = &self.compare_a {
                    plot_ui.line(Line::new("Track A", PlotPoints::new(a.points.clone())).color(egui::Color32::GREEN));
                }
                if let Some(b) = &self.compare_b {
                    plot_ui.line(Line::new("Track B", PlotPoints::new(b.points.clone())).color(egui::Color32::RED));
                }

                // ⭐ 新增：交互检测
                if plot_ui.response().hovered() && pointer_down {
                    if let Some(pos) = plot_ui.pointer_coordinate() {
                        compare_seek_request = Some(pos.x);
                    }
                }
            });
        });

        // 下图：差值曲线 - 始终显示
        ui.label(self.lang.compare_plot_diff_label); // I18N
        // ⭐ 修复 ID 冲突：为 Plot 控件提供唯一的 ID 源
        ui.push_id("compare_diff_plot", |ui| {
            let mut plot = Plot::new("compare_diff")
                .height(height)
                // ⭐ 按住 Shift 键时才允许拖拽进度，否则是平移视图
                .allow_drag(!ui.input(|i| i.modifiers.shift));
            
            // 如果启用同步，链接轴
            if self.compare_plots_linked {
                plot = plot.link_axis(link_group_id, true)
                    .link_cursor(link_group_id, true);
            }
            
            plot.show(ui, |plot_ui| {
                // 添加播放位置指示线
                if self.compare_player.is_playing || self.compare_player.current_position > 0.0 {
                    plot_ui.vline(egui_plot::VLine::new("playback_position", self.compare_player.current_position)
                        .color(egui::Color32::from_rgb(255, 165, 0)));
                }
                
                // 只有在有对比结果时才绘制差值曲线
                if let Some(res) = &comparison_result_clone {
                    // 差值曲线颜色更改为 CYAN (青色)，提高可读性
                    plot_ui.line(Line::new("Diff", PlotPoints::new(res.diff_points.clone()))
                        .color(egui::Color32::from_rgb(0, 255, 255))
                    );

                    // 绘制平均线
                    plot_ui.hline(egui_plot::HLine::new("Mean Diff", res.mean_diff)
                        .color(egui::Color32::GRAY)
                        .style(egui_plot::LineStyle::Dashed { length: 5.0 })
                    );

                    // 新增: 绘制零点线，提高可读性
                    plot_ui.hline(egui_plot::HLine::new("Zero", 0.0)
                        .color(egui::Color32::WHITE) // 零点线使用白色突出显示
                        .style(egui_plot::LineStyle::Solid)
                    );
                }

                // ⭐ 新增：交互检测 (同样允许在差值图上拖拽)
                if plot_ui.response().hovered() && pointer_down {
                    if let Some(pos) = plot_ui.pointer_coordinate() {
                        compare_seek_request = Some(pos.x);
                    }
                }
            });
        });

        // ⭐ 统一处理对比模式的跳转
        if let Some(seek_time) = compare_seek_request {
            let target_time = seek_time.clamp(0.0, compare_duration);
            let _ = self.compare_player.seek_to(target_time);
            
            // 强制刷新 UI 以保证线条位置立刻更新
            ctx.request_repaint();
        }
    }


    /// 处理命令行输入
    fn handle_command(&mut self, cmd: String) {
        log_command(&self.logger, &format!("Executed: {}", cmd));
        self.error_msg = None;

        let parts: Vec<&str> = cmd.split_whitespace().collect();
        if parts.is_empty() { return; }

        match parts[0].to_lowercase().as_str() {
            "kill" => {
                if parts.len() == 2 {
                    if let Ok(id) = parts[1].parse::<usize>() {
                        self.worker_pool.command_tx.send(WorkerCommand::Kill(id)).unwrap_or_default();
                    } else {
                        self.error_msg = Some("❌ 命令错误: 'kill <id>' 需要一个数字 ID.".to_string());
                    }
                } else {
                    self.error_msg = Some("❌ 命令错误: 用法: kill <task_id>".to_string());
                }
            }
            "tasks" | "list" => {
                if let Ok(tasks) = self.worker_pool.tasks.lock() {
                    let mut msg = String::from("当前任务列表:\n");
                    for task in tasks.iter() {
                        let state_str = match &task.state {
                            TaskState::Running(p) => format!("Running ({:.0}%)", p * 100.0),
                            s => format!("{:?}", s),
                        };
                        msg.push_str(&format!("ID: {}, Name: {}, State: {}\n", task.id, task.name, state_str));
                    }
                    log_info(&self.logger, &msg);
                }
            }
            "clear" => {
                if let Ok(mut entries) = self.logger.entries.lock() {
                    entries.clear();
                    log_info(&self.logger, "控制台日志已清空。");
                }
            }
            "quit" | "exit" => {
                self.worker_pool.command_tx.send(WorkerCommand::Shutdown).unwrap_or_default();
                self.error_msg = Some("⚠️ 已发送关闭信号给工作池。请手动关闭窗口。".to_string());
            }
            _ => {
                self.error_msg = Some(format!("❌ 未找到命令: {}", parts[0]));
            }
        }
    }

    /// 控制台模式 UI (包含命令行和进程监视器)
    fn ui_console_mode(&mut self, ui: &mut egui::Ui) {
        ui.heading(self.lang.mode_console);
        ui.separator();

        // 估算顶部标题、分隔符和底部命令行输入区域的固定高度 (约 70.0)
        let reserved_height = 70.0;
        let available_height_for_group = ui.available_height() - reserved_height; // 计算 ScrollArea 可用的高度

        // 1. 进程监视器和调试日志 (使用 Group 包含，并使用 ScrollArea)
        // ⭐ 修复 ID 冲突：为 Group 控件提供唯一的 ID 源
        ui.push_id("console_monitor_group", |ui| {
            ui.group(|ui| {
                // 关键：限定 ScrollArea 所在 group 的高度
                ui.set_height(available_height_for_group.max(100.0));

                // 使用 Columns 分离，方便左右布局
                ui.columns(2, |columns| {
                    // --- 进程监视器 (左侧列) ---
                    columns[0].vertical(|ui| {
                        ui.heading("📊 实时进程监视器");
                        // ⭐ 修复 E0501/E0500: 在 vertical 闭包传入的 'ui' 上调用 push_id
                        ui.push_id("process_monitor_scroll", |ui| {
                            egui::ScrollArea::vertical().auto_shrink([false, false]).show(ui, |ui| {
                                if let Ok(tasks) = self.worker_pool.tasks.lock() {
                                    if tasks.is_empty() {
                                        ui.label("当前无活动任务。");
                                    } else {
                                        // 遍历所有任务
                                        for task in tasks.iter() {
                                            // ⭐ 修复 ID 冲突：为每个任务行提供唯一的 ID
                                            ui.push_id(format!("task_{}", task.id), |ui| {
                                                ui.horizontal(|ui| {
                                                    let id_text = format!("[{}]", task.id);
                                                    let state_text = match &task.state {
                                                        TaskState::Waiting => egui::RichText::new("WAITING").color(egui::Color32::GRAY),
                                                        TaskState::Running(progress) => egui::RichText::new(format!("RUNNING ({:.0}%)", progress * 100.0)).color(egui::Color32::GREEN),
                                                        TaskState::Completed => egui::RichText::new("COMPLETED").color(egui::Color32::BLUE),
                                                        TaskState::Killed => egui::RichText::new("KILLED").color(egui::Color32::RED),
                                                        TaskState::Error(e) => egui::RichText::new(format!("ERROR: {}", e)).color(egui::Color32::DARK_RED),
                                                    };

                                                    ui.label(egui::RichText::new(id_text).strong());
                                                    ui.add_space(5.0);
                                                    ui.label(task.name.clone());
                                                    ui.add_space(5.0);
                                                    ui.label(state_text);

                                                    // 仅对 Running 或 Waiting 的任务显示 Kill 按钮
                                                    if matches!(task.state, TaskState::Running(_)) || task.state == TaskState::Waiting {
                                                        if ui.button("❌ Kill").clicked() {
                                                            self.worker_pool.command_tx.send(WorkerCommand::Kill(task.id)).unwrap_or_default();
                                                        }
                                                    }
                                                });
                                            });
                                        }
                                    }
                                }
                            });
                        });
                    });

                    // --- 控制台/日志 (右侧列) ---
                    columns[1].vertical(|ui| {
                        ui.heading("🗒️ 调试日志");
                        // ⭐ 修复 E0501/E0500: 在 vertical 闭包传入的 'ui' 上调用 push_id
                        ui.push_id("debug_log_scroll", |ui| {
                            egui::ScrollArea::vertical().auto_shrink([false, false]).show(ui, |ui| {
                                if let Ok(entries) = self.logger.entries.lock() {
                                    for entry in entries.iter().rev() { // 倒序显示，最新日志在最上面
                                        let color = match entry.level {
                                            LogLevel::Info => egui::Color32::LIGHT_GREEN,
                                            LogLevel::Error => egui::Color32::RED,
                                            LogLevel::Debug => egui::Color32::YELLOW,
                                            LogLevel::Command => egui::Color32::LIGHT_BLUE,
                                        };

                                        let level_text = format!("{:?}", entry.level).to_uppercase();
                                        let log_text = format!("[{}] <{}> {}", entry.time, level_text, entry.message);
                                        ui.colored_label(color, log_text);
                                    }
                                }
                            });
                        });
                    });
                });
            });
        });

        ui.separator();

        // 2. 命令行输入 (底部)
        ui.horizontal(|ui| {
            // 修正：使用 I18N 字段替代硬编码的 "CMD >"
            ui.label(egui::RichText::new(self.lang.console_cmd_label).strong());
            let response = ui.add(
                egui::TextEdit::singleline(&mut self.cmd_input)
                    .desired_width(ui.available_width() - 80.0)
                    .id(egui::Id::new("cmd_input_field")) // 确保输入框 ID 唯一
            );

            // 监听回车键和失焦事件
            if response.lost_focus() && ui.input(|i| i.key_pressed(egui::Key::Enter)) {
                self.handle_command(self.cmd_input.trim().to_string());
                self.cmd_input.clear();
                response.request_focus();
            }

            if ui.button("执行").clicked() && !self.cmd_input.is_empty() {
                self.handle_command(self.cmd_input.trim().to_string());
                self.cmd_input.clear();
                response.request_focus();
            }
        });
        // 修正：使用 I18N 字段替代 if/else 逻辑
        ui.label(self.lang.console_cmd_hint_cn);
    }
}

fn main() -> Result<(), eframe::Error> {

    let options = eframe::NativeOptions {
        viewport: egui::ViewportBuilder::default()
            .with_inner_size([1024.0, 768.0])
            .with_title("WAV Dynamics Analyzer (Rust GUI)"),
        ..Default::default()
    };
    eframe::run_native(
        "WAV Analyzer",
        options,
        Box::new(|cc| Ok(Box::new(WavLufsApp::new(cc)))),
    )
}