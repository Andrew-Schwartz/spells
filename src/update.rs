use std::env::consts::EXE_SUFFIX;
use std::fs;
use std::hash::{Hash, Hasher};
use std::io::Write;
use std::path::{Path, PathBuf};

use iced_native::subscription::Recipe;
use reqwest::header::{self, HeaderValue};
use self_update::{cargo_crate_version, Move};
use semver::Version;

use crate::{DndSpells, UpdateState};

#[derive(Clone, Debug)]
pub enum Message {
    CheckForUpdate,
    Progress(Progress),
}

#[derive(Clone, Debug)]
pub enum Progress {
    Started,
    Advanced(f32),
    Finished(Option<Vec<u8>>),
    Errored(String),
}

pub struct Download {
    pub url: String,
}

pub enum State {
    Ready(String),
    Downloading {
        response: reqwest::Response,
        buf: Vec<u8>,
        total: u64,
        downloaded: u64,
    },
    /// true if new version was downloaded
    Finished,
}

impl<H: Hasher, E> Recipe<H, E> for Download {
    type Output = Progress;

    fn hash(&self, state: &mut H) {
        struct Marker;
        std::any::TypeId::of::<Marker>().hash(state);
    }

    fn stream(
        self: Box<Self>,
        _input: futures::stream::BoxStream<E>,
    ) -> futures::stream::BoxStream<Self::Output> {
        Box::pin(futures::stream::unfold(
            State::Ready(self.url),
            |state| async move {
                match state {
                    State::Ready(url) => {
                        let client = reqwest::Client::new();
                        let response = client.get(url)
                            .header(header::USER_AGENT, HeaderValue::from_str("rust-reqwest/update").unwrap())
                            .header(header::ACCEPT, HeaderValue::from_str("application/octet-stream").unwrap())
                            .send().await;
                        match response {
                            Ok(resp) => {
                                match resp.content_length() {
                                    Some(total) => Some((Progress::Started, State::Downloading {
                                        response: resp,
                                        buf: vec![],
                                        total,
                                        downloaded: 0,
                                    })),
                                    None => Some((Progress::Finished(Some(resp.bytes().await.unwrap().to_vec())), State::Finished)),
                                }
                            }
                            Err(e) => Some((Progress::Errored(e.to_string()), State::Finished)),
                        }
                    }
                    State::Downloading {
                        mut response,
                        mut buf,
                        total,
                        mut downloaded,
                    } => {
                        match response.chunk().await {
                            Ok(Some(bytes)) => {
                                downloaded += bytes.len() as u64;
                                let percent = downloaded as f32 / total as f32 * 100.0;
                                buf.extend_from_slice(&bytes);
                                Some((Progress::Advanced(percent), State::Downloading {
                                    response,
                                    buf,
                                    total,
                                    downloaded,
                                }))
                            }
                            Ok(None) => Some((Progress::Finished(Some(buf)), State::Finished)),
                            Err(e) => Some((Progress::Errored(e.to_string()), State::Finished)),
                        }
                    }
                    State::Finished => {
                        // ig?

                        // We do not let the stream die, as it would start a
                        // new download repeatedly if the user is not careful
                        // in case of errors.
                        let _: () = iced::futures::future::pending().await;

                        None
                    }
                }
            },
        ))
    }
}

pub fn handle(app: &mut DndSpells, message: Message) -> anyhow::Result<()> {
    match message {
        Message::CheckForUpdate => {
            let latest_release = self_update::backends::github::ReleaseList::configure()
                .repo_owner("Andrew-Schwartz")
                .repo_name("spells")
                .build()?
                .fetch()?
                .into_iter()
                .filter(|release| release.has_target_asset(self_update::get_target()))
                .next();

            app.update_state = if let Some(latest_release) = latest_release {
                if Version::parse(&latest_release.version)? > Version::parse(cargo_crate_version!())? {
                    if let Some(asset) = latest_release.asset_for(self_update::get_target()) {
                        app.update_url = asset.download_url.clone();
                        UpdateState::Ready
                    } else {
                        UpdateState::UpToDate
                    }
                } else {
                    UpdateState::UpToDate
                }
            } else {
                UpdateState::UpToDate
            };
            Ok(())
        }
        Message::Progress(progress) => {
            // println!("progress = {:?}", progress);

            app.update_state = match progress {
                Progress::Started => UpdateState::Downloading(0.0),
                Progress::Advanced(pct) => UpdateState::Downloading(pct),
                Progress::Finished(None) => UpdateState::UpToDate,
                Progress::Errored(e) => UpdateState::Errored(e),
                Progress::Finished(Some(bytes)) => {
                    update_extended(bytes)?;
                    UpdateState::Downloaded
                }
            };

            Ok(())
        }
    }
}

/// taken from self_update, but modified so that it uses the downloaded file
fn update_extended(bytes: Vec<u8>) -> anyhow::Result<()> {
    let bin_install_path = std::env::current_exe()?;

    let string = bin_install_path.file_name().unwrap()
        .to_string_lossy()
        .to_string();
    let bin_name = string.trim_end_matches(EXE_SUFFIX);

    let tmp_dir_parent = bin_install_path
        .parent()
        .map(PathBuf::from)
        .ok_or_else(|| anyhow::Error::msg("Failed to determine parent dir"))?;
    let tmp_backup_dir_prefix = format!("__{}_backup", bin_name);
    let tmp_backup_filename = tmp_backup_dir_prefix.clone();

    if cfg!(windows) {
        // Windows executables can not be removed while they are running, which prevents clean up
        // of the temporary directory by the `tempfile` crate after we move the running executable
        // into it during an update. We clean up any previously created temporary directories here.
        // Ignore errors during cleanup since this is not critical for completing the update.
        let _ = cleanup_backup_temp_directories(
            &tmp_dir_parent,
            &tmp_backup_dir_prefix,
            &tmp_backup_filename,
        );
    }

    let tmp_archive_dir_prefix = format!("{}_download", bin_name);
    let tmp_archive_dir = tempfile::Builder::new()
        .prefix(&tmp_archive_dir_prefix)
        .tempdir_in(&tmp_dir_parent)?;
    let tmp_archive_path = tmp_archive_dir.path().join("spells");
    let mut tmp_archive = fs::File::create(&tmp_archive_path)?;
    tmp_archive.write_all(&bytes)?;

    let bin_path_in_archive = bin_name;
    let new_exe = tmp_archive_dir.path().join(&bin_path_in_archive);

    // Make executable
    #[cfg(not(windows))]
        {
            let mut permissions = fs::metadata(&new_exe)?.permissions();
            permissions.set_mode(0o755);
            fs::set_permissions(&new_exe, permissions)?;
        }

    let tmp_backup_dir = tempfile::Builder::new()
        .prefix(&tmp_backup_dir_prefix)
        .tempdir_in(&tmp_dir_parent)?;
    let tmp_file_path = tmp_backup_dir.path().join(&tmp_backup_filename);

    Move::from_source(&new_exe)
        .replace_using_temp(&tmp_file_path)
        .to_dest(&bin_install_path)?;

    Ok(())
}

fn cleanup_backup_temp_directories<P: AsRef<Path>>(
    tmp_dir_parent: P,
    tmp_dir_prefix: &str,
    expected_tmp_filename: &str,
) -> anyhow::Result<()> {
    for entry in fs::read_dir(tmp_dir_parent)? {
        let entry = entry?;
        let tmp_dir_name = if let Ok(tmp_dir_name) = entry.file_name().into_string() {
            tmp_dir_name
        } else {
            continue;
        };

        // For safety, check that the temporary directory contains only the expected backup
        // binary file before removing. If subdirectories or other files exist then the user
        // is using the temp directory for something else. This is unlikely, but we should
        // be careful with `fs::remove_dir_all`.
        let is_expected_tmp_file = |tmp_file_entry: std::io::Result<fs::DirEntry>| {
            tmp_file_entry
                .ok()
                .filter(|e| e.file_name() == expected_tmp_filename)
                .is_some()
        };

        if tmp_dir_name.starts_with(tmp_dir_prefix)
            && fs::read_dir(entry.path())?.all(is_expected_tmp_file)
        {
            fs::remove_dir_all(entry.path())?;
        }
    }
    Ok(())
}
