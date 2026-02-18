use std::path::PathBuf;
use std::sync::{Arc, Mutex};
use std::thread;
use std::time::{Duration, Instant};
use std::process::{Command, Stdio, Child};
use std::io::Read;
use image::RgbaImage;

pub struct VideoPlayer {
    pub duration: f32,
    pub is_playing: Arc<Mutex<bool>>,
    pub is_paused: Arc<Mutex<bool>>,
    pub current_time: Arc<Mutex<f32>>,
    pub current_frame: Arc<Mutex<Option<RgbaImage>>>,
    pub seek_time: Arc<Mutex<Option<f32>>>,
    playback_generation: Arc<Mutex<u64>>,
    audio_process: Arc<Mutex<Option<Child>>>,
    audio_generation: Arc<Mutex<u64>>,
    pub volume: Arc<Mutex<f32>>,
    video_path: Option<PathBuf>,
    video_width: u32,
    video_height: u32,
}

impl VideoPlayer {
    pub fn new() -> Self {
        Self {
            duration: 0.0,
            is_playing: Arc::new(Mutex::new(false)),
            is_paused: Arc::new(Mutex::new(false)),
            current_time: Arc::new(Mutex::new(0.0)),
            current_frame: Arc::new(Mutex::new(None)),
            seek_time: Arc::new(Mutex::new(None)),
            playback_generation: Arc::new(Mutex::new(0)),
            audio_process: Arc::new(Mutex::new(None)),
            audio_generation: Arc::new(Mutex::new(0)),
            volume: Arc::new(Mutex::new(1.0)),
            video_path: None,
            video_width: 960,
            video_height: 600,
        }
    }

    pub fn load_video(&mut self, path: PathBuf) -> Result<(), String> {
        self.video_path = Some(path.clone());
        
        // 動画の情報を取得
        match self.get_video_info(&path) {
            Ok((duration, width, height)) => {
                self.duration = duration;
                self.video_width = width;
                self.video_height = height;
                println!("動画を読み込みました: {} ({}秒, {}x{})", path.display(), duration, width, height);
                
                // 最初のフレームを読み込む
                self.load_first_frame(&path)?;
                
                Ok(())
            }
            Err(e) => {
                println!("警告: {}", e);
                self.duration = 300.0;
                self.video_width = 960;
                self.video_height = 600;
                Ok(())
            }
        }
    }

    fn get_video_info(&self, path: &PathBuf) -> Result<(f32, u32, u32), String> {
        // ffprobeで動画情報を取得
        let output = Command::new("ffprobe")
            .args(&[
                "-v", "error",
                "-select_streams", "v:0",
                "-show_entries", "stream=width,height,duration",
                "-of", "csv=p=0",
                path.to_str().unwrap(),
            ])
            .output();

        match output {
            Ok(output) if output.status.success() => {
                let info_str = String::from_utf8_lossy(&output.stdout);
                let parts: Vec<&str> = info_str.trim().split(',').collect();
                
                if parts.len() >= 3 {
                    let width = parts[0].parse::<u32>().unwrap_or(1280);
                    let height = parts[1].parse::<u32>().unwrap_or(720);
                    let duration = parts[2].parse::<f32>().unwrap_or(0.0);
                    
                    // 1280x720にスケーリング
                    let (scaled_width, scaled_height) = self.calculate_scaled_size(width, height);
                    
                    Ok((duration, scaled_width, scaled_height))
                } else {
                    Err("動画情報の解析に失敗".to_string())
                }
            }
            _ => Err("ffprobeが利用できません".to_string())
        }
    }

    fn calculate_scaled_size(&self, orig_width: u32, orig_height: u32) -> (u32, u32) {
        let max_width = 960u32;
        let max_height = 600u32;
        
        if orig_width <= max_width && orig_height <= max_height {
            return (orig_width, orig_height);
        }
        
        let width_ratio = max_width as f32 / orig_width as f32;
        let height_ratio = max_height as f32 / orig_height as f32;
        let ratio = width_ratio.min(height_ratio);
        
        let new_width = (orig_width as f32 * ratio) as u32;
        let new_height = (orig_height as f32 * ratio) as u32;
        
        // 偶数にする（ffmpegの要件）
        (new_width & !1, new_height & !1)
    }

    fn load_first_frame(&mut self, path: &PathBuf) -> Result<(), String> {
        // 最初のフレームを抽出
        let output = Command::new("ffmpeg")
            .args(&[
                "-i", path.to_str().unwrap(),
                "-vf", &format!("scale={}:{}", self.video_width, self.video_height),
                "-vframes", "1",
                "-f", "image2pipe",
                "-vcodec", "ppm",
                "-"
            ])
            .stdout(Stdio::piped())
            .stderr(Stdio::null())
            .output();

        match output {
            Ok(output) if output.status.success() && !output.stdout.is_empty() => {
                match image::load_from_memory(&output.stdout) {
                    Ok(img) => {
                        let rgba = img.to_rgba8();
                        *self.current_frame.lock().unwrap() = Some(rgba);
                        Ok(())
                    }
                    Err(e) => Err(format!("画像の読み込みエラー: {}", e))
                }
            }
            Ok(_) => Err("フレームの抽出に失敗".to_string()),
            Err(e) => Err(format!("ffmpegエラー: {}", e))
        }
    }

    pub fn play(&mut self) -> Result<(), String> {
        // 一時停止からの再開
        if *self.is_paused.lock().unwrap() {
            *self.is_paused.lock().unwrap() = false;
            *self.is_playing.lock().unwrap() = true;
            self.start_audio_playback()?;
            println!("一時停止から再開");
            return Ok(());
        }
        
        if let Some(path) = &self.video_path {
            *self.is_playing.lock().unwrap() = true;
            *self.is_paused.lock().unwrap() = false;
            
            // 再生世代をインクリメント
            let generation = {
                let mut gen = self.playback_generation.lock().unwrap();
                *gen += 1;
                *gen
            };
            
            let path_str = path.to_str().unwrap().to_string();
            let is_playing = Arc::clone(&self.is_playing);
            let is_paused = Arc::clone(&self.is_paused);
            let current_time = Arc::clone(&self.current_time);
            let current_frame = Arc::clone(&self.current_frame);
            let seek_time = Arc::clone(&self.seek_time);
            let playback_generation = Arc::clone(&self.playback_generation);
            let duration = self.duration;
            let width = self.video_width;
            let height = self.video_height;
            
            // 音声再生を開始
            self.start_audio_playback()?;
            
            // 別スレッドで動画を再生
            thread::spawn(move || {
                Self::play_video_with_frames(&path_str, is_playing, is_paused, current_time, current_frame, seek_time, playback_generation, generation, duration, width, height);
            });
            
            Ok(())
        } else {
            Err("動画ファイルが読み込まれていません".to_string())
        }
    }

    fn play_video_with_frames(
        path: &str,
        is_playing: Arc<Mutex<bool>>,
        is_paused: Arc<Mutex<bool>>,
        current_time: Arc<Mutex<f32>>,
        current_frame: Arc<Mutex<Option<RgbaImage>>>,
        seek_time: Arc<Mutex<Option<f32>>>,
        playback_generation: Arc<Mutex<u64>>,
        my_generation: u64,
        duration: f32,
        width: u32,
        height: u32,
    ) {
        // シーク位置を取得
        let start_position = {
            let mut seek = seek_time.lock().unwrap();
            let pos = seek.unwrap_or(0.0);
            *seek = None; // 使用後クリア
            pos
        };
        
        println!("ffmpegで動画を再生中... (開始位置: {}秒, 世代: {})", start_position, my_generation);
        
        // ffmpegでrawvideo形式でフレームを出力（RGBA形式）
        let mut args = vec![
            "-ss".to_string(),
            start_position.to_string(),
        ];
        args.extend_from_slice(&[
            "-re".to_string(), // リアルタイム再生
            "-i".to_string(),
            path.to_string(),
            "-vf".to_string(),
            format!("scale={}:{}", width, height),
            "-f".to_string(),
            "rawvideo".to_string(),
            "-pix_fmt".to_string(),
            "rgba".to_string(),
            "-".to_string(),
        ]);
        
        let mut child = match Command::new("ffmpeg")
            .args(&args)
            .stdout(Stdio::piped())
            .stderr(Stdio::null())
            .spawn()
        {
            Ok(child) => child,
            Err(e) => {
                println!("ffmpegの起動に失敗: {}", e);
                *is_playing.lock().unwrap() = false;
                return;
            }
        };

        let mut stdout = child.stdout.take().unwrap();
        let start_time = Instant::now();
        
        // 一時停止時間の追跡
        let mut pause_start: Option<Instant> = None;
        let mut total_paused_secs: f32 = 0.0;
        
        // 1フレームのサイズを計算（RGBA = 4バイト/ピクセル）
        let frame_size = (width * height * 4) as usize;
        let mut frame_buffer = vec![0u8; frame_size];

        loop {
            // 世代番号をチェック（新しいシークや再生があれば、このスレッドは古くなっている）
            if *playback_generation.lock().unwrap() != my_generation {
                let _ = child.kill();
                println!("新しい再生が開始されたため、古い再生スレッド（世代: {}）を終了", my_generation);
                break;
            }
            
            // シーク要求をチェック
            if seek_time.lock().unwrap().is_some() {
                // 新しいシーク要求があるため、現在の再生を停止
                let _ = child.kill();
                println!("シーク要求により再生を中断");
                break;
            }
            
            // 一時停止チェック: ffmpegプロセスは生かしたまま待機（停止チェックより先）
            if *is_paused.lock().unwrap() {
                if pause_start.is_none() {
                    pause_start = Some(Instant::now());
                }
                thread::sleep(Duration::from_millis(30));
                continue;
            } else if let Some(ps) = pause_start.take() {
                // 一時停止から復帰: 停止していた時間を累積
                total_paused_secs += ps.elapsed().as_secs_f32();
                println!("一時停止から復帰（停止時間: {:.2}秒, 累積: {:.2}秒）", ps.elapsed().as_secs_f32(), total_paused_secs);
            }
            
            // 停止チェック（一時停止でない場合のみ到達）
            if !*is_playing.lock().unwrap() {
                let _ = child.kill();
                println!("再生を停止しました");
                break;
            }

            let elapsed = start_time.elapsed().as_secs_f32() - total_paused_secs + start_position;
            *current_time.lock().unwrap() = elapsed;

            if elapsed >= duration && duration > 0.0 {
                *is_playing.lock().unwrap() = false;
                println!("再生が終了しました");
                break;
            }

            // フレームを読み込む（正確なサイズを読み取る）
            let mut pos = 0;
            while pos < frame_size {
                match stdout.read(&mut frame_buffer[pos..]) {
                    Ok(0) => {
                        // EOFに達した
                        *is_playing.lock().unwrap() = false;
                        println!("動画の終端に達しました");
                        let _ = child.kill();
                        return;
                    }
                    Ok(n) => {
                        pos += n;
                    }
                    Err(e) => {
                        println!("読み込みエラー: {}", e);
                        *is_playing.lock().unwrap() = false;
                        let _ = child.kill();
                        return;
                    }
                }
            }

            // フレームをRgbaImageに変換
            if let Some(rgba_image) = RgbaImage::from_raw(width, height, frame_buffer.clone()) {
                *current_frame.lock().unwrap() = Some(rgba_image);
            }

            // フレームレートを調整するための待機は不要（-reオプションで自動調整）
        }

        let _ = child.wait();
    }

    pub fn pause(&mut self) {
        // is_playing=false（UI同期用）、is_paused=true（スレッド維持用）
        *self.is_playing.lock().unwrap() = false;
        *self.is_paused.lock().unwrap() = true;
        self.stop_audio();
        let current = *self.current_time.lock().unwrap();
        println!("一時停止（位置: {:.2}秒）", current);
    }

    pub fn stop(&mut self) {
        *self.is_paused.lock().unwrap() = false;
        *self.is_playing.lock().unwrap() = false;
        *self.current_time.lock().unwrap() = 0.0;
        self.stop_audio();
        
        // 最初のフレームを再読み込み
        if let Some(path) = self.video_path.clone() {
            let _ = self.load_first_frame(&path);
        }
        
        println!("停止");
    }

    pub fn seek(&mut self, time: f32) {
        let was_playing = self.is_playing() || *self.is_paused.lock().unwrap();
        
        // 一時停止状態をクリア
        *self.is_paused.lock().unwrap() = false;
        
        // 世代番号をインクリメント（古いスレッドを無効化）
        *self.playback_generation.lock().unwrap() += 1;
        
        // 現在の再生を停止（音声も含む）
        *self.is_playing.lock().unwrap() = false;
        self.stop_audio();
        
        // シーク時刻を設定
        *self.current_time.lock().unwrap() = time;
        *self.seek_time.lock().unwrap() = Some(time);
        
        println!("シーク: {}秒", time);
        
        // 指定された位置のフレームを非同期で読み込む
        if let Some(path) = self.video_path.clone() {
            let current_frame = Arc::clone(&self.current_frame);
            let width = self.video_width;
            let height = self.video_height;
            
            thread::spawn(move || {
                Self::load_frame_at_time_async(&path, time, current_frame, width, height);
            });
        }
        
        // 再生中だった場合は、シーク位置から即座に再生を再開
        if was_playing {
            let _ = self.play();
        }
    }
    
    fn load_frame_at_time_async(
        path: &PathBuf,
        time: f32,
        current_frame: Arc<Mutex<Option<RgbaImage>>>,
        width: u32,
        height: u32,
    ) {
        // 指定された時刻のフレームを抽出（高速化のため-ssを-iの前に配置）
        let output = Command::new("ffmpeg")
            .args(&[
                "-ss", &time.to_string(),
                "-i", path.to_str().unwrap(),
                "-vf", &format!("scale={}:{}", width, height),
                "-vframes", "1",
                "-f", "rawvideo",
                "-pix_fmt", "rgba",
                "-"
            ])
            .stdout(Stdio::piped())
            .stderr(Stdio::null())
            .output();

        match output {
            Ok(output) if output.status.success() && !output.stdout.is_empty() => {
                let frame_size = (width * height * 4) as usize;
                if output.stdout.len() >= frame_size {
                    if let Some(rgba_image) = RgbaImage::from_raw(width, height, output.stdout) {
                        *current_frame.lock().unwrap() = Some(rgba_image);
                    }
                }
            }
            Err(e) => {
                println!("フレームの読み込みエラー: {}", e);
            }
            _ => {}
        }
    }

    pub fn get_current_time(&self) -> f32 {
        *self.current_time.lock().unwrap()
    }

    pub fn is_playing(&self) -> bool {
        *self.is_playing.lock().unwrap()
    }

    pub fn get_current_frame(&self) -> Option<RgbaImage> {
        self.current_frame.lock().unwrap().clone()
    }
    
    pub fn set_volume(&mut self, volume: f32) {
        *self.volume.lock().unwrap() = volume.clamp(0.0, 1.0);
        println!("音量を設定: {}%", (volume * 100.0) as i32);
        
        // 再生中の場合は音声を再起動
        if self.is_playing() {
            self.stop_audio();
            let _ = self.start_audio_playback();
        }
    }
    
    fn start_audio_playback(&mut self) -> Result<(), String> {
        // 既存の音声プロセスを停止
        self.stop_audio();
        
        if let Some(path) = &self.video_path {
            let start_position = *self.current_time.lock().unwrap();
            let volume = *self.volume.lock().unwrap();
            
            // 音声世代をインクリメント
            let audio_gen = {
                let mut gen = self.audio_generation.lock().unwrap();
                *gen += 1;
                *gen
            };
            
            println!("音声再生を開始（位置: {}秒, 音量: {}%, 世代: {}）", start_position, (volume * 100.0) as i32, audio_gen);
            
            // ffplayで音声のみを再生（ビデオは非表示）
            let child = Command::new("ffplay")
                .args(&[
                    "-ss", &start_position.to_string(),
                    "-i", path.to_str().unwrap(),
                    "-vn", // ビデオなし
                    "-nodisp", // ウィンドウを表示しない
                    "-af", &format!("volume={}", volume), // ボリュームフィルター
                    "-autoexit", // 終了時に自動で閉じる
                ])
                .stdout(Stdio::null())
                .stderr(Stdio::null())
                .spawn();
            
            match child {
                Ok(process) => {
                    *self.audio_process.lock().unwrap() = Some(process);
                    Ok(())
                }
                Err(e) => {
                    println!("音声再生の開始に失敗: {}", e);
                    Err(format!("ffplayの起動に失敗: {}", e))
                }
            }
        } else {
            Ok(())
        }
    }
    
    fn stop_audio(&mut self) {
        let mut audio_proc = self.audio_process.lock().unwrap();
        if let Some(mut child) = audio_proc.take() {
            let _ = child.kill();
            let _ = child.wait();
            println!("音声プロセスを停止しました");
        }
    }
}

impl Drop for VideoPlayer {
    fn drop(&mut self) {
        // VideoPlayerが破棄される際に確実に音声プロセスを停止
        println!("VideoPlayerをクリーンアップ中...");
        self.stop_audio();
    }
}
