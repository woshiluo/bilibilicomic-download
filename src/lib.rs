//
// mod.rs
// Copyright (C) 2022 Woshiluo Luo <woshiluo.luo@outlook.com>
// Distributed under terms of the GNU AGPLv3+ license.
//

extern crate reqwest;

pub mod book;
pub mod chapter;

#[derive(Debug)]
pub enum ComicError {
    DownloadFailed(String),
    GetListFailed(String),
    GetBookFailed(String),
    ArchiveFailed(String),
}

impl ComicError {
    pub fn to_download(err: impl std::error::Error) -> ComicError {
        ComicError::DownloadFailed(err.to_string())
    }
    pub fn to_get_list(err: impl std::error::Error) -> ComicError {
        ComicError::GetListFailed(err.to_string())
    }
    pub fn to_get_book(err: impl std::error::Error) -> ComicError {
        ComicError::GetBookFailed(err.to_string())
    }
    pub fn to_archive(err: impl std::error::Error) -> ComicError {
        ComicError::ArchiveFailed(err.to_string())
    }
}

pub fn get_header(cookie: &str, refer: Option<&str>) -> reqwest::header::HeaderMap {
    use reqwest::header;
    use reqwest::header::HeaderMap;

    let refer = match refer {
        Some(refer) => refer,
        None => "https://manga.bilibili.com",
    };
    let mut headers = HeaderMap::new();
    headers.insert(header::REFERER, refer.parse().unwrap());
    headers.insert(header::COOKIE, cookie.parse().unwrap());

    headers
}

pub async fn download_to_file<T>(url: &str, output: T) -> Result<(), ComicError>
where
    T: AsRef<std::path::Path>,
{
    let path = output.as_ref();
    std::fs::create_dir_all(path.parent().unwrap()).map_err(ComicError::to_download)?;
    let resp = reqwest::get(url)
        .await
        .map_err(ComicError::to_download)?
        .bytes()
        .await
        .map_err(ComicError::to_download)?;
    let mut resp = resp.as_ref();
    let mut out = std::fs::File::create(path).map_err(ComicError::to_download)?;

    std::io::copy(&mut resp, &mut out).map_err(ComicError::to_download)?;

    Ok(())
}

pub async fn archive_to_file<T, P>(src: T, dst: P) -> Result<(), ComicError>
where
    T: AsRef<std::path::Path>,
    P: AsRef<std::path::Path>,
{
    use std::io::{Read, Write};
    let src = src.as_ref();
    let dst = dst.as_ref();

    let file = std::fs::File::create(dst).map_err(ComicError::to_archive)?;
    let mut zip = zip::ZipWriter::new(file);

    let image_list = std::fs::read_dir(src).map_err(ComicError::to_archive)?;

    let mut buffer = Vec::new();
    for image in image_list {
        let path = image.map_err(ComicError::to_archive)?.path();
        zip.start_file(path.to_str().unwrap(), zip::write::FileOptions::default())
            .map_err(ComicError::to_archive)?;
        let mut f = std::fs::File::open(path).map_err(ComicError::to_archive)?;

        f.read_to_end(&mut buffer).map_err(ComicError::to_archive)?;
        zip.write_all(&*buffer).map_err(ComicError::to_archive)?;
        buffer.clear();
    }

    zip.finish().map_err(ComicError::to_archive)?;

    Ok(())
}

#[cfg(test)]
mod tests {
    //use super::*;

    #[test]
    fn it_works() {}
}
