//
// chapter.rs
// Copyright (C) 2022 Woshiluo Luo <woshiluo.luo@outlook.com>
// Distributed under terms of the GNU AGPLv3+ license.
//

use crate::{download_to_file, get_header, ComicError};

#[derive(Debug)]
pub struct Chapter {
    id: u32,
    chapter_id: u32,
    title: String,
    locked: bool,
}

impl Chapter {
    pub fn new(id: u32, chapter_id: u32, title: String, locked: bool) -> Chapter {
        Chapter {
            id,
            chapter_id,
            title,
            locked,
        }
    }
    pub fn get_id(&self) -> u32 {
        self.id
    }
    pub fn get_title(&self) -> &str {
        &self.title
    }
    pub fn get_locked(&self) -> bool {
        self.locked
    }

    pub async fn get_image_list(&self, cookie: &str) -> Result<Vec<String>, ComicError> {
        use serde::{Deserialize, Serialize};

        let params = [("ep_id", self.id)];
        let client = reqwest::Client::new();
        let content = client
        .post("https://manga.bilibili.com/twirp/comic.v1.Comic/GetImageIndex?device=pc&platform=web")
        .form(&params)
        .headers(get_header(cookie, None))
        .send()
        .await
        .map_err(ComicError::to_get_list)?
        .text()
        .await
        .map_err(ComicError::to_get_list)?;

        #[derive(Serialize, Deserialize)]
        struct RawImage {
            path: String,
        }
        #[derive(Serialize, Deserialize)]
        struct RawData {
            images: Vec<RawImage>,
        }
        #[derive(Serialize, Deserialize)]
        struct RawContent {
            data: RawData,
        }

        let content: RawContent =
            serde_json::from_str(&content).map_err(ComicError::to_get_list)?;
        let images = content
            .data
            .images
            .into_iter()
            .map(|cur| cur.path.clone())
            .collect::<Vec<String>>();

        {
            #[derive(Serialize, Deserialize)]
            struct RawImage {
                url: String,
                token: String,
            }
            #[derive(Serialize, Deserialize)]
            struct RawContent {
                data: Vec<RawImage>,
            }

            let params = [("urls", serde_json::to_string(&images).unwrap())];
            let content = client
            .post(
                "https://manga.bilibili.com/twirp/comic.v1.Comic/ImageToken?device=pc&platform=web",
            )
            .headers(get_header(cookie, None))
            .form(&params)
            .send()
            .await
            .map_err(ComicError::to_get_list)?
            .text()
            .await
            .map_err(ComicError::to_get_list)?;

            let content: RawContent = serde_json::from_str(&content).unwrap();
            let images = content.data;

            let images = images
                .into_iter()
                .map(|image| format!("{}?token={}", image.url, image.token))
                .collect::<Vec<String>>();

            Ok(images)
        }
    }

    pub async fn download<T>(
        &self,
        cookie: &str,
        progressbar: &indicatif::MultiProgress,
        base_path: T,
    ) -> Result<(), ComicError>
    where
        T: AsRef<std::path::Path>,
    {
        let sty = indicatif::ProgressStyle::default_bar()
            .template("[{elapsed_precise}] {bar:40.cyan/blue} {pos:>7}/{len:7} {msg}")
            .progress_chars("##-");
        let base_path = base_path.as_ref();

        let images = self.get_image_list(cookie).await?;

        let pb = progressbar.add(indicatif::ProgressBar::new(images.len() as u64));
        pb.set_style(sty);

        for i in 0..images.len() {
            pb.set_message(format!("item #{}", i + 1));

            download_to_file(
                &images[i],
                base_path
                    .join(format!("{}-{}", self.chapter_id, self.title))
                    .join(format!("{}-{}.png", self.id, i + 1)),
            )
            .await?;

            pb.inc(1);
        }

        progressbar.remove(&pb);

        Ok(())
    }
}

#[cfg(test)]
mod tests {
    //    use super::*;

    #[test]
    fn it_works() {}
}
