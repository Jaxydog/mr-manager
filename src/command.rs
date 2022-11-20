use serenity::model::{
    prelude::{
        command::CommandOptionType, Attachment, PartialChannel, PartialMember, ResolvedOption,
        ResolvedValue, Role,
    },
    user::User,
};

use crate::utility::{Error, Result};

#[allow(clippy::use_self)]
pub mod apply;
pub mod embed;
pub mod offer;
pub mod ping;
#[allow(clippy::use_self)]
pub mod poll;

#[inline]
fn __get_data<'d>(opts: &'d [ResolvedOption], name: &str) -> Result<&'d ResolvedValue<'d>> {
    opts.iter()
        .find(|d| d.name == name)
        .map(|v| &v.value)
        .ok_or(Error::MissingCommandData)
}

fn get_attachment<'cmd>(opts: &'cmd [ResolvedOption], name: &str) -> Result<&'cmd Attachment> {
    match __get_data(opts, name) {
        Ok(ResolvedValue::Attachment(a)) => Ok(a),
        Err(e) => Err(e),
        _ => Err(Error::InvalidCommandData),
    }
}
fn get_autocomplete<'cmd>(
    opts: &'cmd [ResolvedOption],
    name: &str,
) -> Result<(CommandOptionType, &'cmd str)> {
    match __get_data(opts, name) {
        Ok(ResolvedValue::Autocomplete { kind, value }) => Ok((*kind, value)),
        Err(e) => Err(e),
        _ => Err(Error::InvalidCommandData),
    }
}
fn get_bool(opts: &[ResolvedOption], name: &str) -> Result<bool> {
    match __get_data(opts, name) {
        Ok(ResolvedValue::Boolean(b)) => Ok(*b),
        Err(e) => Err(e),
        _ => Err(Error::InvalidCommandData),
    }
}
fn get_channel<'cmd>(opts: &'cmd [ResolvedOption], name: &str) -> Result<&'cmd PartialChannel> {
    match __get_data(opts, name) {
        Ok(ResolvedValue::Channel(c)) => Ok(c),
        Err(e) => Err(e),
        _ => Err(Error::InvalidCommandData),
    }
}
fn get_i64(opts: &[ResolvedOption], name: &str) -> Result<i64> {
    match __get_data(opts, name) {
        Ok(ResolvedValue::Integer(i)) => Ok(*i),
        Err(e) => Err(e),
        _ => Err(Error::InvalidCommandData),
    }
}
fn get_f64(opts: &[ResolvedOption], name: &str) -> Result<f64> {
    match __get_data(opts, name) {
        Ok(ResolvedValue::Number(n)) => Ok(*n),
        Err(e) => Err(e),
        _ => Err(Error::InvalidCommandData),
    }
}
fn get_role<'cmd>(opts: &'cmd [ResolvedOption], name: &str) -> Result<&'cmd Role> {
    match __get_data(opts, name) {
        Ok(ResolvedValue::Role(r)) => Ok(r),
        Err(e) => Err(e),
        _ => Err(Error::InvalidCommandData),
    }
}
fn get_str<'cmd>(opts: &'cmd [ResolvedOption], name: &str) -> Result<&'cmd str> {
    match __get_data(opts, name) {
        Ok(ResolvedValue::String(s)) => Ok(s),
        Err(e) => Err(e),
        _ => Err(Error::InvalidCommandData),
    }
}
fn get_subcommand<'cmd>(
    opts: &'cmd [ResolvedOption],
    name: &str,
) -> Result<&'cmd Vec<ResolvedOption<'cmd>>> {
    match __get_data(opts, name) {
        Ok(ResolvedValue::SubCommand(v)) => Ok(v),
        Err(e) => Err(e),
        _ => Err(Error::InvalidCommandData),
    }
}
fn get_subcommand_group<'cmd>(
    opts: &'cmd [ResolvedOption],
    name: &str,
) -> Result<&'cmd Vec<ResolvedOption<'cmd>>> {
    match __get_data(opts, name) {
        Ok(ResolvedValue::SubCommandGroup(v)) => Ok(v),
        Err(e) => Err(e),
        _ => Err(Error::InvalidCommandData),
    }
}
fn get_user<'cmd>(
    opts: &'cmd [ResolvedOption],
    name: &str,
) -> Result<(&'cmd User, Option<&'cmd PartialMember>)> {
    match __get_data(opts, name) {
        Ok(ResolvedValue::User(u, g)) => Ok((u, g.as_ref().copied())),
        Err(e) => Err(e),
        _ => Err(Error::InvalidCommandData),
    }
}
