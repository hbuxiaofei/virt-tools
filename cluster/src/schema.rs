use rand::Rng;

use sector::schema::SectorSchema;

#[derive(Debug, Default)]
pub struct ClusterSchema {
    pub buf: Vec<u8>,
    disk_size: u64,
    id: u64,
}

pub const CLUSTER_SIZE: u64 = 512 * 2 * 1024; // 1M

impl ClusterSchema {
    pub fn new() -> Self {
        let mut clu = ClusterSchema {
            buf: Vec::with_capacity(CLUSTER_SIZE as usize),
            disk_size: 0,
            id: 0,
        };

        unsafe {
            clu.buf.set_len(CLUSTER_SIZE as usize);
        }

        clu
    }

    pub fn with_disk_size(mut self, disk_size: u64) -> Self {
        self.disk_size = disk_size;

        self
    }

    pub fn with_id(mut self, id: u64) -> Self {
        self.id = id;

        self
    }

    pub fn set_id(&mut self, id: u64) {
        self.id = id;
    }

    pub fn get_id(&self) -> u64 {
        self.id
    }

    pub fn get_cluster_size(&self) -> u64 {
        CLUSTER_SIZE
    }

    pub fn fill(&mut self) {
        let mut sec = SectorSchema::new()
            .with_disk_size(self.disk_size)
            .with_cluster_size(CLUSTER_SIZE)
            .with_cluster_id(self.id);

        let sector_size = sec.get_sector_size();
        let nr_sector = CLUSTER_SIZE / sector_size;

        for i in 0..nr_sector {
            sec.sector_id = i;
            sec.update_hash();
            sec.serialize(&mut self.buf, (sector_size * i) as usize);
        }
    }

    pub fn check(&self) -> Vec<u64> {
        let mut vec = Vec::new();
        let mut sec = SectorSchema::new();

        let sector_size = sec.get_sector_size();
        let nr_sector = CLUSTER_SIZE / sector_size;

        for i in 0..nr_sector {
            let ok = sec.check(&self.buf, (sector_size * i) as usize);
            if !ok {
                vec.push(i);
            }
        }

        vec
    }

    pub fn inject_error(&mut self) -> bool {
        let mut rng = rand::thread_rng();
        let sec = SectorSchema::new();

        let sector_size = sec.get_sector_size();
        let nr_sector = CLUSTER_SIZE / sector_size;

        let sector_id = rng.gen_range(0..nr_sector);

        let start = sector_id * sector_size;

        let mut v = vec![0, 0];
        v[0] = rng.gen_range(0..sector_size);
        v[1] = rng.gen_range(0..sector_size);
        v.sort();
        println!("start:{:?} {:?}", start, v);

        for i in v[0]..v[1] {
            self.buf[(i + start) as usize] = rng.gen_range(0..256) as u8;
        }

        true
    }
}
