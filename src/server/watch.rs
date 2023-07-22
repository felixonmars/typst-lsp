use tower_lsp::lsp_types::{
    DidChangeWatchedFilesRegistrationOptions, FileChangeType, FileEvent, FileSystemWatcher,
    GlobPattern, Registration,
};
use tracing::warn;

use crate::workspace::Workspace;

use super::TypstServer;

static WATCH_TYPST_FILES_REGISTRATION_ID: &str = "watch_typst_files";
static WATCH_FILES_METHOD: &str = "workspace/didChangeWatchedFiles";

impl TypstServer {
    pub fn get_watcher_registration(&self) -> Registration {
        Registration {
            id: WATCH_TYPST_FILES_REGISTRATION_ID.to_owned(),
            method: WATCH_FILES_METHOD.to_owned(),
            register_options: Some(
                serde_json::to_value(DidChangeWatchedFilesRegistrationOptions {
                    watchers: vec![FileSystemWatcher {
                        glob_pattern: GlobPattern::String("**/*".to_owned()),
                        kind: None,
                    }],
                })
                .unwrap(),
            ),
        }
    }

    pub fn handle_file_change_event(&self, workspace: &mut Workspace, event: FileEvent) {
        let Ok(id) = workspace.uri_to_id(&event.uri) else { return };

        match event.typ {
            FileChangeType::CHANGED | FileChangeType::CREATED => workspace.invalidate_local(id),
            FileChangeType::DELETED => workspace.delete_local(id),
            _ => {
                warn!("unexpected event type");
            }
        }
    }
}
