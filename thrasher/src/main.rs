extern crate imbricata;

use std::sync::Arc;
use std::sync::atomic::{AtomicBool, Ordering};

use cpal::{
    traits::{DeviceTrait, StreamTrait},
};

use handy_keys::{
    Hotkey, HotkeyManager, Key,
    Modifiers, check_accessibility, open_accessibility_settings,
    KeyboardListener
};

use imbricata::host_device_setup;

#[derive(Default)]
struct ControlInput {
    pan_left_pressed: AtomicBool,
    pan_right_pressed: AtomicBool
}

fn init_keyboard_listener() -> handy_keys::Result<KeyboardListener> {
    let manager = HotkeyManager::new()?;
    let pan_left_hotkey = Hotkey::new(Modifiers::empty(), Key::LeftArrow)?;
    let _pan_left_id = manager.register(pan_left_hotkey)?;
    let pan_right_hotkey = Hotkey::new(Modifiers::empty(), Key::RightArrow)?;
    let _pan_right_id = manager.register(pan_right_hotkey)?;
    KeyboardListener::new()
}

fn main() -> anyhow::Result<()> {
    if !check_accessibility() {
        open_accessibility_settings()?;
    }

    let main_thread_controls = Arc::new(ControlInput::default());
    let audio_controls = Arc::clone(&main_thread_controls);

    let listener = init_keyboard_listener()?;

    let (_host, device, config) = host_device_setup()?;

    let err_fn = |err| eprintln!("Error building output sound stream: {err}");

    let sample_rate = config.sample_rate as f32;
    let channels = config.channels as usize;
    let mut pan_setting = 0f32;
    let pan_incr = 0.1f32;
    let mut sample_clock = 0f32;

    let stream = device.build_output_stream(
        &config,
        move |output: &mut [f32], _: &cpal::OutputCallbackInfo| {
            match audio_controls.pan_left_pressed.compare_exchange(
                true, false,
                Ordering::Acquire,
                Ordering::Relaxed
            ) {
                Ok(_) => {pan_setting -= pan_incr},
                Err(_) => {},
            };
            match audio_controls.pan_right_pressed.compare_exchange(
                true, false,
                Ordering::Acquire,
                Ordering::Relaxed
            ) {
                Ok(_) => { pan_setting += pan_incr},
                Err(_) => {},
            };
            pan_setting = pan_setting.clamp(-1.0, 1.0);

            sample_clock = (sample_clock + 1.0) % sample_rate;
            for frame in output.chunks_mut(channels) {
                let mut channel_index: u32 = 0;
                for sample in frame.iter_mut() {
                    *sample = (sample_clock * 440.0 * 2.0 * std::f32::consts::PI / sample_rate).sin();
                    if channel_index == 0 {
                        *sample = *sample * (-1.0 * (-0.5 + 0.5 * pan_setting));
                    } else {
                        *sample = *sample * (0.5 + 0.5 * pan_setting);
                    }
                    channel_index += 1;
                }
            }
        },
        err_fn,
        None,
    )?;

    stream.play()?;

    while let Ok(event) = listener.recv() {
        if !event.is_key_down {
            continue;
        }

        let hotkey = if let Some(hotkey) = event.key {
            hotkey
        } else {
            continue;
        };

        match hotkey {
            Key::Q => {
                break;
            },
            Key::LeftArrow => {
                main_thread_controls.pan_left_pressed.store(true, Ordering::Release);
            },
            Key::RightArrow => {
                main_thread_controls.pan_right_pressed.store(true, Ordering::Release);
            },
            _ => {
                continue;
            }
        }
    }

    Ok(())
}
