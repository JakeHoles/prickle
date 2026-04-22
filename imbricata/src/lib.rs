extern crate anyhow;
use anyhow::Result;

extern crate cpal;
use cpal::{
    Sample,
    traits::{DeviceTrait, HostTrait, StreamTrait},
};

mod audio_buffer_ring;

pub fn host_device_setup() -> Result<(cpal::Host, cpal::Device, cpal::StreamConfig), anyhow::Error>
{
    let host = cpal::default_host();

    let device = host
        .default_output_device()
        .ok_or_else(|| anyhow::Error::msg("Default output device is not available"))?;
    println!("Output device: {}", device.id()?);

    let config = device.default_output_config()?;
    println!("Default output config: {config:?}");

    Ok((host, device, config.into()))
}

pub fn play_beep() -> Result<()> {
    let host = cpal::default_host();
    let device = host
        .default_output_device()
        .expect("failed to find output device");

    let config: cpal::StreamConfig = device.default_output_config().unwrap().into();
    println!("Default output config: {config:?}");

    let sample_rate = config.sample_rate as f32;
    let channels = config.channels as usize;

    // Produce a sinusoid of maximum amplitude.
    let mut sample_clock = 0f32;
    let mut next_value = move |channel_index: &u32| {
        sample_clock = (sample_clock + 1.0) % sample_rate;
        (sample_clock * 440.0 * 2.0 * std::f32::consts::PI / sample_rate).sin()
    };

    let err_fn = |err| eprintln!("an error occurred on stream: {err}");

    let stream = device.build_output_stream(
        &config,
        move |data: &mut [f32], _: &cpal::OutputCallbackInfo| {
            write_data(data, channels, &mut next_value)
        },
        err_fn,
        None,
    )?;
    stream.play()?;

    std::thread::sleep(std::time::Duration::from_millis(1000));

    Ok(())
}

fn write_data(output: &mut [f32], channels: usize, next_sample: &mut dyn FnMut(&u32) -> f32) {
    println!("{channels} channels");
    for frame in output.chunks_mut(channels) {
        let mut channel_index: u32 = 0;
        for sample in frame.iter_mut() {
            let value: f32 = f32::from_sample(next_sample(&channel_index));
            *sample = value;
            channel_index += 1;
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn it_works() {}
}
