use std::{
    collections::{BTreeMap, BTreeSet},
    ffi::OsStr,
    fs::read_dir,
    path::PathBuf,
};

use serde::{Deserialize, Serialize};
use tokio::{
    fs::{create_dir_all, remove_file, File},
    io::AsyncWriteExt,
};

use super::{Error, Result};

pub const DATA_DIR: &str = "data";
pub const FILE_EXT: &str = "rmp";

#[derive(Clone, Debug, PartialEq, Eq, PartialOrd, Ord)]
pub struct Request {
    dir: PathBuf,
    key: String,
    use_cache: bool,
    use_files: bool,
}

impl Request {
    pub fn new(dir: &str, key: &str) -> Self {
        Self {
            dir: dir.into(),
            key: key.into(),
            use_cache: true,
            use_files: true,
        }
    }
    pub const fn no_cache(mut self) -> Self {
        self.use_cache = false;
        self
    }
    pub const fn no_files(mut self) -> Self {
        self.use_files = false;
        self
    }

    pub fn full_dir(&self) -> PathBuf {
        PathBuf::from(DATA_DIR).join(&self.dir)
    }
    pub fn full_path(&self) -> PathBuf {
        self.full_dir().join(&self.key).with_extension(FILE_EXT)
    }
}

#[derive(Clone, Debug)]
pub struct Storage {
    cache: BTreeMap<PathBuf, Vec<u8>>,
}

impl Storage {
    pub fn new() -> Self {
        Self {
            cache: BTreeMap::new(),
        }
    }

    pub fn keys(&self, dir: &str, cache: bool, files: bool) -> Result<Vec<Request>> {
        if !cache && !files {
            return Err(Error::InvalidRequest);
        }

        let mut values = BTreeSet::new();

        if cache {
            for entry in self.cache.keys() {
                let os_key = entry.file_name().and_then(OsStr::to_str);

                if let Some(key) = os_key {
                    let mut request = Request::new(dir, key);
                    request.use_cache = cache;
                    request.use_files = files;

                    values.insert(request);
                }
            }
        }
        if files {
            for entry in read_dir(dir)? {
                let entry = entry?;

                if !entry.metadata()?.is_file() {
                    continue;
                }

                let os_key = entry.file_name();

                if let Some(key) = os_key.to_str() {
                    let mut request = Request::new(dir, key);
                    request.use_cache = cache;
                    request.use_files = files;

                    values.insert(request);
                }
            }
        }

        Ok(values.into_iter().collect())
    }
    pub async fn values<T>(&mut self, dir: &str, cache: bool, files: bool) -> Result<Vec<T>>
    where
        T: Send + Sync + Serialize + for<'de> Deserialize<'de>,
    {
        let mut values = vec![];

        for request in self.keys(dir, cache, files)? {
            let value = self.get(&request).await?;
            values.push(value);
        }

        Ok(values)
    }
    pub async fn pairs<T>(
        &mut self,
        dir: &str,
        cache: bool,
        files: bool,
    ) -> Result<Vec<(Request, T)>>
    where
        T: Send + Sync + Serialize + for<'de> Deserialize<'de>,
    {
        let mut values = vec![];

        for request in self.keys(dir, cache, files)? {
            let value = self.get(&request).await?;
            values.push((request, value));
        }

        Ok(values)
    }

    pub async fn contains(&self, request: &Request) -> bool {
        let path = request.full_path();

        if request.use_cache && self.cache.contains_key(&path) {
            true
        } else if request.use_files {
            File::open(&path).await.is_ok()
        } else {
            false
        }
    }
    pub async fn contains_all(&self, requests: &[Request]) -> bool {
        for request in requests {
            if !self.contains(request).await {
                return false;
            }
        }

        true
    }
    pub async fn contains_any(&self, requests: &[Request]) -> bool {
        for request in requests {
            if self.contains(request).await {
                return true;
            }
        }

        false
    }

    #[allow(clippy::unwrap_used)]
    pub async fn get<T>(&mut self, request: &Request) -> Result<T>
    where
        T: Send + Sync + Serialize + for<'de> Deserialize<'de>,
    {
        let path = request.full_path();

        if request.use_cache && self.cache.contains_key(&path) {
            let raw = self.cache.get(&path).unwrap();
            let value = rmp_serde::from_slice(raw)?;

            Ok(value)
        } else if request.use_files {
            let file = File::open(&path).await?;
            let value = rmp_serde::from_read(file.into_std().await)?;

            if request.use_cache {
                let raw = rmp_serde::to_vec(&value)?;
                self.cache.insert(path, raw);
            }

            Ok(value)
        } else {
            Err(Error::InvalidRequest)
        }
    }
    pub async fn get_all<T>(&mut self, requests: &[Request]) -> Result<Vec<(PathBuf, T)>>
    where
        T: Send + Sync + Serialize + for<'de> Deserialize<'de>,
    {
        let mut values = vec![];

        for request in requests {
            let value = self.get(request).await?;
            values.push((request.full_path(), value));
        }

        Ok(values)
    }
    pub async fn get_any<T>(&mut self, requests: &[Request]) -> Vec<(PathBuf, Option<T>)>
    where
        T: Send + Sync + Serialize + for<'de> Deserialize<'de>,
    {
        let mut values = vec![];

        for request in requests {
            let value = self.get(request).await.ok();
            values.push((request.full_path(), value));
        }

        values
    }

    pub async fn insert<T>(&mut self, request: &Request, data: &T) -> Result<()>
    where
        T: Send + Sync + Serialize + for<'de> Deserialize<'de>,
    {
        if !request.use_cache && !request.use_files {
            return Err(Error::InvalidRequest);
        }

        let path = request.full_path();
        let raw = rmp_serde::to_vec(data)?;

        if request.use_files {
            create_dir_all(&request.full_dir()).await?;

            let mut file = File::create(&path).await?;
            file.write_all(&raw).await?;
        }
        if request.use_cache {
            self.cache.insert(path, raw);
        }

        Ok(())
    }
    pub async fn insert_all<T>(&mut self, requests: &[(Request, T)]) -> Result<()>
    where
        T: Send + Sync + Serialize + for<'de> Deserialize<'de>,
    {
        for (request, data) in requests {
            self.insert(request, data).await?;
        }

        Ok(())
    }

    pub async fn remove(&mut self, request: &Request) -> Result<()> {
        if !request.use_cache && !request.use_files {
            return Err(Error::InvalidRequest);
        }

        let path = request.full_path();

        if request.use_cache {
            self.cache.remove(&path);
        }
        if request.use_files {
            remove_file(path).await.map_err(Error::from)?;
        }

        Ok(())
    }
    pub async fn remove_all(&mut self, requests: &[Request]) -> Result<()> {
        for request in requests {
            self.remove(request).await?;
        }

        Ok(())
    }

    pub async fn ensure<T>(&mut self, request: &Request, data: &T) -> Result<T>
    where
        T: Send + Sync + Serialize + for<'de> Deserialize<'de>,
    {
        if let Ok(value) = self.get(request).await {
            Ok(value)
        } else {
            self.insert(request, data).await?;
            Ok(self.get(request).await?)
        }
    }
    pub async fn ensure_all<T>(&mut self, requests: &[(Request, T)]) -> Result<Vec<(PathBuf, T)>>
    where
        T: Send + Sync + Serialize + for<'de> Deserialize<'de>,
    {
        let mut values = vec![];

        for (request, data) in requests {
            let value = self.ensure(request, data).await?;
            values.push((request.full_path(), value));
        }

        Ok(values)
    }

    pub async fn assert<T, F>(&mut self, request: &Request, f: F) -> Result<bool>
    where
        T: Send + Sync + Serialize + for<'de> Deserialize<'de>,
        F: Send + Sync + FnOnce(T, PathBuf) -> bool,
    {
        let data = self.get(request).await?;
        Ok(f(data, request.full_path()))
    }
    pub async fn assert_all<T, F>(&mut self, requests: &[Request], f: F) -> Result<bool>
    where
        T: Send + Sync + Serialize + for<'de> Deserialize<'de>,
        F: Send + Sync + Fn(T, PathBuf) -> bool,
    {
        for request in requests {
            if !self.assert(request, &f).await? {
                return Ok(false);
            }
        }

        Ok(true)
    }
    pub async fn assert_any<T, F>(&mut self, requests: &[Request], f: F) -> Result<bool>
    where
        T: Send + Sync + Serialize + for<'de> Deserialize<'de>,
        F: Send + Sync + Fn(T, PathBuf) -> bool,
    {
        for request in requests {
            if self.assert(request, &f).await? {
                return Ok(true);
            }
        }

        Ok(false)
    }

    pub async fn modify<T, F>(&mut self, request: &Request, f: F) -> Result<()>
    where
        T: Send + Sync + Serialize + for<'de> Deserialize<'de>,
        F: Send + Sync + Fn(&mut T, PathBuf),
    {
        let data = &mut self.get(request).await?;
        f(data, request.full_path());
        self.insert(request, data).await
    }
    pub async fn modify_all<T, F>(&mut self, requests: &[Request], f: F) -> Result<()>
    where
        T: Send + Sync + Serialize + for<'de> Deserialize<'de>,
        F: Send + Sync + Fn(&mut T, PathBuf),
    {
        for request in requests {
            self.modify(request, &f).await?;
        }

        Ok(())
    }
}
