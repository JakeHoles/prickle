#![feature(generic_const_exprs)]

extern crate anyhow;
use anyhow::Result;

extern crate cpal;
use cpal::traits::{DeviceTrait, HostTrait};

mod sample_ring;

pub fn default_host_device_setup()
-> Result<(cpal::Host, cpal::Device, cpal::SupportedStreamConfig), anyhow::Error> {
    let host = cpal::default_host();

    let device = host
        .default_output_device()
        .ok_or_else(|| anyhow::Error::msg("Default output device is not available"))?;
    println!("Output device: {}", device.id()?);

    let config = device.default_output_config()?;
    println!("Default output config: {config:?}");

    Ok((host, device, config))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn create_audio_ring_with_config() -> Result<(), &'static str> {
        let Ok((_host, _device, stream_config)) = default_host_device_setup() else {
            return Err("failed to get default host and device");
        };

        let sample_format = stream_config.sample_format();
        println!("sample format is {sample_format}");
        return Ok(());
    }
}
