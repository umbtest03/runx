use std::ffi::OsString;

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct ConnectPlan {
    pub json: bool,
}

pub fn parse_connect_plan(args: &[OsString]) -> Result<ConnectPlan, String> {
    let command = os_arg(args, 0)?;
    if command != "connect" {
        return Err("connect parser requires the connect command".to_owned());
    }

    let mut json = false;
    for arg in args.iter().skip(1) {
        let token = arg
            .to_str()
            .ok_or_else(|| "connect arguments must be UTF-8".to_owned())?;
        if token == "--json" {
            json = true;
        }
    }

    Ok(ConnectPlan { json })
}

fn os_arg(args: &[OsString], index: usize) -> Result<&str, String> {
    args.get(index)
        .and_then(|arg| arg.to_str())
        .ok_or_else(|| "connect arguments must be UTF-8".to_owned())
}
