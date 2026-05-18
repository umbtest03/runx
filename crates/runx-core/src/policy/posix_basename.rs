pub(crate) fn posix_basename(value: &str) -> String {
    let normalized = value.trim_end_matches(['/', '\\']);
    if normalized.is_empty() {
        return String::new();
    }

    normalized
        .rsplit(['/', '\\'])
        .next()
        .map_or_else(String::new, ToOwned::to_owned)
}

#[cfg(test)]
mod tests {
    use super::posix_basename;

    #[test]
    fn returns_executable_name_from_posix_paths() {
        assert_eq!(posix_basename("/usr/local/bin/node"), "node");
    }

    #[test]
    fn normalizes_windows_separators_into_posix_semantics() {
        assert_eq!(posix_basename(r"C:\Tools\node.exe"), "node.exe");
    }

    #[test]
    fn handles_mixed_separators_and_trailing_slashes() {
        assert_eq!(posix_basename(r"C:\Tools/bin/bash/"), "bash");
        assert_eq!(posix_basename("/"), "");
    }
}
