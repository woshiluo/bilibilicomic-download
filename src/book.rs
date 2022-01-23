//
// book.rs
// Copyright (C) 2022 Woshiluo Luo <woshiluo.luo@outlook.com>
// Distributed under terms of the GNU AGPLv3+ license.
//

use crate::{get_header, ComicError};

use crate::chapter::Chapter;
use serde_json::Value;

#[derive(Debug)]
pub struct Book {
    id: u32,
    name: String,
    chapter_list: Vec<Chapter>,
}

impl Book {
    pub fn id(&self) -> u32 {
        self.id
    }
    pub fn name(&self) -> &String {
        &self.name
    }
    pub fn get_total(&self) -> u32 {
        self.chapter_list.len() as u32
    }
    pub fn get_chapter(&self, id: u32) -> &Chapter {
        &self.chapter_list[id as usize]
    }
}

pub async fn get_book(cookie: &str, id: u32) -> Result<Book, ComicError> {
    let client = reqwest::Client::new();
    let params = [("comic_id", id)];
    let content = client
        .post("https://manga.bilibili.com/twirp/comic.v1.Comic/ComicDetail?device=pc&platform=web")
        .form(&params)
        .headers(get_header(cookie, None))
        .send()
        .await
        .map_err(ComicError::to_get_book)?
        .text()
        .await
        .map_err(ComicError::to_get_book)?;

    let content: Value = serde_json::from_str(&content).map_err(ComicError::to_get_book)?;
    let mut chapter_list: Vec<Chapter> = vec![];

    let total = content["data"]["total"].to_string().parse::<u32>().unwrap();
    let ep_list = &content["data"]["ep_list"];

    for i in 0..total {
        let ep = &ep_list[i as usize];
        chapter_list.push(Chapter::new(
            ep["id"].to_string().parse::<u32>().unwrap(),
            ep["ord"].to_string(),
            ep["title"].to_string(),
            ep["is_locked"].to_string().parse::<bool>().unwrap(),
        ));
    }
    chapter_list.reverse();

    Ok(Book {
        id,
        name: content["data"]["title"].to_string(),
        chapter_list,
    })
}

#[cfg(test)]
mod tests {
    //    use super::*;

    #[test]
    fn it_works() {}
}
