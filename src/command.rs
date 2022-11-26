use crate::prelude::*;

pub mod data;
pub mod embed;
pub mod help;
pub mod offer;
pub mod oracle;
pub mod ping;
pub mod role;

fn __get_data_from<'c>(
    c: String,
    o: &[ResolvedOption<'c>],
    n: &'static str,
) -> Result<ResolvedValue<'c>> {
    o.iter().find(|o| o.name == n).map_or_else(
        || Err(Error::MissingCommandData(c, n)),
        |o| Ok(o.value.clone()),
    )
}
fn __get_data<'c>(cmd: &'c CommandInteraction, n: &'static str) -> Result<ResolvedValue<'c>> {
    __get_data_from(cmd.data.name.clone(), &cmd.data.options(), n)
}

macro_rules! get_data_from {
    ($name:ident, $variant:path => $result:ty) => {
        pub fn $name<'c>(c: String, o: &[ResolvedOption<'c>], n: &'static str) -> Result<$result> {
            match __get_data_from(c.clone(), o, n)? {
                $variant(v) => Ok(v.to_owned()),
                _ => Err(Error::InvalidCommandData(c, n)),
            }
        }
    };
}
macro_rules! get_data {
    ($name:ident, $variant:path => $result:ty) => {
        pub fn $name<'c>(cmd: &'c CommandInteraction, n: &'static str) -> Result<$result> {
            match __get_data(cmd, n)? {
                $variant(v) => Ok(v.to_owned()),
                _ => Err(Error::InvalidCommandData(cmd.data.name.clone(), n)),
            }
        }
    };
}

get_data_from!(get_bool_from, ResolvedValue::Boolean => bool);
get_data_from!(get_channel_from, ResolvedValue::Channel => PartialChannel);
get_data_from!(get_i64_from, ResolvedValue::Integer => i64);
get_data_from!(get_f64_from, ResolvedValue::Number => f64);
get_data_from!(get_role_from, ResolvedValue::Role => Role);
get_data_from!(get_str_from, ResolvedValue::String => String);
get_data_from!(get_subcommand_from, ResolvedValue::SubCommand => Vec<ResolvedOption<'c>>);
get_data_from!(get_subcommand_group_from, ResolvedValue::SubCommandGroup => Vec<ResolvedOption<'c>>);

get_data!(get_bool, ResolvedValue::Boolean => bool);
get_data!(get_channel, ResolvedValue::Channel => PartialChannel);
get_data!(get_i64, ResolvedValue::Integer => i64);
get_data!(get_f64, ResolvedValue::Number => f64);
get_data!(get_role, ResolvedValue::Role => Role);
get_data!(get_str, ResolvedValue::String => String);
get_data!(get_subcommand, ResolvedValue::SubCommand => Vec<ResolvedOption<'c>>);
get_data!(get_subcommand_group, ResolvedValue::SubCommandGroup => Vec<ResolvedOption<'c>>);

pub fn get_user(
    cmd: &CommandInteraction,
    n: &'static str,
) -> Result<(User, Option<PartialMember>)> {
    match __get_data(cmd, n)? {
        ResolvedValue::User(a, b) => Ok((a.clone(), b.cloned())),
        _ => Err(Error::InvalidCommandData(cmd.data.name.clone(), n)),
    }
}
