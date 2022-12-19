use crate::prelude::*;

#[allow(clippy::use_self)]
pub mod apply;
pub mod data;
pub mod embed;
pub mod help;
pub mod offer;
pub mod oracle;
pub mod ping;
#[allow(clippy::use_self)]
pub mod poll;
pub mod quote;
pub mod role;

fn __get_any<'c>(o: &'c [ResolvedOption<'c>], n: &'c str) -> Result<&'c ResolvedValue<'c>> {
    o.iter()
        .find(|r| r.name == n)
        .map_or_else(|| Err(Error::MissingValue(Value::Data)), |r| Ok(&r.value))
}

macro_rules! get {
    ($name:ident; $kind:path => $gives:ty) => {
        #[allow(dead_code)]
        pub fn $name<'c>(o: &'c [ResolvedOption<'c>], n: &'c str) -> Result<$gives> {
            match __get_any(o, n)? {
                $kind(v) => Ok(*v),
                _ => Err(Error::InvalidValue(Value::Data, n.to_string())),
            }
        }
    };
    ($name:ident; $kind:path => ref $gives:ty) => {
        #[allow(dead_code)]
        pub fn $name<'c>(o: &'c [ResolvedOption<'c>], n: &'c str) -> Result<&'c $gives> {
            match __get_any(o, n)? {
                $kind(v) => Ok(v),
                _ => Err(Error::InvalidValue(Value::Data, n.to_string())),
            }
        }
    };
}

get!(get_bool; ResolvedValue::Boolean => bool);
get!(get_channel; ResolvedValue::Channel => ref PartialChannel);
get!(get_i64; ResolvedValue::Integer => i64);
get!(get_f64; ResolvedValue::Number => f64);
get!(get_role; ResolvedValue::Role => ref Role);
get!(get_str; ResolvedValue::String => ref str);
get!(get_subcommand; ResolvedValue::SubCommand => ref [ResolvedOption<'c>]);
get!(get_subcommand_group; ResolvedValue::SubCommandGroup => ref [ResolvedOption<'c>]);

pub fn get_user<'c>(
    o: &'c [ResolvedOption<'c>],
    n: &'c str,
) -> Result<(&'c User, Option<&'c PartialMember>)> {
    match __get_any(o, n)? {
        ResolvedValue::User(u, m) => Ok((u, *m)),
        _ => Err(Error::InvalidValue(Value::Data, n.to_string())),
    }
}

pub fn get_input_text<'c>(o: &'c [ActionRow], n: &'c str) -> Result<String> {
    for row in o {
        let Some(ActionRowComponent::InputText(input)) = row.components.first() else {
            continue;
        };

        if input.custom_id == n && !input.value.is_empty() {
            return Ok(input.value.clone());
        }
    }

    Err(Error::MissingValue(Value::Data))
}
