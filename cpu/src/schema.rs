use ndarray;
use rand::Rng;
use std::collections::HashMap;
use std::fs::File;
use std::io::{BufRead, BufReader, Write};
use std::sync::{Arc, Mutex};
use std::{thread, time};

#[derive(PartialEq, Copy, Clone)]
enum Command {
    Stop,
    Start,
    Kill,
}

#[derive(PartialEq, Copy, Clone)]
enum State {
    Stoped,
    Running,
}

pub struct CpuSchema {
    cmd: Arc<Mutex<Command>>,
    stat: Arc<Mutex<State>>,
}

impl Default for Command {
    fn default() -> Self {
        Command::Stop
    }
}

impl Default for State {
    fn default() -> Self {
        State::Stoped
    }
}

impl CpuSchema {
    pub fn new() -> Self {
        CpuSchema {
            cmd: Arc::new(Mutex::new(Command::Stop)),
            stat: Arc::new(Mutex::new(State::Stoped)),
        }
    }

    pub fn start(&mut self) {
        println!("Receive start");
        let cmd = Arc::clone(&self.cmd);
        let mut cmd = cmd.lock().unwrap();
        *cmd = Command::Start;
    }

    pub fn stop(&mut self) {
        println!("Receive stop");
        let cmd = Arc::clone(&self.cmd);
        let mut cmd = cmd.lock().unwrap();
        *cmd = Command::Stop;
    }

    pub fn kill(&mut self) {
        println!("Receive kill");
        let cmd = Arc::clone(&self.cmd);
        let mut cmd = cmd.lock().unwrap();
        *cmd = Command::Kill;

        // for hander in self.handlers.into_iter() {
        // hander.join().unwrap();
        // }
    }

    pub fn create(&mut self) {
        let nr_threads: usize = get_num_cpus();
        println!("Using {} threads for cpu stress", nr_threads);

        let mut handlers = vec![];
        for i in 0..nr_threads {
            let thread_num = Arc::new(Mutex::new(i));

            let cmd = Arc::clone(&self.cmd);
            let stat = Arc::clone(&self.stat);

            let handler = thread::spawn(move || loop {
                if let (Ok(mut cmd), Ok(mut stat)) = (cmd.lock(), stat.lock()) {
                    if !worker(&mut cmd, &mut stat) {
                        break;
                    }
                }
                // thread::sleep(time::Duration::from_micros(1));
            });

            handlers.push(handler);
        }

        thread::spawn(|| {
            for hander in handlers {
                hander.join().unwrap();
            }
            println!("All thread exit");
        });

        println!("Spawning thread over");
    }
}

fn worker(cmd: &mut Command, stat: &mut State) -> bool {
    if *cmd == Command::Start {
        let mut rng = rand::thread_rng();
        let num: f64 = rng.gen_range(0..(u64::MAX)) as f64;
        let ret = format!("{:?}", sqrt(num));

        // println!("ret: {:?}", ret);

        // const length: usize = 3;

        // let mut arry1 = [[0; length]; length];
        // let mut arry2 = [[0; length]; length];

        // let mut rng = rand::thread_rng();

        // for i in 0..length {
        // for j in 0..length {
        // let num: u64 = rng.gen_range(0..(u8::MAX as u8)) as u64;
        // arry1[i][j] = num;
        // let num: u64 = rng.gen_range(0..(u8::MAX as u8)) as u64;
        // arry2[i][j] = num;
        // }
        // }

        // let a = ndarray::arr2(&arry1);
        // let b = ndarray::arr2(&arry2);

        // let ret = a.dot(&b);
        // println!("ret: {:?}", ret.into_raw_vec());

        *stat = State::Running;
    } else if *cmd == Command::Stop {
        thread::sleep(time::Duration::from_micros(1));
        *stat = State::Stoped;
    } else if *cmd == Command::Kill {
        return false;
    }

    true
}

fn sqrt(x: f64) -> f64 {
    let mut low: f64 = 0.0;
    let mut up: f64 = x;
    let mut mid: f64 = x / 2.0;
    let mut result: f64 = mid * mid;

    loop {
        // println!("low = {}, up = {}, mid = {}", low, up, mid);

        if low == up || mid == low || mid == up {
            break;
        }

        if result > x {
            up = mid;
            mid = (low + up) / 2.0;
            result = mid * mid;
        } else if result < x {
            low = mid;
            mid = (low + up) / 2.0;
            result = mid * mid;
        } else {
            break;
        }
    }

    mid
}

fn get_num_cpus() -> usize {
    let file = match File::open("/proc/cpuinfo") {
        Ok(val) => val,
        Err(_) => return 0,
    };
    let reader = BufReader::new(file);
    let mut map = HashMap::new();
    let mut physid: u32 = 0;
    let mut cores: usize = 0;
    let mut chgcount = 0;
    for line in reader.lines().filter_map(|result| result.ok()) {
        let mut it = line.split(':');
        let (key, value) = match (it.next(), it.next()) {
            (Some(key), Some(value)) => (key.trim(), value.trim()),
            _ => continue,
        };
        if key == "physical id" {
            match value.parse() {
                Ok(val) => physid = val,
                Err(_) => break,
            };
            chgcount += 1;
        }
        if key == "cpu cores" {
            match value.parse() {
                Ok(val) => cores = val,
                Err(_) => break,
            };
            chgcount += 1;
        }
        if chgcount == 2 {
            map.insert(physid, cores);
            chgcount = 0;
        }
    }
    let count = map.into_iter().fold(0, |acc, (_, cores)| acc + cores);

    count
}
