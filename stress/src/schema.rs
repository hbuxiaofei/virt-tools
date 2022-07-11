use rand::Rng;
use std::collections::HashMap;
use std::fs::File;
use std::io::{BufRead, BufReader};
use std::sync::{Arc, RwLock};
use std::{thread, time};

#[derive(PartialEq, Copy, Clone)]
enum Command {
    Pause,
    Start,
    Kill,
}

#[derive(PartialEq, Copy, Clone)]
enum State {
    Paused,
    Running,
    Killed,
}

pub struct StressSchema {
    cmd: Arc<RwLock<Command>>,
    stat: Arc<RwLock<State>>,
}

impl Default for Command {
    fn default() -> Self {
        Command::Pause
    }
}

impl Default for State {
    fn default() -> Self {
        State::Killed
    }
}

impl StressSchema {
    pub fn new() -> Self {
        StressSchema {
            cmd: Arc::new(RwLock::new(Command::Pause)),
            stat: Arc::new(RwLock::new(State::Killed)),
        }
    }

    pub fn start(&mut self) {
        println!("Receive start");
        let cmd = Arc::clone(&self.cmd);
        let mut cmd = cmd.write().unwrap();
        *cmd = Command::Start;

        let stat = Arc::clone(&self.stat);
        let mut stat = stat.write().unwrap();
        *stat = State::Running;
    }

    pub fn pause(&mut self) {
        println!("Receive pause");
        let cmd = Arc::clone(&self.cmd);
        let mut cmd = cmd.write().unwrap();
        *cmd = Command::Pause;

        let stat = Arc::clone(&self.stat);
        let mut stat = stat.write().unwrap();
        *stat = State::Paused;
    }

    pub fn kill(&mut self) {
        println!("Receive kill");
        let cmd = Arc::clone(&self.cmd);
        let mut cmd = cmd.write().unwrap();
        *cmd = Command::Kill;

        let stat = Arc::clone(&self.stat);
        let mut stat = stat.write().unwrap();
        *stat = State::Killed;

        // for hander in self.handlers.into_iter() {
        // hander.join().unwrap();
        // }
    }

    pub fn create(&mut self) {
        self.pause();

        let nr_threads: usize = get_num_cpus();
        println!("Using {} threads for cpu stress", nr_threads);

        let mut handlers = vec![];
        for _x in 0..nr_threads {
            let cmd = Arc::clone(&self.cmd);

            let handler = thread::spawn(move || loop {
                let mut cur_cmd = Command::Pause;
                if let Ok(cmd) = cmd.read() {
                    cur_cmd = *cmd;
                }

                if cur_cmd == Command::Start {
                    worker();
                } else if cur_cmd == Command::Pause {
                    thread::sleep(time::Duration::from_micros(1));
                } else if cur_cmd == Command::Kill {
                    break;
                }
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

fn worker() {
    let mut rng = rand::thread_rng();
    let num: f64 = rng.gen_range(0..(u64::MAX)) as f64;
    let _x = sqrt(num);
    // let ret = format!("{:?}", x);
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
