use futures::{
    channel::mpsc::{channel, Receiver},
    SinkExt, StreamExt,
};
use notify::{Config, Event, RecommendedWatcher, RecursiveMode, Watcher};
use std::{collections::HashMap, path::Path};

mod templates;

fn main() {
    let content_dir: &Path = Path::new("./content");
    let public_dir: &Path = Path::new("./public");
    let template_dir: &Path = Path::new("./templates");

    let watch_paths: HashMap<&str, &Path> = HashMap::from([
        ("content", content_dir),
        ("public", public_dir),
        ("template", template_dir),
    ]);
    match templates::build(&watch_paths) {
        Err(e) => println!("{:?}", e),
        _ => (),
    }

    futures::executor::block_on(async {
        if let Err(e) = async_watch(&watch_paths).await {
            println!("error: {:?}", e)
        }
    });
}

fn async_watcher() -> notify::Result<(RecommendedWatcher, Receiver<notify::Result<Event>>)> {
    let (mut tx, rx) = channel(1);

    // Automatically select the best implementation for the platform.
    let watcher = RecommendedWatcher::new(
        move |res| {
            futures::executor::block_on(async {
                tx.send(res).await.unwrap();
            })
        },
        Config::default(),
    )?;

    Ok((watcher, rx))
}

async fn async_watch(watch_paths: &HashMap<&str, &Path>) -> notify::Result<()> {
    let (mut watcher, mut rx) = async_watcher()?;

    // Add directories to be watched. All files and subdirectories will be monitored for changes.
    for path in watch_paths.values() {
        watcher.watch(path.as_ref(), RecursiveMode::Recursive)?;
    }

    while let Some(res) = rx.next().await {
        match res {
            Ok(_) => templates::build(watch_paths)?,
            Err(e) => println!("watch error: {:?}", e),
        }
    }

    Ok(())
}
