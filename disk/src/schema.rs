use block::device::BlockDevice;
use cluster::schema::ClusterSchema;
use sector::schema::SectorSchema;

pub struct DiskSchema {
    path: String,
}

impl DiskSchema {
    pub fn new(path: &str) -> Self {
        DiskSchema {
            path: path.to_string(),
        }
    }

    fn show_sector(&self, cluster_id: u64, sector_id: u64) {
        let blk = BlockDevice::new(self.path.as_str()).unwrap();
        let disk_size = blk.get_disk_size();

        let mut clu = ClusterSchema::new().with_disk_size(disk_size);
        let cluster_size = clu.get_cluster_size();

        let nr_cluster = disk_size / cluster_size;

        if cluster_id >= nr_cluster {
            return;
        }

        blk.read_direct_at(&mut clu.buf, cluster_id * cluster_size);

        let mut sec = SectorSchema::new();
        sec.deserialize(&clu.buf, (sector_id * sec.get_sector_size()) as usize);
        sec.show_info();
    }

    pub fn check_disk(&self, cluster_id: u64) {
        let blk = BlockDevice::new(self.path.as_str()).unwrap();
        let disk_size = blk.get_disk_size();

        let mut clu = ClusterSchema::new().with_disk_size(disk_size);
        let cluster_size = clu.get_cluster_size();

        let nr_cluster = disk_size / cluster_size;

        if cluster_id >= nr_cluster {
            return;
        }

        blk.read_direct_at(&mut clu.buf, cluster_id * cluster_size);
        let err_sectors = clu.check();
        if !err_sectors.is_empty() {
            println!("\n>>> check error: {:?} - {:?}", cluster_id, err_sectors);
            for i in 0..err_sectors.len() {
                self.show_sector(cluster_id, err_sectors[i]);
            }
        }
    }

    pub fn check_whole_disk(&self) {
        let blk = BlockDevice::new(self.path.as_str()).unwrap();
        let disk_size = blk.get_disk_size();

        let mut clu = ClusterSchema::new().with_disk_size(disk_size);
        let cluster_size = clu.get_cluster_size();

        let nr_cluster = disk_size / cluster_size;

        for i in 0..nr_cluster {
            blk.read_direct_at(&mut clu.buf, i * cluster_size);
            let err_sectors = clu.check();
            if !err_sectors.is_empty() {
                println!("\n>>> check error: {:?} - {:?}", i, err_sectors);
                for j in 0..err_sectors.len() {
                    self.show_sector(i, err_sectors[j]);
                }
            }
        }
    }

    pub fn fill_disk(&self, cluster_id: u64) {
        let blk = BlockDevice::new(self.path.as_str()).unwrap();
        let disk_size = blk.get_disk_size();

        let mut clu = ClusterSchema::new().with_disk_size(disk_size);
        let cluster_size = clu.get_cluster_size();

        let nr_cluster = disk_size / cluster_size;
        if cluster_id >= nr_cluster {
            return;
        }

        clu.set_id(cluster_id);
        clu.fill();
        blk.write_direct_at(&clu.buf, cluster_id * cluster_size);
    }

    pub fn fill_whole_disk(&self) {
        let blk = BlockDevice::new(self.path.as_str()).unwrap();
        let disk_size = blk.get_disk_size();

        let mut clu = ClusterSchema::new().with_disk_size(disk_size);
        let cluster_size = clu.get_cluster_size();

        let nr_cluster = disk_size / cluster_size;

        for i in 0..nr_cluster {
            clu.set_id(i);
            clu.fill();
            blk.write_direct_at(&clu.buf, i * cluster_size);
        }
    }

    pub fn inject_cluster_error(&self, cluster_id: u64) -> bool {
        let blk = BlockDevice::new(self.path.as_str()).unwrap();
        let disk_size = blk.get_disk_size();

        let mut clu = ClusterSchema::new().with_disk_size(disk_size);
        let cluster_size = clu.get_cluster_size();

        let nr_cluster = disk_size / cluster_size;

        if cluster_id >= nr_cluster {
            return false;
        }

        blk.read_direct_at(&mut clu.buf, cluster_id * cluster_size);

        let inject_ok = clu.inject_error();

        blk.write_direct_at(&clu.buf, cluster_id * cluster_size);

        inject_ok
    }
}
