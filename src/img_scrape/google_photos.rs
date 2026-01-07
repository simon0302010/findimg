use regex::Regex;
use reqwest::blocking::Client;
use std::path::PathBuf;
use std::{
    collections::HashSet,
    fs::File,
    io::Write,
    path::Path,
};

pub fn scrape(path: PathBuf, album_url: &str) -> Result<(), Box<dyn std::error::Error>> {

    let client = Client::builder().user_agent("Mozilla/5.0").build()?;

    println!("Fetching album page...");
    let html = client.get(album_url).send()?.text()?;

    let image_urls = extract_image_urls(&html);

    println!("Found {} images", image_urls.len());

    for (i, url) in image_urls.iter().enumerate() {
        let filename = format!("{}/img_{:04}.jpg", path.display(), i);
        download_image(&client, url, &filename)?;
        println!("Downloaded {}", filename);
    }

    Ok(())
}

fn extract_image_urls(html: &str) -> Vec<String> {
    let re = Regex::new(r"https://lh3\.googleusercontent\.com/[a-zA-Z0-9_\-=/]+").unwrap();

    let mut urls = HashSet::new();

    for m in re.find_iter(html) {
        let mut url = m.as_str().to_string();

        // Remove size params (=w2048-h2048 etc)
        if let Some(idx) = url.find('=') {
            url.truncate(idx);
        }

        urls.insert(url);
    }

    urls.into_iter().collect()
}

fn download_image(
    client: &Client,
    url: &str,
    path: &str,
) -> Result<(), Box<dyn std::error::Error>> {
    let resp = client.get(url).send()?;
    let bytes = resp.bytes()?;

    let mut file = File::create(Path::new(path))?;
    file.write_all(&bytes)?;

    Ok(())
}
