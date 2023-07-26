use std::collections::HashSet;

use tower_lsp::lsp_types::{TextDocumentContentChangeEvent, Url};
use typst::diag::{FileError, FileResult};
use typst::syntax::Source;
use typst::util::Bytes;

use crate::config::PositionEncoding;
use crate::workspace::project::manager::ProjectManager;

use super::cache::Cache;
use super::local::LocalFs;
use super::lsp::LspFs;
use super::{KnownUriProvider, ReadProvider, WriteProvider};

/// Composes [`FsProvider`]s into a single provider for a workspace
#[derive(Default)]
pub struct FsManager {
    lsp: LspFs,
    local: Cache<LocalFs>,
}

impl ReadProvider for FsManager {
    type Error = FileError;

    fn read_bytes(&self, uri: &Url) -> FileResult<Bytes> {
        self.lsp
            .read_bytes(uri)
            .or_else(|()| self.local.read_bytes(uri))
    }

    fn read_source(&self, uri: &Url, project_manager: &ProjectManager) -> FileResult<Source> {
        self.lsp
            .read_source(uri, project_manager)
            .or_else(|()| self.local.read_source(uri, project_manager))
    }
}

impl WriteProvider for FsManager {
    type Error = FileError;

    fn write_raw(&self, uri: &Url, data: &[u8]) -> FileResult<()> {
        self.local.inner().write_raw(uri, data)
    }
}

impl KnownUriProvider for FsManager {
    fn known_uris(&self) -> HashSet<Url> {
        let mut uris = self.local.known_uris();
        uris.extend(self.lsp.known_uris().into_iter());
        uris
    }
}

impl FsManager {
    pub fn open_lsp(
        &mut self,
        uri: Url,
        text: String,
        project_manager: &ProjectManager,
    ) -> FileResult<()> {
        self.lsp.open(uri, text, project_manager)
    }

    pub fn close_lsp(&mut self, uri: &Url) {
        self.lsp.close(uri)
    }

    pub fn edit_lsp(
        &mut self,
        uri: &Url,
        changes: impl IntoIterator<Item = TextDocumentContentChangeEvent>,
        position_encoding: PositionEncoding,
    ) {
        self.lsp.edit(uri, changes, position_encoding)
    }

    pub fn new_local(&mut self, uri: &Url) {
        self.local.cache_new(uri)
    }

    pub fn invalidate_local(&mut self, uri: &Url) {
        self.local.invalidate(uri)
    }

    pub fn delete_local(&mut self, uri: &Url) {
        self.local.delete(uri)
    }

    pub fn clear(&mut self) {
        self.lsp.clear();
        self.local.clear();
    }
}
