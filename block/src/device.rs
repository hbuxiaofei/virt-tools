use libc;
use std::fs;
use std::fs::File;
use std::fs::OpenOptions;
use std::io::Read;
use std::os::unix::fs::FileExt;
use std::os::unix::fs::FileTypeExt;
use std::os::unix::fs::OpenOptionsExt;

#[derive(Default, Debug, Clone)]
pub struct BlockDevice {
    dev_path: String,
    size: u64,
}

const CHUNK_SIZE: u64 = 4096;

// `O_DIRECT` requires all reads and writes
// to be aligned to the block device's block
// size. 4096 might not be the best, or even
// a valid one, for yours!
#[repr(align(4096))]
struct Aligned([u8; CHUNK_SIZE as usize]);

impl BlockDevice {
    pub fn new(path: &str) -> Result<Self, String> {
        let meta = fs::metadata(path).unwrap();
        let file_type = meta.file_type();

        if !file_type.is_block_device() {
            return Err(format!("{} is not a block device", path));
        }

        let path_string = path.to_string();

        let pos: Vec<&str> = path_string.split("/").collect();
        let size_file = format!("/sys/class/block/{}/size", pos[pos.len() - 1]);
        let mut f = File::open(size_file.to_string()).unwrap();
        let mut buf = String::new();
        f.read_to_string(&mut buf).unwrap();

        let buf = buf.as_str().trim();
        Ok(BlockDevice {
            dev_path: path.to_string(),
            size: 512 * buf.to_string().parse::<u64>().unwrap(),
        })
    }

    pub fn get_disk_size(&self) -> u64 {
        self.size
    }

    pub fn read(&mut self, buf: &mut [u8]) -> usize {
        let mut options = OpenOptions::new();
        options.read(true);
        if cfg!(unix) {
            options.custom_flags(libc::O_RDONLY);
        }

        let mut f = options.open(self.dev_path.as_str()).unwrap();
        let size = f.read(buf).unwrap();
        size
    }

    pub fn read_at(&self, buf: &mut [u8], offset: u64) -> usize {
        let mut options = OpenOptions::new();
        options.read(true);
        if cfg!(unix) {
            options.custom_flags(libc::O_RDONLY);
        }

        let f = options.open(self.dev_path.as_str()).unwrap();
        let size = f.read_at(buf, offset).unwrap();
        size
    }

    pub fn read_direct_at(&self, buf: &mut [u8], offset: u64) -> usize {
        if (buf.len() as u64) % CHUNK_SIZE != 0 {
            return 0;
        }

        if offset % CHUNK_SIZE != 0 {
            return 0;
        }

        let mut options = OpenOptions::new();
        options.read(true);
        if cfg!(unix) {
            options.custom_flags(libc::O_DIRECT);
        }

        let nr_block = (buf.len() as u64) / CHUNK_SIZE;
        let mut read_size = 0;
        let mut out_buf = Aligned([0; CHUNK_SIZE as usize]);

        let f = options.open(self.dev_path.as_str()).unwrap();
        let out_slice: &mut [u8] = &mut out_buf.0;
        for n in 0..nr_block {
            let size = f.read_at(out_slice, offset + n * CHUNK_SIZE).unwrap();

            let start = (n * CHUNK_SIZE) as usize;
            let end = ((n + 1) * CHUNK_SIZE) as usize;
            for i in start..end {
                buf[i] = out_slice[i - start];
            }

            read_size += size;
        }

        read_size
    }

    pub fn write_direct_at(&self, buf: &[u8], offset: u64) -> usize {
        if (buf.len() as u64) % CHUNK_SIZE != 0 {
            return 0;
        }

        if offset % CHUNK_SIZE != 0 {
            return 0;
        }

        let nr_block = (buf.len() as u64) / CHUNK_SIZE;
        let mut write_size = 0;

        let mut options = OpenOptions::new();
        options.write(true);
        if cfg!(unix) {
            options.custom_flags(libc::O_DIRECT);
        }
        let file = options.open(self.dev_path.as_str()).unwrap();

        let mut in_buf = Aligned([0; CHUNK_SIZE as usize]);
        for n in 0..nr_block {
            let start = (n * CHUNK_SIZE) as usize;
            let end = ((n + 1) * CHUNK_SIZE) as usize;

            for i in start..end {
                in_buf.0[i - start] = buf[i];
            }

            let out_slice = in_buf.0.as_ref();
            let size = file.write_at(&out_slice, offset + n * CHUNK_SIZE).unwrap();
            write_size += size;
        }

        write_size
    }

    pub fn show_info(&self) {
        println!("{:?}", self);
    }
}
