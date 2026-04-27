use std::sync::atomic::{AtomicUsize, Ordering};

pub struct SampleRing<T, const CAPACITY: usize>
where
    T: std::default::Default + std::marker::Copy,
    [(); 2 as usize * CAPACITY]:,
{
    data: [T; 2 as usize * CAPACITY],
    write_index: AtomicUsize,
    read_index: AtomicUsize,
}

impl<T, const CAPACITY: usize> SampleRing<T, CAPACITY>
where
    T: std::default::Default + std::marker::Copy,
    [(); 2 as usize * CAPACITY]:,
{
    pub fn new() -> SampleRing<T, CAPACITY> {
        SampleRing {
            data: [T::default(); 2 as usize * CAPACITY],
            write_index: AtomicUsize::new(0),
            read_index: AtomicUsize::new(0),
        }
    }

    pub fn write(self: &mut Self, write_data: &[T]) -> Result<(), &str> {
        let write_i = self.write_index.load(Ordering::Relaxed);
        loop {
            let read_i = self.read_index.load(Ordering::Acquire);
            let usage = if write_i >= read_i {
                write_i - read_i
            } else {
                (2 * CAPACITY - read_i) + write_i
            };

            if usage + write_data.len() <= CAPACITY {
                if read_i == self.read_index.load(Ordering::Acquire) {
                    for i in 0..write_data.len() {
                        self.data[(write_i + i) % (2 * CAPACITY)] = write_data[i]
                    }
                    self.write_index.store(
                        (write_i + write_data.len()) % (2 * CAPACITY),
                        Ordering::Release,
                    );
                    return Ok(());
                } else {
                    continue;
                }
            } else {
                if read_i == self.read_index.load(Ordering::Acquire) {
                    return Err(
                        "sample ring has {remaining_capacity} remaining capacity. can't write {write_data.len()} elements",
                    );
                } else {
                    continue;
                }
            }
        }

        return Ok(());
    }

    pub fn read_n_elements(self: &mut Self, n: usize) -> Result<Vec<T>, &str> {
        let read_i = self.read_index.load(Ordering::Relaxed);
        loop {
            let write_i = self.write_index.load(Ordering::Acquire);
            let usage = if write_i >= read_i {
                write_i - read_i
            } else {
                (2 * CAPACITY - read_i) + write_i
            };

            if n < usage {
                if write_i == self.write_index.load(Ordering::Acquire) {
                    let mut read_result = Vec::with_capacity(n);
                    for i in 0..n {
                        read_result.push(self.data[(read_i + i) % (2 * CAPACITY)]);
                    }
                    self.read_index
                        .store((read_i + n) % (2 * CAPACITY), Ordering::Release);
                    return Ok(read_result);
                } else {
                    continue;
                }
            } else {
                if write_i == self.write_index.load(Ordering::Acquire) {
                    return Err("only {usage} elements in sample ring. can't read {n} elements");
                } else {
                    continue;
                }
            }
        }

        return Err("no read occurred");
    }

    pub fn read_to_buffer<const READ_SIZE: usize>(
        self: &mut Self,
        read_buffer: &mut [T; READ_SIZE],
    ) -> Result<(), &str> {
        let read_i = self.read_index.load(Ordering::Relaxed);
        loop {
            let write_i = self.write_index.load(Ordering::Acquire);
            let usage = if write_i >= read_i {
                write_i - read_i
            } else {
                (2 * CAPACITY - read_i) + write_i
            };

            if READ_SIZE <= usage {
                if write_i == self.write_index.load(Ordering::Acquire) {
                    for i in 0..READ_SIZE {
                        read_buffer[i] = self.data[(read_i + i) % (2 * CAPACITY)];
                    }
                    self.read_index
                        .store((read_i + READ_SIZE) % (2 * CAPACITY), Ordering::Release);
                    return Ok(());
                } else {
                    continue;
                }
            } else {
                if write_i == self.write_index.load(Ordering::Acquire) {
                    return Err("only {usage} elements in sample ring. can't read {n} elements");
                } else {
                    continue;
                }
            }
        }

        return Err("no read occurred");
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_write_capacity() -> Result<(), &'static str> {
        const CAPACITY: usize = 256;

        let mut buf = SampleRing::<u32, CAPACITY>::new();

        let too_many_samples: [u32; 512] = [0; 512];
        if let Ok(()) = buf.write(&too_many_samples) {
            return Err("trying to write too many samples should have failed");
        };

        let samples: [u32; 255] = [0; 255];
        let Ok(()) = buf.write(&samples) else {
            return Err("write to buffer of valid size failed");
        };

        let couple_samples: [u32; 2] = [0; 2];
        if let Ok(()) = buf.write(&couple_samples) {
            return Err(
                "writing 2 samples back to back with writing 255 samples should have exceeded capacity",
            );
        };

        return Ok(());
    }

    #[test]
    fn test_reads() -> Result<(), String> {
        const CAPACITY: usize = 256;

        let mut ring = SampleRing::<u32, CAPACITY>::new();

        if let Ok(read_result) = ring.read_n_elements(12) {
            return Err("shouldn't have been enough samples to read into buffer".to_string());
        };

        let samples: [u32; 5] = [0, 1, 2, 3, 4];
        let Ok(()) = ring.write(&samples) else {
            return Err("failed to write samples to empty sample ring".to_string());
        };

        match ring.read_n_elements(4) {
            Ok(read_result) => {
                assert_eq!(0, read_result[0]);
                assert_eq!(1, read_result[1]);
                assert_eq!(2, read_result[2]);
                assert_eq!(3, read_result[3]);
            }
            Err(err) => {
                return Err(format!(
                    "failed to read samples from ring with data: error {}",
                    err
                ));
            }
        };

        let next_samples: [u32; 5] = [5, 6, 7, 8, 9];
        let Ok(()) = ring.write(&next_samples) else {
            return Err("failed to write samples".to_string());
        };

        let mut read_buffer: [u32; 6] = [0; 6];
        match ring.read_to_buffer(&mut read_buffer) {
            Ok(()) => {
                assert_eq!(read_buffer[0], 4);
                assert_eq!(read_buffer[1], 5);
                assert_eq!(read_buffer[2], 6);
                assert_eq!(read_buffer[3], 7);
                assert_eq!(read_buffer[4], 8);
                assert_eq!(read_buffer[5], 9);
            }
            Err(err) => {
                return Err("failed to read to buffer".to_string());
            }
        };

        return Ok(());
    }
}
