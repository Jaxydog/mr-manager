use std::{
    fs::{create_dir_all, remove_file, File},
    io::Write,
    path::PathBuf,
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

    #[allow(clippy::needless_pass_by_value)]
    pub fn new(dir: impl ToString, key: impl ToString) -> Self {
        Self {
            dir: dir.to_string(),
            key: key.to_string(),
            _marker: PhantomData,
        }
    }

    pub fn dir(&self) -> PathBuf {
        PathBuf::from(Self::DIR).join(&self.dir)
    }
    pub fn path(&self) -> PathBuf {
        self.dir().join(&self.key).with_extension(Self::EXT)
    }

    pub fn read(&self) -> Result<T> {
        let file = File::open(self.path())?;

        Ok(rmp_serde::from_read(file)?)
    }
    pub fn write(&self, value: &T) -> Result<()> {
        create_dir_all(self.dir())?;

        let raw = rmp_serde::to_vec(value)?;
        let mut file = File::create(self.path())?;

        file.write_all(&raw).map_err(Error::from)
    }
    pub fn remove(&self) -> Result<()> {
        remove_file(self.path()).map_err(Error::from)
    }
}

pub trait TryNewReq<T>
where
    Self: Send + Sync + Serialize + for<'de> Deserialize<'de>,
    T: Send + Sync,
{
    fn try_new_req(_: T) -> Result<Req<Self>>;

    fn try_read(value: T) -> Result<Self> {
        Self::try_new_req(value)?.read()
    }
}

pub trait NewReq<T>
where
    Self: Send + Sync + Serialize + for<'de> Deserialize<'de>,
    T: Send + Sync,
{
    fn new_req(_: T) -> Req<Self>;

    fn read(value: T) -> Result<Self> {
        Self::new_req(value).read()
    }
}

impl<T, A> TryNewReq<A> for T
where
    T: NewReq<A>,
    A: Send + Sync,
{
    fn try_new_req(value: A) -> Result<Req<Self>> {
        Ok(Self::new_req(value))
    }
}

pub trait TryAsReq<T>
where
    Self: Send + Sync + Serialize + for<'de> Deserialize<'de>,
    T: Send + Sync,
{
    fn try_as_req(&self, _: T) -> Result<Req<Self>>;

    fn try_write(&self, value: T) -> Result<()> {
        self.try_as_req(value)?.write(self)
    }
    fn try_remove(self, value: T) -> Result<()> {
        self.try_as_req(value)?.remove()
    }
}

pub trait AsReq<T>
where
    Self: Send + Sync + Serialize + for<'de> Deserialize<'de>,
{
    fn as_req(&self, _: T) -> Req<Self>;

    fn write(&self, value: T) -> Result<()> {
        self.as_req(value).write(self)
    }
    fn remove(self, value: T) -> Result<()> {
        self.as_req(value).remove()
    }
}

impl<T, A> TryAsReq<A> for T
where
    T: AsReq<A>,
    A: Send + Sync,
{
    fn try_as_req(&self, value: A) -> Result<Req<Self>> {
        Ok(self.as_req(value))
    }
}
