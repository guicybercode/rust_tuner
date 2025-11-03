use cpal::traits::{DeviceTrait, HostTrait, StreamTrait};
use cpal::{Device, SampleFormat, SampleRate, StreamConfig};
use crossbeam_channel::Sender;

pub struct AudioCapture {
    device: Device,
    config: StreamConfig,
}

impl AudioCapture {
    pub fn new() -> Result<Self, String> {
        let host = cpal::default_host();
        let device = host
            .default_input_device()
            .ok_or("No input device available")?;

        let config = device
            .default_input_config()
            .map_err(|e| format!("Failed to get default config: {}", e))?
            .into();

        Ok(AudioCapture { device, config })
    }

    pub fn start_capture(&self, _sample_rate: SampleRate, sender: Sender<Vec<f32>>) -> Result<cpal::Stream, String> {
        let err_fn = |err| eprintln!("Error in audio stream: {}", err);

        let stream = match self.device.default_input_config() {
            Ok(config) => {
                let sample_format = config.sample_format();
                let config: StreamConfig = config.into();
                match sample_format {
                    SampleFormat::F32 => self.device
                        .build_input_stream(
                            &config,
                            move |data: &[f32], _: &cpal::InputCallbackInfo| {
                                let samples: Vec<f32> = data.to_vec();
                                let _ = sender.try_send(samples);
                            },
                            err_fn,
                            None,
                        )
                        .map_err(|e| format!("Failed to build stream: {}", e))?,
                    SampleFormat::I16 => self.device
                        .build_input_stream(
                            &config,
                            move |data: &[i16], _: &cpal::InputCallbackInfo| {
                                let samples: Vec<f32> = data.iter().map(|s| *s as f32 / 32768.0).collect();
                                let _ = sender.try_send(samples);
                            },
                            err_fn,
                            None,
                        )
                        .map_err(|e| format!("Failed to build stream: {}", e))?,
                    SampleFormat::U16 => self.device
                        .build_input_stream(
                            &config,
                            move |data: &[u16], _: &cpal::InputCallbackInfo| {
                                let samples: Vec<f32> = data.iter().map(|s| (*s as f32 / 65535.0) * 2.0 - 1.0).collect();
                                let _ = sender.try_send(samples);
                            },
                            err_fn,
                            None,
                        )
                        .map_err(|e| format!("Failed to build stream: {}", e))?,
                    _ => return Err("Unsupported sample format".to_string()),
                }
            }
            Err(e) => return Err(format!("Failed to get input config: {}", e)),
        };

        stream.play().map_err(|e| format!("Failed to play stream: {}", e))?;
        Ok(stream)
    }

    pub fn sample_rate(&self) -> u32 {
        self.config.sample_rate.0
    }
}

