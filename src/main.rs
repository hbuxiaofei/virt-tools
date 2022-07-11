use clap::{App, Arg, SubCommand};
use std::{thread, time};

use block::device::BlockDevice;
use cpu::schema::CpuSchema;
use disk::schema::DiskSchema;
use sector::schema::SectorSchema;

fn main() {
    let mut cpu = CpuSchema::new();
    cpu.create();
    thread::sleep(time::Duration::from_secs(2));
    cpu.start();
    thread::sleep(time::Duration::from_secs(30));
    // cpu.stop();
    cpu.kill();

    let dev_path = "/dev/nbd0";
    let disk = DiskSchema::new(dev_path);

    let matches = App::new("KVM virtualization test tools.")
        .subcommand(
            SubCommand::with_name("disk-write").arg(
                Arg::with_name("debug")
                    .short('d')
                    .help("print debug information verbosely"),
            ),
        )
        .subcommand(
            SubCommand::with_name("disk-check").arg(
                Arg::with_name("debug")
                    .short('d')
                    .help("print debug information verbosely"),
            ),
        )
        .subcommand(
            SubCommand::with_name("disk-inject-fault").arg(
                Arg::with_name("debug")
                    .short('d')
                    .help("print debug information verbosely"),
            ),
        )
        .get_matches();

    if let Some(matches) = matches.subcommand_matches("disk-write") {
        if matches.is_present("debug") {
            println!("Printing debug info...");
        } else {
            println!("Printing normally...");
        }

        disk.fill_whole_disk();
    } else if let Some(matches) = matches.subcommand_matches("disk-check") {
        if matches.is_present("debug") {
            println!("Printing debug info...");
        } else {
            println!("Printing normally...");
        }

        disk.check_whole_disk();
    } else if let Some(matches) = matches.subcommand_matches("disk-inject-fault") {
        if matches.is_present("debug") {
            println!("Printing debug info...");
        } else {
            println!("Printing normally...");
        }

        let inject_ok = disk.inject_cluster_error(0);
        println!("\n>>> inject:{:?}", inject_ok);
    }
}

#[allow(dead_code)]
fn main_bak() {
    // fill_whole_disk();

    let dev_path = "/dev/nbd0";
    let blk = BlockDevice::new(dev_path).unwrap();

    let mut buf = vec![0; 4096];
    let read_len = blk.read_direct_at(&mut buf, 512 * 512);
    println!("\n>>> Read len: {}, buf len: {}", read_len, buf.len());

    let mut sec = SectorSchema::new();

    println!("\n>>> The text({}):\n {:?}", read_len, &buf[0..512]);
    sec.deserialize(&buf, 0);
    println!(">>> Check result: {}", sec.check(&buf, 0));
    sec.show_info();

    println!("\n>>> The text({}):\n {:?}", read_len, &buf[512..1024]);
    println!(">>> Check result: {}", sec.check(&buf, 512));
    sec.deserialize(&buf, 512);
    sec.show_info();
}
