mod ansi;
mod args;
mod highlight;
mod input;
mod language;
mod render;

use std::process;

fn main() {
    if let Err(err) = run() {
        eprintln!("mybat: {err}");
        process::exit(1);
    }
}

fn run() -> Result<(), Box<dyn std::error::Error>> {
    let config = args::Config::parse(std::env::args().skip(1))?;

    if config.help {
        println!("{}", args::help_text());
        return Ok(());
    }

    let source = input::read_source(config.path.as_deref())?;
    let lang = match source
        .path
        .as_deref()
        .and_then(language::Language::from_path)
    {
        Some(lang) => lang,
        None if source.path.is_none() => language::Language::infer_from_content(&source.content),
        None => language::Language::PlainText,
    };

    let output = render::render(&source.content, lang, config.line_numbers);
    print!("{output}");
    Ok(())
}
