use pulldown_cmark;
use sailfish::TemplateOnce;
use std::{collections::HashMap, ffi::OsStr, fs, io::prelude::*, io::Result, path::Path};
use walkdir::{DirEntry, WalkDir};

#[derive(TemplateOnce)]
#[template(path = "base.stpl")]
struct BaseTemplate {
    title: String,
    content: String,
}

pub fn build(watch_paths: &HashMap<&str, &Path>) -> Result<()> {
    let output_dir: &Path = Path::new("./output");
    let _ = fs::remove_dir_all(output_dir);

    println!("building...");

    let content_dir = watch_paths.get(&"content").unwrap();
    build_posts(&content_dir, output_dir)?;
    println!("done building!");
    Ok(())
}

fn build_posts(content_dir: &Path, output_dir: &Path) -> Result<()> {
    let md_extension = OsStr::new("md");
    let content_walker = WalkDir::new(content_dir)
        .follow_links(true)
        .into_iter()
        .filter_map(|e| e.ok());

    let markdown_files: Vec<DirEntry> = content_walker
        .filter(|e| e.path().extension() == Some(md_extension))
        .collect();
    let mut html_files = Vec::with_capacity(markdown_files.len());

    for file in &markdown_files {
        let mut open_file = fs::File::open(file.path())?;
        let mut file_contents = String::new();
        open_file.read_to_string(&mut file_contents)?;

        let parser = pulldown_cmark::Parser::new(&file_contents);
        let mut parsed_content = String::new();
        pulldown_cmark::html::push_html(&mut parsed_content, parser);

        let ctx = BaseTemplate {
            title: String::from("Cameron Otsuka"),
            content: parsed_content,
        };
        let html_content = ctx.render_once().unwrap();

        match file.path().strip_prefix(content_dir) {
            Ok(markdown_file) => {
                let html_file = output_dir.join(markdown_file).with_extension("html");
                let parent_folder = Path::new(&html_file).parent().unwrap();
                fs::create_dir_all(parent_folder)?;
                fs::write(&html_file, html_content)?;
                html_files.push(html_file);
            }
            Err(_) => println!("error stripping prefix from markdown file!"),
        }
    }
    Ok(())
}
