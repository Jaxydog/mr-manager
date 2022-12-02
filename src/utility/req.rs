use std::path::PathBuf;

use tokio::{
    fs::{create_dir_all, remove_file, File},
    io::AsyncWriteExt,
};

use crate::prelude::*;

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Req<T>
where
    T: Send + Sync + Serialize + for<'de> Deserialize<'de>,
{
    dir: String,
    key: String,
    _marker: PhantomData<T>,
}

impl<T> Req<T>
where
    T: Send + Sync + Serialize + for<'de> Deserialize<'de>,
{
    pub const DIR: &str = "data";
    pub const EXT: &str = "rmp";

    pub fn new(dir: impl Into<String>, key: impl Into<String>) -> Self {
        Self {
            dir: dir.into(),
            key: key.into(),
            _marker: PhantomData,
        }
    }

    pub fn dir(&self) -> PathBuf {
        PathBuf::from(Self::DIR).join(&self.dir)
    }
    pub fn path(&self) -> PathBuf {
        self.dir().join(&self.key).with_extension(Self::EXT)
    }

    pub async fn exists(&self) -> bool {
        File::open(self.path()).await.is_ok()
    }
    pub async fn read(&self) -> Result<T> {
        let file = File::open(self.path()).await?;

        Ok(rmp_serde::from_read(file.into_std().await)?)
    }
    pub async fn write(&self, val: &T) -> Result<()> {
        create_dir_all(self.dir()).await?;

        let raw = rmp_serde::to_vec(val)?;
        let mut file = File::create(self.path()).await?;

        file.write_all(&raw).await.map_err(Error::from)
    }
    pub async fn remove(&self) -> Result<()> {
        remove_file(self.path()).await.map_err(Error::from)
    }
}

pub trait TryNewReq: Send + Sync + Serialize + for<'de> Deserialize<'de> {
    type Args: Send + Sync;

    fn try_new_req(args: Self::Args) -> Result<Req<Self>>;
}

pub trait NewReq: Send + Sync + Serialize + for<'de> Deserialize<'de> {
    type Args: Send + Sync;

    fn new_req(args: Self::Args) -> Req<Self>;
}

impl<T: NewReq> TryNewReq for T {
    type Args = <Self as NewReq>::Args;

    fn try_new_req(args: Self::Args) -> Result<Req<Self>> {
        Ok(Self::new_req(args))
    }
}

pub trait TryAsReq: Send + Sync + Serialize + for<'de> Deserialize<'de> {
    fn try_as_req(&self) -> Result<Req<Self>>;
}

pub trait AsReq: Send + Sync + Serialize + for<'de> Deserialize<'de> {
    fn as_req(&self) -> Req<Self>;
}

impl<T: AsReq> TryAsReq for T {
    fn try_as_req(&self) -> Result<Req<Self>> {
        Ok(self.as_req())
    }
}

#[async_trait]
pub trait ExistsReq: TryNewReq {
    async fn exists(args: Self::Args) -> bool {
        if let Ok(req) = Self::try_new_req(args) {
            req.exists().await
        } else {
            false
        }
    }
}

impl<T: TryNewReq> ExistsReq for T {}

#[async_trait]
pub trait ReadReq: TryNewReq {
    async fn read(args: Self::Args) -> Result<Self> {
        Self::try_new_req(args)?.read().await
    }
}

impl<T: TryNewReq> ReadReq for T {}

#[async_trait]
pub trait WriteReq: TryAsReq {
    async fn write(&self) -> Result<()> {
        self.try_as_req()?.write(self).await
    }
}

impl<T: TryAsReq> WriteReq for T {}

#[async_trait]
pub trait RemoveReq: TryAsReq {
    async fn remove(self) -> Result<()> {
        self.try_as_req()?.remove().await
    }
}

impl<T: TryAsReq> RemoveReq for T {}
