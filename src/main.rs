use clap::Parser;
use std::io::Read;

/// Simple program to greet a person
#[derive(Parser, Debug)]
#[clap(author, version, about, long_about = None)]
struct Args {
    #[clap(long)]
    cookie_file: String,

    #[clap(long)]
    book_id: u32,

    #[clap(long)]
    dir: String,

    #[clap(long)]
    from: u32,

    #[clap(long)]
    end: Option<u32>,
}

#[tokio::main]
async fn main() {
    use indicatif::{MultiProgress, ProgressBar, ProgressStyle};
    let args = Args::parse();

    let m = MultiProgress::new();
    let sty = ProgressStyle::default_bar()
        .template("[{elapsed_precise}] {bar:40.cyan/blue} {pos:>7}/{len:7} {msg}")
        .progress_chars("##-");

    let mut cookie = String::new();

    std::fs::File::open(args.cookie_file)
        .unwrap()
        .read_to_string(&mut cookie)
        .unwrap();

    cookie.remove(cookie.len() - 1);

    let book = bilibilicomic_download::book::get_book(&cookie, args.book_id)
        .await
        .unwrap();

    let from = args.from;
    let end = match args.end {
        Some(val) => val,
        None => book.get_total() - 1,
    };

    let pb = m.add(ProgressBar::new(book.get_total() as u64));
    pb.set_style(sty.clone());

    for i in from..=end {
        pb.set_message(format!("{}-{}", i + 1, book.get_chapter(i).get_title()));
        pb.inc(1);

        book.get_chapter(i)
            .download(&cookie, &m, &args.dir)
            .await
            .unwrap();
    }

    m.remove(&pb);
    let pb = m.add(ProgressBar::new(book.get_total() as u64));
    pb.set_style(sty);

    let mut handles = Vec::new();
    for file in std::fs::read_dir(&args.dir).unwrap() {
        let file = file.unwrap().path();
        let pb = pb.clone();

        handles.push(tokio::spawn(async move {
            bilibilicomic_download::archive_to_file(&file, format!("{}.cbz", file.display()))
                .await
                .unwrap();
            pb.inc(1);
        }));
    }

    futures::future::join_all(handles).await;

    m.clear().unwrap();
}
