use crate::prelude::*;

pub mod data;
pub mod embed;
pub mod help;
pub mod offer;
pub mod oracle;
pub mod ping;
#[allow(clippy::use_self)]
pub mod poll;
pub mod role;

fn __get_data<'c>(
    c: &'c str,
    o: &'c [ResolvedOption<'c>],
    n: &'static str,
) -> Result<&'c ResolvedValue<'c>> {
    o.iter().find(|o| o.name == n).map_or_else(
        || Err(Error::MissingCommandData(c.to_string(), n)),
        |o| Ok(&o.value),
    )
}

macro_rules! get_data {
    ($name:ident, $variant:path => <$result:ty>) => {
        pub fn $name<'c>(
            c: &'c str,
            o: &'c [ResolvedOption<'c>],
            n: &'static str,
        ) -> Result<$result> {
            match __get_data(c, o, n)? {
                $variant(v) => Ok(*v),
                _ => Err(Error::InvalidCommandData(c.to_string(), n)),
            }
        }
    };
    ($name:ident, $variant:path => $result:ty) => {
        pub fn $name<'c>(
            c: &'c str,
            o: &'c [ResolvedOption<'c>],
            n: &'static str,
        ) -> Result<&'c $result> {
            match __get_data(c, o, n)? {
                $variant(v) => Ok(v),
                _ => Err(Error::InvalidCommandData(c.to_string(), n)),
            }
        }
    };
}

get_data!(get_bool, ResolvedValue::Boolean => <bool>);
get_data!(get_channel, ResolvedValue::Channel => PartialChannel);
get_data!(get_i64, ResolvedValue::Integer => <i64>);
get_data!(get_f64, ResolvedValue::Number => <f64>);
get_data!(get_role, ResolvedValue::Role => Role);
get_data!(get_str, ResolvedValue::String => str);
get_data!(get_subcommand, ResolvedValue::SubCommand => Vec<ResolvedOption<'c>>);
get_data!(get_subcommand_group, ResolvedValue::SubCommandGroup => Vec<ResolvedOption<'c>>);

pub fn get_user<'c>(
    c: &'c str,
    o: &'c [ResolvedOption<'c>],
    n: &'static str,
) -> Result<(&'c User, Option<&'c PartialMember>)> {
    match __get_data(c, o, n)? {
        ResolvedValue::User(user, member) => Ok((user, *member)),
        _ => Err(Error::InvalidCommandData(c.to_string(), n)),
    }
}
