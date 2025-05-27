mod configure;
pub mod path;
use super::messages;
use cmd_arg::cmd_arg::Option;
use std::io::Error;
pub fn system(args: Vec<&Option>) -> Result<(), Error> {
    if args.is_empty() {
        return messages::unknown();
    }

    let sub_cmd = args.first().unwrap().to_owned();
    let sub_args: Vec<&Option> = args[1..].to_vec();
    match sub_cmd.opt_str.as_str() {
        "configure" => configure::configure(sub_args),
        _ => messages::unknown(),
    }
}
