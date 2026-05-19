use std::ffi::OsString;

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum ConnectAction {
    List,
    Revoke,
    Preprovision,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum ConnectAuthorityKind {
    ReadOnly,
    Constructive,
    Destructive,
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct ConnectPlan {
    pub action: ConnectAction,
    pub provider: Option<String>,
    pub grant_id: Option<String>,
    pub scopes: Vec<String>,
    pub scope_family: Option<String>,
    pub authority_kind: Option<ConnectAuthorityKind>,
    pub target_repo: Option<String>,
    pub target_locator: Option<String>,
    pub json: bool,
}

impl ConnectAuthorityKind {
    pub fn as_str(self) -> &'static str {
        match self {
            Self::ReadOnly => "read_only",
            Self::Constructive => "constructive",
            Self::Destructive => "destructive",
        }
    }
}

// rust-style-allow: long-function because connect has one provider/list/revoke
// grammar and shared option parsing must stay centralized during cutover.
pub fn parse_connect_plan(args: &[OsString]) -> Result<ConnectPlan, String> {
    let command = os_arg(args, 0)?;
    if command != "connect" {
        return Err("connect parser requires the connect command".to_owned());
    }

    let Some(subcommand) = args.get(1).and_then(|arg| arg.to_str()) else {
        return Err("runx connect requires a provider, list, or revoke".to_owned());
    };
    let mut options = ConnectOptions::default();
    let mut positionals = Vec::new();
    let mut index = 2;

    while index < args.len() {
        let token = os_arg(args, index)?;
        if !token.starts_with("--") {
            positionals.push(token.to_owned());
            index += 1;
            continue;
        }

        let (flag, inline_value) = split_flag(token);
        match flag {
            "--json" => {
                if inline_value.is_some() {
                    return Err("--json does not take a value".to_owned());
                }
                options.json = true;
                index += 1;
            }
            "--scope" => {
                let (value, next_index) = flag_value(args, index, flag, inline_value)?;
                options.scopes.extend(split_scopes(&value));
                index = next_index;
            }
            "--scope-family" | "--scope_family" | "--scopeFamily" => {
                let (value, next_index) = flag_value(args, index, flag, inline_value)?;
                options.scope_family = Some(value);
                index = next_index;
            }
            "--authority-kind" | "--authority_kind" | "--authorityKind" => {
                let (value, next_index) = flag_value(args, index, flag, inline_value)?;
                options.authority_kind = Some(parse_authority_kind(&value)?);
                index = next_index;
            }
            "--target-repo" | "--target_repo" | "--targetRepo" => {
                let (value, next_index) = flag_value(args, index, flag, inline_value)?;
                options.target_repo = Some(value);
                index = next_index;
            }
            "--target-locator" | "--target_locator" | "--targetLocator" => {
                let (value, next_index) = flag_value(args, index, flag, inline_value)?;
                options.target_locator = Some(value);
                index = next_index;
            }
            _ => return Err(format!("unknown connect flag {flag}")),
        }
    }

    match subcommand {
        "list" => connect_list_plan(positionals, options),
        "revoke" => connect_revoke_plan(positionals, options),
        provider => connect_preprovision_plan(provider, positionals, options),
    }
}

#[derive(Default)]
struct ConnectOptions {
    scopes: Vec<String>,
    scope_family: Option<String>,
    authority_kind: Option<ConnectAuthorityKind>,
    target_repo: Option<String>,
    target_locator: Option<String>,
    json: bool,
}

fn connect_list_plan(
    positionals: Vec<String>,
    options: ConnectOptions,
) -> Result<ConnectPlan, String> {
    if !positionals.is_empty() {
        return Err("runx connect list does not accept extra arguments".to_owned());
    }
    if has_connect_targeting_options(&options) {
        return Err("runx connect list does not accept provider or scope flags".to_owned());
    }
    Ok(ConnectPlan {
        action: ConnectAction::List,
        provider: None,
        grant_id: None,
        scopes: Vec::new(),
        scope_family: None,
        authority_kind: None,
        target_repo: None,
        target_locator: None,
        json: options.json,
    })
}

fn connect_revoke_plan(
    positionals: Vec<String>,
    options: ConnectOptions,
) -> Result<ConnectPlan, String> {
    if has_connect_targeting_options(&options) {
        return Err("runx connect revoke does not accept provider or scope flags".to_owned());
    }
    let [grant_id] = positionals.as_slice() else {
        return Err("runx connect revoke requires exactly one grant id".to_owned());
    };
    Ok(ConnectPlan {
        action: ConnectAction::Revoke,
        provider: None,
        grant_id: Some(grant_id.clone()),
        scopes: Vec::new(),
        scope_family: None,
        authority_kind: None,
        target_repo: None,
        target_locator: None,
        json: options.json,
    })
}

fn connect_preprovision_plan(
    provider: &str,
    positionals: Vec<String>,
    options: ConnectOptions,
) -> Result<ConnectPlan, String> {
    if provider.trim().is_empty() || provider.starts_with('-') {
        return Err("runx connect requires a provider, list, or revoke".to_owned());
    }
    if !positionals.is_empty() {
        return Err("runx connect provider accepts no extra arguments".to_owned());
    }
    Ok(ConnectPlan {
        action: ConnectAction::Preprovision,
        provider: Some(provider.to_owned()),
        grant_id: None,
        scopes: options.scopes,
        scope_family: options.scope_family,
        authority_kind: options.authority_kind,
        target_repo: options.target_repo,
        target_locator: options.target_locator,
        json: options.json,
    })
}

fn has_connect_targeting_options(options: &ConnectOptions) -> bool {
    !options.scopes.is_empty()
        || options.scope_family.is_some()
        || options.authority_kind.is_some()
        || options.target_repo.is_some()
        || options.target_locator.is_some()
}

fn parse_authority_kind(value: &str) -> Result<ConnectAuthorityKind, String> {
    match value {
        "read_only" => Ok(ConnectAuthorityKind::ReadOnly),
        "constructive" => Ok(ConnectAuthorityKind::Constructive),
        "destructive" => Ok(ConnectAuthorityKind::Destructive),
        _ => Err(format!("invalid connect authority kind {value}")),
    }
}

fn os_arg(args: &[OsString], index: usize) -> Result<&str, String> {
    args.get(index)
        .and_then(|arg| arg.to_str())
        .ok_or_else(|| "connect arguments must be UTF-8".to_owned())
}

fn split_flag(token: &str) -> (&str, Option<&str>) {
    token
        .split_once('=')
        .map_or((token, None), |(flag, value)| (flag, Some(value)))
}

fn flag_value(
    args: &[OsString],
    index: usize,
    flag: &str,
    inline_value: Option<&str>,
) -> Result<(String, usize), String> {
    if let Some(value) = inline_value {
        return Ok((value.to_owned(), index + 1));
    }
    let value = os_arg(args, index + 1).map_err(|_| format!("{flag} requires a value"))?;
    if value.starts_with("--") {
        return Err(format!("{flag} requires a value"));
    }
    Ok((value.to_owned(), index + 2))
}

fn split_scopes(value: &str) -> Vec<String> {
    value
        .split(',')
        .map(str::trim)
        .filter(|scope| !scope.is_empty())
        .map(str::to_owned)
        .collect()
}
