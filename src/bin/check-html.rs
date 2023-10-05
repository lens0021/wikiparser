use std::{
    fs,
    io::{stdin, stdout},
};

use scraper::{Html, Selector};

use om_wikiparser::html;

fn main() -> anyhow::Result<()> {
    let mut wtr = csv::WriterBuilder::new()
        .delimiter(b'\t')
        .from_writer(stdout());

    let selectors = [
        r#"link[rel="mw:PageProp/redirect"]"#,
        r#"link[rel="mw:PageProp/Category"]"#,
        r#"head[prefix*=".wikipedia.org/wiki/Special:Redirect/"]"#,
    ];

    let mut headers = vec![
        "File",
        "OriginalSize",
        "ProcessedSize",
        "Error",
        "Lang",
        "Redirect",
    ];

    for selector in selectors {
        headers.push(selector);
    }
    let selectors = selectors.map(|s| Selector::parse(s).unwrap());

    wtr.write_record(&headers)?;

    for line in stdin().lines() {
        let file = line?;
        let contents = fs::read_to_string(&file)?;
        let original_size = contents.len().to_string();
        let html = Html::parse_document(&contents);

        let selector_counts: Vec<usize> =
            selectors.iter().map(|s| html.select(s).count()).collect();

        let lang = html::detect_lang(&html);
        let redirect = html::detect_redirect(&html).map(ToOwned::to_owned);

        let (processed_size, error) = match html::process(html, lang.as_deref().unwrap_or("en")) {
            Ok(html) => (html.html().len().to_string(), String::default()),
            Err(e) => (String::default(), format!("{:?}", e)),
        };

        let mut row = vec![
            file,
            original_size,
            processed_size,
            error,
            lang.unwrap_or_default(),
            redirect.unwrap_or_default(),
        ];

        for count in selector_counts {
            row.push(count.to_string());
        }

        wtr.write_record(row)?;
    }

    Ok(())
}
