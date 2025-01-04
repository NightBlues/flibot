use std::fs::{File, DirBuilder};
use std::io::{Read, Write};
use bytes::Bytes;
use anyhow::Result;
use sha1::{Sha1, Digest};

pub fn filepath(sha1: String) -> String {
    format!("files/{}/{}", &sha1[0..2], sha1)
}

pub fn create_basedir(sha1: String) -> Result<()> {
    let path = format!("files/{}", &sha1[0..2]);
    log::info!("DBG: create_basedir {}", &path);
    let () = DirBuilder::new()
        .recursive(true)
        .create(path)?;
    Ok(())
}

pub fn fb2_sha1(data: Bytes) -> Result<String> {
    let mut hasher = Sha1::new();
    hasher.update(data);
    let result = hasher.finalize();
    // let result : Vec<u8> = result[..].into();
    // let result = String::from_utf8(result)?;
    Ok(format!("{result:x}"))
}

/// Get file from disk
pub fn get_file(sha1: String) -> Result<Bytes> {
    let filename = filepath(sha1);
    let mut fp = File::open(filename)?;
    let mut res = Vec::new();
    fp.read_to_end(&mut res)?;
    let res : Bytes = res.into();
    
    Ok(res)
}

/// Put file to disk
pub fn put_file(sha1: String, data: Bytes) -> Result<()> {
    create_basedir(sha1.clone())?;
    let filename = filepath(sha1);
    log::info!("DBG: open file {}", &filename);
    let mut fp = File::options()
        .write(true)
        .truncate(true)
        .create(true)
        .open(filename)?;
    let data : Vec<u8> = (*data).into();
    fp.write_all(&data)?;

    Ok(())
}
