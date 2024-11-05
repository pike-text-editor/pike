use crate::config::Config;
use scribe::Workspace;

/// Backend of the app
pub struct Pike {
    workspace: Workspace,
    config: Config,
}

#[cfg(test)]
mod test {
    #[test]
    fn doesnt_fail() {
        assert!(true)
    }
}
