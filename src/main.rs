use cpal::{
    traits::{DeviceTrait, HostTrait, StreamTrait},
    FromSample, SizedSample, StreamConfig,
};
use fundsp::hacker::*;
use std::io;

fn main() -> io::Result<()> {
    let host = cpal::default_host();
    let device = host
        .default_output_device()
        .expect("Failed to retreive default output device");

    let config = device.default_output_config().unwrap();

    println!(
        "Device info: {:?} {:?}",
        config.sample_format(),
        config.sample_rate()
    );

    run::<f32>(&device, &config.into()).unwrap();

    Ok(())
}

fn run<T>(device: &cpal::Device, config: &StreamConfig) -> Result<(), ()>
where
    T: SizedSample + FromSample<f64>,
{
    let channels = config.channels as usize;

    let a = 0.6 * (organ_hz(midi_hz(57.0)) + organ_hz(midi_hz(61.0)) + organ_hz(midi_hz(64.0)));
    let a = a >> pan(0.0);
    let a = a >> (declick() | declick()) >> (dcblock() | dcblock());
    let a = a >> (chorus(0, 0.0, 0.01, 0.2) | chorus(1, 0.0, 0.01, 0.2));
    let mut a = a >> limiter_stereo((0.4, 0.5));

    a.set_sample_rate(config.sample_rate.0 as f64);
    a.allocate();

    let error_callback = |e| eprint!("Error {}", e);
    let mut next_sample = move || a.get_stereo();

    let stream = device
        .build_output_stream(
            config,
            move |data: &mut [T], _: &cpal::OutputCallbackInfo| {
                for frame in data.chunks_mut(channels) {
                    let (next_sample_left, next_sample_right) = next_sample();

                    for (channel, sample) in frame.iter_mut().enumerate() {
                        if channel == 0 {
                            *sample = T::from_sample(next_sample_left);
                        } else if channel == 1 {
                            *sample = T::from_sample(next_sample_right);
                        }
                    }
                }
            },
            error_callback,
            None,
        )
        .unwrap();

    stream.play().unwrap();

    std::thread::sleep(std::time::Duration::from_millis(10000));

    Ok(())
}
