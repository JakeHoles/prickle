use std::sync::atomic::{AtomicPtr, AtomicUsize};

struct AudioBuffer<T, const BUFF_SIZE: usize>
where
    T: std::default::Default + std::marker::Copy,
{
    data: [T; BUFF_SIZE],
}

impl<T, const BUFF_SIZE: usize> AudioBuffer<T, BUFF_SIZE>
where
    T: std::default::Default + std::marker::Copy,
{
    pub fn new() -> AudioBuffer<T, BUFF_SIZE> {
        AudioBuffer {
            data: [T::default(); BUFF_SIZE],
        }
    }
}

struct AudioBufferRing<T, const BUFF_SIZE: usize, const N: usize>
where
    T: std::default::Default + std::marker::Copy,
{
    ring: [AtomicPtr<AudioBuffer<T, BUFF_SIZE>>; N],
    write_index: AtomicUsize,
}

impl<T, const BUFF_SIZE: usize, const N: usize> AudioBufferRing<T, BUFF_SIZE, N>
where
    T: std::default::Default + std::marker::Copy,
{
    pub fn new() -> AudioBufferRing<T, BUFF_SIZE, N> {
        AudioBufferRing {
            ring: [const { AtomicPtr::new(std::ptr::null_mut()) }; N],
            write_index: AtomicUsize::new(0),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_create_ring() {
        let audio_ring = AudioBufferRing::<f32, 64, 1024>::new();
    }
}
