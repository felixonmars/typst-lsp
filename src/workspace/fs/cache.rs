use elsa::sync::FrozenMap;
use once_cell::sync::OnceCell;
use tower_lsp::lsp_types::Url;
use typst::syntax::Source;
use typst::util::Bytes;

use crate::workspace::project::manager::ProjectManager;

use super::ReadProvider;

#[derive(Default)]
pub struct Cache<Fs: ReadProvider> {
    entries: FrozenMap<Url, Box<CacheEntry>>,
    fs: Fs,
}

impl<Fs: ReadProvider> ReadProvider for Cache<Fs> {
    type Error = Fs::Error;

    fn read_bytes(&self, uri: &Url) -> Result<Bytes, Self::Error> {
        self.read_bytes_ref(uri).cloned()
    }

    fn read_source(
        &self,
        uri: &Url,
        project_manager: &ProjectManager,
    ) -> Result<Source, Self::Error> {
        self.read_source_ref(uri, project_manager).cloned()
    }
}

impl<Fs: ReadProvider> Cache<Fs> {
    /// Gives a reference to the wrapped [`ReadProvider`]. Note that this can cause cache
    /// invalidation errors if the inner reference writes to a cached file without the cache being
    /// notified.
    pub fn inner(&self) -> &Fs {
        &self.fs
    }

    pub fn read_bytes_ref(&self, uri: &Url) -> Result<&Bytes, Fs::Error> {
        self.entry(uri.clone()).read_bytes(uri, &self.fs)
    }

    pub fn read_source_ref(
        &self,
        uri: &Url,
        project_manager: &ProjectManager,
    ) -> Result<&Source, Fs::Error> {
        self.entry(uri.clone())
            .read_source(uri, &self.fs, project_manager)
    }

    pub fn cache_new(&mut self, uri: &Url) {
        self.entry_mut(uri.clone());
    }

    pub fn invalidate(&mut self, uri: &Url) {
        self.entry_mut(uri.clone()).invalidate()
    }

    pub fn delete(&mut self, uri: &Url) {
        self.entries.as_mut().remove(uri);
    }

    pub fn clear(&mut self) {
        self.entries.as_mut().clear()
    }

    fn entry(&self, uri: Url) -> &CacheEntry {
        self.entries
            .get(&uri) // don't take write lock unnecessarily
            .unwrap_or_else(|| self.entries.insert(uri, Box::default()))
    }

    fn entry_mut(&mut self, uri: Url) -> &mut CacheEntry {
        self.entries.as_mut().entry(uri).or_default()
    }
}

#[derive(Default)]
pub struct CacheEntry {
    source: OnceCell<Source>,
    bytes: OnceCell<Bytes>,
}

impl CacheEntry {
    pub fn read_bytes<Fs: ReadProvider>(&self, uri: &Url, fs: &Fs) -> Result<&Bytes, Fs::Error> {
        self.bytes.get_or_try_init(|| fs.read_bytes(uri))
    }

    pub fn read_source<Fs: ReadProvider>(
        &self,
        uri: &Url,
        fs: &Fs,
        project_manager: &ProjectManager,
    ) -> Result<&Source, Fs::Error> {
        self.source
            .get_or_try_init(|| fs.read_source(uri, project_manager))
    }

    pub fn invalidate(&mut self) {
        self.source.take();
        self.bytes.take();
    }
}
