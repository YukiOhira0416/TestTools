use slint::*;
use std::sync::{Arc, Mutex};
use device_query::{DeviceQuery, DeviceState, Keycode};

mod player;
use player::VideoPlayer;

slint::include_modules!();

fn main() {
    // 高DPIスケーリングを無効化（実ピクセルで動作）
    std::env::set_var("SLINT_SCALE_FACTOR", "1.0");
    std::env::set_var("WINIT_HIDPI_FACTOR", "1.0");
    
    let ui = VideoPlayerUI::new().unwrap();
    
    // 動画プレイヤーのインスタンスを作成
    let video_player = Arc::new(Mutex::new(VideoPlayer::new()));
    
    // 動画選択コールバック
    let ui_weak = ui.as_weak();
    let player_clone = Arc::clone(&video_player);
    ui.on_select_video(move || {
        let ui = ui_weak.unwrap();
        
        if let Some(path) = rfd::FileDialog::new()
            .add_filter("Video Files", &["mp4", "avi", "mkv", "mov", "wmv", "flv", "webm"])
            .add_filter("All Files", &["*"])
            .pick_file()
        {
            let mut player = player_clone.lock().unwrap();
            match player.load_video(path.clone()) {
                Ok(_) => {
                    ui.set_video_path(path.display().to_string().into());
                    ui.set_duration(player.duration);
                    
                    // 最初のフレームを表示
                    if let Some(frame) = player.get_current_frame() {
                        let width = frame.width();
                        let height = frame.height();
                        let buffer = SharedPixelBuffer::<Rgba8Pixel>::clone_from_slice(
                            frame.as_raw(),
                            width,
                            height,
                        );
                        ui.set_video_frame(Image::from_rgba8(buffer));
                    }
                    
                    println!("動画を選択しました: {}", path.display());
                }
                Err(e) => {
                    eprintln!("エラー: {}", e);
                }
            }
        }
    });
    
    // 再生/一時停止コールバック
    let ui_weak = ui.as_weak();
    let player_clone = Arc::clone(&video_player);
    ui.on_play_pause(move || {
        let ui = ui_weak.unwrap();
        let mut player = player_clone.lock().unwrap();
        
        if player.is_playing() {
            player.pause();
            ui.set_is_playing(false);
        } else {
            match player.play() {
                Ok(_) => {
                    ui.set_is_playing(true);
                }
                Err(e) => {
                    eprintln!("再生エラー: {}", e);
                }
            }
        }
    });
    
    // 停止コールバック
    let ui_weak = ui.as_weak();
    let player_clone = Arc::clone(&video_player);
    ui.on_stop(move || {
        let ui = ui_weak.unwrap();
        let mut player = player_clone.lock().unwrap();
        player.stop();
        ui.set_is_playing(false);
        ui.set_current_time(0.0);
    });
    
    // シークコールバック
    let player_clone = Arc::clone(&video_player);
    ui.on_seek(move |time| {
        let mut player = player_clone.lock().unwrap();
        player.seek(time);
    });
    
    // リピート回数変更コールバック
    ui.on_repeat_changed(move |count| {
        if count == -1 {
            println!("リピート: 無限");
        } else {
            println!("リピート回数: {}回", count);
        }
    });
    
    // ボリューム変更コールバック
    let player_clone = Arc::clone(&video_player);
    ui.on_volume_changed(move |volume| {
        let mut player = player_clone.lock().unwrap();
        player.set_volume(volume);
    });
    
    // キーボード状態を監視するためのデバイス
    let device_state = DeviceState::new();
    
    // Ctrl+Alt押下の前回状態を記憶（連続トグル防止）
    let last_fullscreen_key_pressed = Arc::new(Mutex::new(false));
    
    // 再生時間とフレーム更新用タイマー
    let ui_weak = ui.as_weak();
    let player_clone = Arc::clone(&video_player);
    let last_key_pressed = Arc::clone(&last_fullscreen_key_pressed);
    let timer = Timer::default();
    timer.start(
        TimerMode::Repeated,
        std::time::Duration::from_millis(33), // 約30fps
        move || {
            let ui = ui_weak.unwrap();
            let player = player_clone.lock().unwrap();
            
            // キーボードショートカットチェック（Ctrl + Alt）
            let keys = device_state.get_keys();
            let ctrl_pressed = keys.contains(&Keycode::LControl) || keys.contains(&Keycode::RControl);
            let alt_pressed = keys.contains(&Keycode::LAlt) || keys.contains(&Keycode::RAlt);
            let fullscreen_key_combo = ctrl_pressed && alt_pressed;
            
            let mut last_pressed = last_key_pressed.lock().unwrap();
            if fullscreen_key_combo && !*last_pressed {
                // Ctrl+Altが押された（エッジ検出）
                let current_mode = ui.get_fullscreen_mode();
                let new_mode = !current_mode;
                ui.set_fullscreen_mode(new_mode);
                // モニタ全体のフルスクリーン切替
                ui.window().set_fullscreen(new_mode);
                *last_pressed = true;
            } else if !fullscreen_key_combo {
                // キーが離された
                *last_pressed = false;
            }
            drop(last_pressed);
            
            // 現在の再生時間を更新（再生中のみ）
            let is_playing = player.is_playing();
            let current = player.get_current_time();
            if is_playing {
                ui.set_current_time(current);
            }
            
            // 再生状態を同期
            if ui.get_is_playing() != is_playing {
                ui.set_is_playing(is_playing);
            }
            
            // フレームを更新
            if let Some(frame) = player.get_current_frame() {
                let width = frame.width();
                let height = frame.height();
                let buffer = SharedPixelBuffer::<Rgba8Pixel>::clone_from_slice(
                    frame.as_raw(),
                    width,
                    height,
                );
                ui.set_video_frame(Image::from_rgba8(buffer));
            }
            
            // 動画終了時の処理
            if is_playing && current >= player.duration && player.duration > 0.0 {
                drop(player); // ロックを解放
                let mut player = player_clone.lock().unwrap();
                
                let repeat_count = ui.get_repeat_count();
                if repeat_count == -1 {
                    // 無限リピート
                    player.stop();
                    let _ = player.play();
                } else if repeat_count > 1 {
                    // リピートカウント減少
                    ui.set_repeat_count(repeat_count - 1);
                    player.stop();
                    let _ = player.play();
                } else {
                    // 再生終了
                    player.stop();
                    ui.set_is_playing(false);
                }
            }
        },
    );
    
    println!("動画プレイヤーUIを起動しました");
    println!("動画を選択して再生してください");
    println!("【必要】FFmpegがPATH に設定されていることを確認してください");
    
    ui.run().unwrap();
    
    // アプリ終了時のクリーンアップ
    println!("アプリケーションを終了します...");
    let mut player = video_player.lock().unwrap();
    player.stop();
    println!("クリーンアップ完了");
}
