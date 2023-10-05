use std::{
    fs,
    io::{self, stdin, stdout},
    path::{Path, PathBuf},
};

use scraper::Html;
use serde::{Serialize, Serializer};

use om_wikiparser::html::{self, HtmlError};

#[derive(Debug, Serialize)]
struct Stats {
    file: PathBuf,
    original_size: usize,
    processed_size: Option<usize>,
    #[serde(serialize_with = "write_error")]
    error: Option<HtmlError>,
    lang: Option<String>,
    redirect: Option<String>,
}

fn write_error<S>(error: &Option<HtmlError>, s: S) -> Result<S::Ok, S::Error>
where
    S: Serializer,
{
    let debug = error
        .as_ref()
        .map(|e| format!("{:?}", e))
        .unwrap_or_default();
    debug.serialize(s)
}

fn check(path: impl AsRef<Path>) -> io::Result<Stats> {
    let file = path.as_ref().to_owned();
    let contents = fs::read_to_string(&file)?;
    let original_size = contents.len();
    let html = Html::parse_document(&contents);

    let lang = html::detect_lang(&html);
    let redirect = html::detect_redirect(&html).map(ToOwned::to_owned);

    let (processed_size, error) = html::process(html, lang.as_deref().unwrap_or("en"))
        .map_or_else(|e| (None, Some(e)), |html| (Some(html.html().len()), None));

    Ok(Stats {
        file,
        original_size,
        processed_size,
        error,
        lang,
        redirect,
    })
}

fn main() -> anyhow::Result<()> {
    let mut wtr = csv::WriterBuilder::new()
        .delimiter(b'\t')
        .from_writer(stdout());

    for line in stdin().lines() {
        let stats = check(line?)?;
        wtr.serialize(stats)?;
    }

    Ok(())
}
