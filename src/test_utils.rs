use std::fmt::Display;
use std::fs::File;
use std::io::Read;
use std::path::PathBuf;

pub enum Version {
    V1_0,
}

impl Display for Version {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Version::V1_0 => f.write_str("v1_0"),
        }
    }
}

fn sample_dir() -> PathBuf {
    let mut pb = PathBuf::from(env!("CARGO_MANIFEST_DIR"));
    pb.push("samples");
    pb
}

pub fn open_sample(version: Version, stem: &str) -> File {
    let mut p = sample_dir();
    p.push(version.to_string());
    p.push(format!("{stem}.canvas"));
    File::open(p).unwrap()
}

pub fn read_sample(version: Version, stem: &str) -> String {
    let mut f = open_sample(version, stem);
    let len = f.metadata().unwrap().len();
    let mut s = String::with_capacity(len.try_into().unwrap());
    f.read_to_string(&mut s).unwrap();
    s
}
