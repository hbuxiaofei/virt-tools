use chrono::prelude::{DateTime, Local};
use sha2::{Digest, Sha256};

use byteorder::{BigEndian, ByteOrder};
use positioned_io::WriteAt;

const MAX_STRING_LENGTH: usize = 68;
const SECTOR_SIZE: u64 = 512;

const MAGIC: u32 = 0x434653fb; // CFS
const VERSION: u32 = 1;

#[derive(Debug, Default)]
pub struct SectorSchema {
    pub magic: u32,         // 4
    pub version: u32,       // 4
    pub flags: u64,         // 8
    pub cluster_id: u64,    // 8
    pub sector_id: u64,     // 8
    pub disk_size: u64,     // 8
    cluster_size: u64,      // 8
    sector_size: u64,       // 8
    pub local_time: String, // [u8; MAX_STRING_LENGTH]
    pub reversed: String,

    // sector tail
    pub sha256: String, // [u8; MAX_STRING_LENGTH]
}

impl SectorSchema {
    pub fn new() -> Self {
        let now: DateTime<Local> = Local::now();
        let sec = SectorSchema {
            magic: MAGIC,
            version: VERSION,
            flags: 0,
            cluster_id: 0,
            sector_id: 0,
            disk_size: 0,
            cluster_size: 0,
            sector_size: SECTOR_SIZE,
            local_time: now.to_string(),
            ..Default::default()
        };

        sec
    }

    pub fn with_disk_size(mut self, disk_size: u64) -> Self {
        self.disk_size = disk_size;

        self
    }

    pub fn with_cluster_size(mut self, cluster_size: u64) -> Self {
        self.cluster_size = cluster_size;

        self
    }

    pub fn with_cluster_id(mut self, cluster_id: u64) -> Self {
        self.cluster_id = cluster_id;

        self
    }

    pub fn get_sector_size(&self) -> u64 {
        self.sector_size
    }

    fn cacle_hash(&self) -> String {
        let buf_len = (SECTOR_SIZE as usize) - MAX_STRING_LENGTH;

        let mut buf = vec![0; buf_len];

        self.head_to_vec(&mut buf, 0);

        let mut hasher = Sha256::new();
        hasher.update(&buf);

        format!("{:x}", hasher.finalize())
    }

    pub fn update_hash(&mut self) -> &mut Self {
        self.sha256 = self.cacle_hash();
        self
    }

    pub fn update_time(&mut self) -> &mut Self {
        let now: DateTime<Local> = Local::now();
        self.local_time = now.to_string();

        self
    }

    pub fn head_to_vec(&self, buf: &mut Vec<u8>, mut pos: usize) {
        BigEndian::write_u32(&mut buf[pos..], self.magic);
        pos += std::mem::size_of_val(&self.magic);

        BigEndian::write_u32(&mut buf[pos..], self.version);
        pos += std::mem::size_of_val(&self.version);

        BigEndian::write_u64(&mut buf[pos..], self.flags);
        pos += std::mem::size_of_val(&self.flags);

        BigEndian::write_u64(&mut buf[pos..], self.cluster_id);
        pos += std::mem::size_of_val(&self.cluster_id);

        BigEndian::write_u64(&mut buf[pos..], self.sector_id);
        pos += std::mem::size_of_val(&self.sector_id);

        BigEndian::write_u64(&mut buf[pos..], self.disk_size);
        pos += std::mem::size_of_val(&self.disk_size);

        BigEndian::write_u64(&mut buf[pos..], self.cluster_size);
        pos += std::mem::size_of_val(&self.cluster_size);

        BigEndian::write_u64(&mut buf[pos..], self.sector_size);
        pos += std::mem::size_of_val(&self.sector_size);

        buf.write_all_at(pos as u64, self.local_time.as_bytes())
            .unwrap();
    }

    pub fn serialize(&self, buf: &mut Vec<u8>, mut pos: usize) {
        self.head_to_vec(buf, pos);

        pos = pos + (SECTOR_SIZE as usize) - MAX_STRING_LENGTH;
        buf.write_all_at(pos as u64, self.sha256.as_bytes())
            .unwrap();
    }

    pub fn deserialize(&mut self, buf: &Vec<u8>, mut pos: usize) {
        let start_pos = pos;

        self.magic = BigEndian::read_u32(&buf[pos..]);
        pos += std::mem::size_of_val(&self.magic);

        self.version = BigEndian::read_u32(&buf[pos..]);
        pos += std::mem::size_of_val(&self.version);

        self.flags = BigEndian::read_u64(&buf[pos..]);
        pos += std::mem::size_of_val(&self.flags);

        self.cluster_id = BigEndian::read_u64(&buf[pos..]);
        pos += std::mem::size_of_val(&self.cluster_id);

        self.sector_id = BigEndian::read_u64(&buf[pos..]);
        pos += std::mem::size_of_val(&self.sector_id);

        self.disk_size = BigEndian::read_u64(&buf[pos..]);
        pos += std::mem::size_of_val(&self.disk_size);

        self.cluster_size = BigEndian::read_u64(&buf[pos..]);
        pos += std::mem::size_of_val(&self.cluster_size);

        self.sector_size = BigEndian::read_u64(&buf[pos..]);
        pos += std::mem::size_of_val(&self.sector_size);

        let s = &buf[pos..(pos + MAX_STRING_LENGTH)];
        self.local_time = String::from_utf8_lossy(s).to_string();
        pos += MAX_STRING_LENGTH;

        let s = &buf[pos..(start_pos + (SECTOR_SIZE as usize) - MAX_STRING_LENGTH)];
        self.reversed = String::from_utf8_lossy(s).to_string();

        let pos = start_pos + (SECTOR_SIZE as usize) - MAX_STRING_LENGTH;
        let s = &buf[pos..(pos + MAX_STRING_LENGTH)];

        self.sha256 = std::str::from_utf8(s)
            .unwrap_or("None")
            .trim_end_matches('\0')
            .to_string();
    }

    pub fn check(&mut self, buf: &Vec<u8>, pos: usize) -> bool {
        let mut hasher = Sha256::new();
        let sha256_buf = &buf[pos..(pos + SECTOR_SIZE as usize - MAX_STRING_LENGTH)];
        hasher.update(sha256_buf);
        let sha256_string = format!("{:x}", hasher.finalize());

        self.deserialize(buf, pos);

        self.sha256 == sha256_string
    }

    pub fn show_info(&self) {
        println!("{:?}\n", self);
    }
}
