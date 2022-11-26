use std::{fs as sfs, io::Write, path::PathBuf};

use tokio::{fs as afs, io::AsyncWriteExt};

use crate::prelude::*;

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Req<T>
where
    T: Serialize + for<'de> Deserialize<'de>,
{
    dir: String,
    key: String,
    _pd: PhantomData<T>,
}

impl<T> Req<T>
where
    T: Serialize + for<'de> Deserialize<'de>,
{
    const __ROOT_DIR: &str = "data";
    const __FILE_EXT: &str = "bin";

    pub fn new(dir: impl Into<String>, key: impl Into<String>) -> Self {
        Self {
            dir: dir.into(),
            key: key.into(),
            _pd: PhantomData,
        }
    }

    #[must_use]
    pub fn root() -> PathBuf {
        PathBuf::from(Self::__ROOT_DIR)
    }
    #[must_use]
    pub fn dir(&self) -> PathBuf {
        Self::root().join(&self.dir)
    }
    #[must_use]
    pub fn path(&self) -> PathBuf {
        self.dir().join(&self.key).with_extension(Self::__FILE_EXT)
    }

    #[must_use]
    pub fn exists_sync(&self) -> bool {
        sfs::File::open(self.path()).is_ok()
    }
    pub fn read_sync(&self) -> Result<T> {
        let file = sfs::File::open(self.path())?;

        Ok(rmp_serde::from_read(file)?)
    }
    pub fn write_sync(&self, value: &T) -> Result<()> {
        sfs::create_dir_all(self.dir())?;

        let data = rmp_serde::to_vec(value)?;
        let mut file = sfs::File::create(self.path())?;

        file.write_all(&data).map_err(Error::from)
    }
    pub fn remove_sync(&self) -> Result<()> {
        sfs::remove_file(self.path()).map_err(Error::from)
    }
}

impl<T> Req<T>
where
    T: Send + Sync + Serialize + for<'de> Deserialize<'de>,
{
    pub async fn exists(&self) -> bool {
        afs::File::open(self.path()).await.is_ok()
    }
    pub async fn read(&self) -> Result<T> {
        let file = afs::File::open(self.path()).await?;

        Ok(rmp_serde::from_read(file.into_std().await)?)
    }
    pub async fn write(&self, value: &T) -> Result<()> {
        afs::create_dir_all(self.dir()).await?;

        let data = rmp_serde::to_vec(value)?;
        let mut file = afs::File::create(self.path()).await?;

        file.write_all(&data).await.map_err(Error::from)
    }
    pub async fn remove(&self) -> Result<()> {
        afs::remove_file(self.path()).await.map_err(Error::from)
    }
}

pub trait AsRequest
where
    Self: Serialize + for<'de> Deserialize<'de>,
{
    fn as_req(&self) -> Req<Self>;
}

pub trait Request
where
    Self: Serialize + for<'de> Deserialize<'de>,
{
    type Args;

    fn req(args: Self::Args) -> Req<Self>;
}

pub trait StoredSync: Request {
    #[must_use]
    fn exists_sync(args: Self::Args) -> bool {
        Self::req(args).exists_sync()
    }
    fn read_sync(args: Self::Args) -> Result<Self> {
        Self::req(args).read_sync()
    }
    fn write_sync(args: Self::Args, value: &Self) -> Result<()> {
        Self::req(args).write_sync(value)
    }
    fn remove_sync(args: Self::Args) -> Result<()> {
        Self::req(args).remove_sync()
    }
}

impl<T: Request> StoredSync for T {}

#[async_trait]
pub trait StoredAsync
where
    Self: Send + Sync + Request,
    Self::Args: Send + Sync,
{
    async fn exists(args: Self::Args) -> bool {
        Self::req(args).exists().await
    }
    async fn read(args: Self::Args) -> Result<Self> {
        Self::req(args).read().await
    }
    async fn write(args: Self::Args, value: &Self) -> Result<()> {
        Self::req(args).write(value).await
    }
    async fn remove(args: Self::Args) -> Result<()> {
        Self::req(args).remove().await
    }
}

impl<T, A> StoredAsync for T
where
    T: Send + Sync + Request<Args = A>,
    A: Send + Sync,
{
}
