use std::{fs, io};
use std::env::consts::EXE_SUFFIX;
use std::fs::DirEntry;
use std::hash::{Hash, Hasher};
use std::io::Write;
use std::path::PathBuf;

use iced_native::subscription::Recipe;
use reqwest::header::{self, HeaderValue};
use self_update::{cargo_crate_version, Move};
use semver::Version;

use crate::{DndSpells, error, Tap, UpdateState};
use crate::error::UpdateError;

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
        _input: futures::stream::BoxStream<'_, E>,
    ) -> futures::stream::BoxStream<'_, Self::Output> {
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
                                #[allow(clippy::cast_precision_loss)]
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
                        #[allow(clippy::let_unit_value)]
                        {
                            let _: () = iced::futures::future::pending().await;
                        }
                        None
                    }
                }
            },
        ))
    }
}

pub fn handle(app: &mut DndSpells, message: Message) -> error::Result<(), UpdateError> {
    match message {
        Message::CheckForUpdate => {
            // ignore any errors here
            let _res = delete_backup_temp_directories();

            let latest_release = self_update::backends::github::ReleaseList::configure()
                .repo_owner("Andrew-Schwartz")
                .repo_name("spells")
                .build()
                .expect("repo owner and name are both set")
                .fetch()?
                .into_iter()
                .find(|release| release.has_target_asset(self_update::get_target()));

            app.update_state = if let Some(latest_release) = latest_release {
                let latest_version = Version::parse(&latest_release.version)
                    .expect("I always use semver correctly");
                let this_version = Version::parse(cargo_crate_version!())
                    .expect("I always use semver correctly");
                if latest_version > this_version {
                    if let Some(asset) = latest_release.asset_for(self_update::get_target(), None) {
                        app.update_url = asset.download_url;
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
            app.update_state = match progress {
                Progress::Started => UpdateState::Downloading(0.0),
                Progress::Advanced(pct) => UpdateState::Downloading(pct),
                Progress::Errored(e) => UpdateState::Errored(e),
                Progress::Finished(None) => UpdateState::UpToDate,
                Progress::Finished(Some(bytes)) => {
                    update_extended(&bytes)?;
                    UpdateState::Downloaded
                }
            };

            Ok(())
        }
    }
}

/// taken from `self_update`, but modified so that it uses the downloaded file
fn update_extended(bytes: &[u8]) -> error::Result<(), UpdateError> {
    let current_exe = std::env::current_exe()?;

    let current_exe_string = current_exe.file_name().unwrap()
        .to_string_lossy()
        .to_string();
    let bin_name = current_exe_string.trim_end_matches(EXE_SUFFIX);

    let tmp_dir_parent = current_exe
        .parent()
        .expect("the current executable is always in a folder")
        .tap(PathBuf::from);
    let tmp_backup_dir_prefix = format!("__{bin_name}_backup");

    if cfg!(windows) {
        // Windows executables can not be removed while they are running, which prevents clean up
        // of the temporary directory by the `tempfile` crate after we move the running executable
        // into it during an update. We clean up any previously created temporary directories here.
        // Ignore errors during cleanup since this is not critical for completing the update.
        for entry in fs::read_dir(&tmp_dir_parent)? {
            let _res = cleanup_backup_temp_directories(
                entry,
                &tmp_backup_dir_prefix,
                &tmp_backup_dir_prefix,
            );
        }
    }

    let tmp_archive_dir_prefix = format!("{bin_name}_download");
    let tmp_archive_dir = tempfile::Builder::new()
        .prefix(&tmp_archive_dir_prefix)
        .tempdir_in(&tmp_dir_parent)?;
    let tmp_archive_path = tmp_archive_dir.path().join(bin_name);
    let mut tmp_archive = fs::File::create(&tmp_archive_path)?;
    tmp_archive.write_all(bytes)?;

    // Make executable
    #[cfg(not(windows))]
    {
        use std::os::unix::fs::PermissionsExt;

        let mut permissions = fs::metadata(&tmp_archive_path)?.permissions();
        permissions.set_mode(0o755);
        fs::set_permissions(&tmp_archive_path, permissions)?;
    }

    let tmp_backup_dir = tempfile::Builder::new()
        .prefix(&tmp_backup_dir_prefix)
        .tempdir_in(&tmp_dir_parent)?;
    let tmp_file_path = tmp_backup_dir.path().join(&tmp_backup_dir_prefix);

    Move::from_source(&tmp_archive_path)
        .replace_using_temp(&tmp_file_path)
        .to_dest(&current_exe)?;

    Ok(())
}


pub fn delete_backup_temp_directories() -> error::Result<(), UpdateError> {
    // Windows executables can not be removed while they are running, which prevents clean up
    // of the temporary directory by the `tempfile` crate after we move the running executable
    // into it during an update. We clean up any previously created temporary directories here.
    // Ignore errors during cleanup since this is not critical for completing the update.
    if cfg!(windows) {
        let current_exe = std::env::current_exe()?;

        let current_exe_string = current_exe.file_name().unwrap()
            .to_string_lossy()
            .to_string();
        let bin_name = current_exe_string.trim_end_matches(EXE_SUFFIX);

        let tmp_dir_parent = current_exe
            .parent()
            .expect("the current executable is always in a folder")
            .tap(PathBuf::from);
        let tmp_backup_dir_prefix = format!("__{bin_name}_backup");

        for entry in fs::read_dir(&tmp_dir_parent)? {
            let _res = cleanup_backup_temp_directories(
                entry,
                &tmp_backup_dir_prefix,
                &tmp_backup_dir_prefix,
            );
        }
    }

    Ok(())
}

fn cleanup_backup_temp_directories(
    entry: io::Result<DirEntry>,
    tmp_dir_prefix: &str,
    expected_tmp_filename: &str,
) -> error::Result<(), UpdateError> {
    let entry = entry?;
    let tmp_dir_name = entry.file_name().into_string()
        .map_err(UpdateError::BadFileName)?;

    // For safety, check that the temporary directory contains only the expected backup
    // binary file before removing. If subdirectories or other files exist then the user
    // is using the temp directory for something else. This is unlikely, but we should
    // be careful with `fs::remove_dir_all`.
    let is_expected_tmp_file = |tmp_file_entry: io::Result<DirEntry>| {
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

    Ok(())
}
