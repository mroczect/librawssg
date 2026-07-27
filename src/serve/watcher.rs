#[cfg(feature = "serve")]
use notify::{RecursiveMode, Watcher};
#[cfg(feature = "serve")]
use std::path::PathBuf;
#[cfg(feature = "serve")]
use std::sync::mpsc;

#[cfg(feature = "serve")]
pub fn watch_dirs<F>(dirs: &[PathBuf], on_change: F) -> notify::Result<Box<dyn Watcher + Send>>
where
    F: Fn() + Send + 'static,
{
    let (tx, rx) = mpsc::channel();
    let watcher_tx = tx.clone();
    let mut watcher = notify::recommended_watcher(move |res: notify::Result<notify::Event>| {
        if let Ok(event) = res
            && matches!(
                event.kind,
                notify::EventKind::Modify(_)
                    | notify::EventKind::Create(_)
                    | notify::EventKind::Remove(_)
            )
        {
            let _ = watcher_tx.send(());
        }
    })?;

    for dir in dirs {
        if dir.exists() {
            watcher.watch(dir, RecursiveMode::Recursive)?;
        }
    }

    std::thread::spawn(move || {
        while rx.recv().is_ok() {
            on_change();
        }
    });

    Ok(Box::new(watcher))
}
