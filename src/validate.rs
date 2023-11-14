use anyhow::{bail, Result};

const WORKSPACE_CHARS: &str = "abcdefghijklmnopqrstuvwxyzABCDEFGHIJKLMNOPQRSTUVWXYZ-_";

pub fn valid_workspace_name(s: &str) -> Result<String> {
    let s = s.trim();
    if s.len() < 2 {
        bail!("too short workspace name: {}", s);
    }

    if s.len() > 25 {
        bail!("too long workspace name");
    }

    let mut invalid = s
        .chars()
        .filter(|ch| !WORKSPACE_CHARS.contains(*ch))
        .collect::<Vec<char>>();
    invalid.dedup();

    if invalid.is_empty() {
        Ok(s.to_string())
    } else {
        let s: String = invalid.iter().collect();
        bail!("contains invalid characters: {}", s)
    }
}

#[cfg(test)]
mod tests {
    use super::valid_workspace_name;

    #[test]
    fn valid_workspace_names() {
        let names = ["abc", "Abettername", "work", "work_space", "work-space"];
        for name in names {
            let res = valid_workspace_name(name);
            assert!(res.is_ok());
        }
    }

    #[test]
    fn invalid_workspace_names() {
        let names = ["", " ", "a", ".", "!#1238"];
        for name in names {
            let res = valid_workspace_name(name);
            assert!(res.is_err());
        }
    }
}
