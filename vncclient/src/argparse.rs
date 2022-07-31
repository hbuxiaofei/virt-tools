/*
 *   This file is part of NCC Group Scrying https://github.com/nccgroup/scrying
 *   Copyright 2020 David Young <david(dot)young(at)nccgroup(dot)com>
 *   Released as open source by NCC Group Plc - https://www.nccgroup.com
 *
 *   Scrying is free software: you can redistribute it and/or modify
 *   it under the terms of the GNU General Public License as published by
 *   the Free Software Foundation, either version 3 of the License, or
 *   (at your option) any later version.
 *
 *   Scrying is distributed in the hope that it will be useful,
 *   but WITHOUT ANY WARRANTY; without even the implied warranty of
 *   MERCHANTABILITY or FITNESS FOR A PARTICULAR PURPOSE.  See the
 *   GNU General Public License for more details.
 *
 *   You should have received a copy of the GNU General Public License
 *   along with Scrying.  If not, see <https://www.gnu.org/licenses/>.
*/

use clap::{App, AppSettings, Arg, ArgGroup};
use std::str::FromStr;

#[derive(Copy, Clone, PartialEq, Debug)]
pub enum Mode {
    Auto,
    Web,
    Rdp,
    Vnc,
}

impl Mode {
    /// Determine whether the supplied mode filter is valid for the
    /// current mode. Combinations are:
    /// Mode::Auto -> all filters valid
    /// Mode::X -> only X and auto are valid
    pub fn selected(&self, filter: Self) -> bool {
        use Mode::*;
        self == &Auto || self == &filter || filter == Auto
    }
}

impl Default for Mode {
    fn default() -> Self {
        Mode::Auto
    }
}

impl FromStr for Mode {
    type Err = &'static str;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        use Mode::{Auto, Rdp, Vnc, Web};
        match s {
            "web" => Ok(Web),
            "rdp" => Ok(Rdp),
            "vnc" => Ok(Vnc),
            "auto" => Ok(Auto),
            _ => Err("Mode must be \"auto\", \"web\" or \"rdp\""),
        }
    }
}

#[derive(Debug, Default)]
pub struct Opts {
    pub files: Vec<String>,
    pub targets: Vec<String>,
    pub mode: Mode,
    pub rdp_timeout: usize,
    pub threads: usize,
    pub log_file: Option<String>,
    pub nmaps: Vec<String>,
    pub nessus: Vec<String>,
    pub output_dir: String,
    pub web_proxy: Option<String>,
    pub rdp_proxy: Option<String>,
    pub silent: bool,
    pub verbose: u64,
    pub test_import: bool,
}

pub fn parse() -> Result<Opts, Box<dyn std::error::Error>> {
    let args = App::new("Scrying")
        .author("David Young https://github.com/nccgroup/scrying")
        .setting(AppSettings::ArgRequiredElseHelp)
        .arg(
            Arg::new("FILE")
                .long("file")
                .multiple(true)
                .short('f')
                .takes_value(true),
        )
        .arg(
            Arg::new("TARGET")
                .long("target")
                .multiple(true)
                .short('t')
                .takes_value(true),
        )
        .arg(
            Arg::new("MODE")
                .default_value("auto")
                .long("mode")
                .possible_values(&["web", "rdp", "vnc", "auto"])
                .short('m')
                .takes_value(true),
        )
        .arg(
            Arg::new("RDP TIMEOUT")
                .default_value("2")
                .long("rdp-timeout")
                .takes_value(true)
                .takes_value(true),
        )
        .arg(
            Arg::new("THREADS")
                .default_value("10")
                .long("threads")
                .takes_value(true),
        )
        .arg(
            Arg::new("LOG FILE")
                .long("log-file")
                .short('l')
                .takes_value(true),
        )
        .arg(
            Arg::new("NMAP XML FILE")
                .long("nmap")
                .multiple(true)
                .takes_value(true),
        )
        .arg(
            Arg::new("NESSUS XML FILE")
                .long("nessus")
                .multiple(true)
                .takes_value(true),
        )
        .arg(
            Arg::new("OUTPUT DIR")
                .default_value("output")
                .long("output")
                .short('o')
                .takes_value(true),
        )
        .arg(
            Arg::new("WEB PROXY")
                .long("web-proxy")
                .takes_value(true),
        )
        .arg(
            Arg::new("RDP PROXY")
                .long("rdp-proxy")
                .takes_value(true),
        )
        .arg(
            Arg::new("PROXY")
                .long("proxy")
                .takes_value(true)
                .validator(is_socks5),
        )
        .arg(
            Arg::new("SILENT")
                .long("silent")
                .short('s'),
        )
        .arg(
            Arg::new("VERBOSE")
                .long("verbose")
                .multiple(true)
                .short('v')
                .takes_value(false),
        )
        .arg(
            Arg::new("TEST IMPORT")
                .long("test-import"),
        )
        .group(
            ArgGroup::new("inputs")
                .multiple(true)
                .required(true)
                .args(&["FILE", "NMAP XML FILE", "NESSUS XML FILE", "TARGET"]),
        )
        .get_matches();

    // Grab input files if present, otherwise an empty Vec
    let mut files: Vec<String> = Vec::new();
    if let Some(f) = args.values_of("FILE") {
        for file in f {
            files.push(file.to_string());
        }
    }

    // Grab targets if present, otherwise an empty Vec
    let mut targets: Vec<String> = Vec::new();
    if let Some(t) = args.values_of("TARGET") {
        for target in t {
            targets.push(target.to_string());
        }
    }

    // Grab Nmap files if present, otherwise an empty Vec
    let mut nmaps: Vec<String> = Vec::new();
    if let Some(n) = args.values_of("NMAP XML FILE") {
        for nmap in n {
            nmaps.push(nmap.to_string());
        }
    }

    // Grab Nessus files if present, otherwise an empty Vec
    let mut nessus: Vec<String> = Vec::new();
    if let Some(n) = args.values_of("NESSUS XML FILE") {
        for nessus_file in n {
            nessus.push(nessus_file.to_string());
        }
    }

    // If global proxy setting is configured then set all indivitual
    // proxy values to it. Then override each one in turn if applicable
    let mut web_proxy = None;
    let mut rdp_proxy = None;
    if let Some(p) = args.value_of("PROXY") {
        web_proxy = Some(p.to_string());
        rdp_proxy = Some(p.to_string());
    }

    if let Some(p) = args.value_of("RDP PROXY") {
        rdp_proxy = Some(p.to_string());
    }

    if let Some(p) = args.value_of("WEB PROXY") {
        web_proxy = Some(p.to_string());
    }

    Ok(Opts {
        files,
        targets,
        mode: args.value_of_t("MODE").unwrap(),
        rdp_timeout: args.value_of_t("RDP TIMEOUT").unwrap(),
        threads: args.value_of_t("THREADS").unwrap(),
        log_file: args
            .value_of("LOG FILE")
            .map_or_else(|| None, |s| Some(s.to_string())),
        nmaps,
        nessus,
        output_dir: args.value_of_t("OUTPUT DIR").unwrap(),
        web_proxy,
        rdp_proxy,
        silent: args.is_present("SILENT"),
        verbose: args.occurrences_of("VERBOSE"),
        test_import: args.is_present("TEST IMPORT"),
    })
}

fn is_socks5(val: &str) -> Result<(), String> {
    if !val.starts_with("socks5://") {
        Err("Global or RDP proxy must be a socks5:// URI".to_string())
    } else {
        Ok(())
    }
}

#[cfg(test)]
mod test {
    #[test]
    fn mode_filter() {
        use super::Mode::*;

        let auto = Auto;
        let rdp = Rdp;
        let web = Web;

        assert!(auto.selected(Auto));
        assert!(auto.selected(Rdp));
        assert!(auto.selected(Web));

        assert!(rdp.selected(Auto));
        assert!(rdp.selected(Rdp));
        assert!(!rdp.selected(Web));

        assert!(web.selected(Auto));
        assert!(!web.selected(Rdp));
        assert!(web.selected(Web));
    }
}
