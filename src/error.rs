use thiserror::Error;

#[derive(Debug, Clone, Error)]
pub enum WorgenXError {
    #[error("Error: {0}")]
    ArgError(ArgError),
}

#[derive(Debug, Clone, Error)]
pub enum ArgError {
    // This error is raised if the user doesn't specify any argument
    #[error("no argument specified\nUsage: worgenx_cli <command> [options]\nTry 'worgenx_cli --help' for more information.")]
    NoArgument,
    // This error is raised if the user specifies an unknown argument
    #[error("unknown argument {0}\nUsage: worgenx_cli <command> [options]\nTry 'worgenx_cli --help' for more information.")]
    UnknownArgument(String),
    // This error is raised if arguments are missing
    #[error("missing argument {0}")]
    MissingArgument(String),
    // This error is raised if the user specified and invalid numerical value for an argument
    #[error("invalid value `{0}` for argument `{1}`\nPlease specify a valid numerical value between 1 and {}\nUsage: worgenx_cli <command> [options]\nTry 'worgenx_cli --help' for more information.", u64::MAX)]
    InvalidNumericalValue(String, String),
    // This error is raised if there isn't any configuration given by the user (for example just -d or --dict without any values after)
    #[error("no configuration given for argument {0}\nUsage: worgenx_cli <command> [options]\nTry 'worgenx_cli --help' for more information.")]
    MissingConfiguration(String),
}
