use thiserror::Error;

#[derive(Debug, Clone, Error)]
pub enum WorgenXError {
    #[error("Error: {0}")]
    ArgError(ArgError),
    #[error("Error: {0}")]
    SystemError(SystemError),
}

#[derive(Debug, Clone, Error)]
pub enum ArgError {
    /// This error is raised if the user doesn't specify any argument
    #[error("no argument specified\nUsage: worgenx_cli <command> [options]\nTry 'worgenx_cli --help' for more information.")]
    NoArgument,
    /// This error is raised if the user specifies an unknown argument
    #[error("unknown argument {0}\nUsage: worgenx_cli <command> [options]\nTry 'worgenx_cli --help' for more information.")]
    UnknownArgument(String),
    /// This error is raised if the user specifies an argument that requires a value but doesn't give it
    #[error("missing value for {0}")]
    MissingValue(String),
    //// This error is raised if the user doesn't specify a mandatory argument
    #[error("missing argument {0}\nUsage: worgenx_cli <command> [options]\nTry 'worgenx_cli --help' for more information.")]
    MissingArgument(String),
    /// This error is raised if the user specified and invalid numerical value for an argument
    #[error("invalid value `{0}` for argument `{1}`\nPlease specify a valid numerical value between 1 and {}\nUsage: worgenx_cli <command> [options]\nTry 'worgenx_cli --help' for more information.", u64::MAX)]
    InvalidNumericalValue(String, String),
    /// This error is raised if there isn't any configuration given by the user (for example just -d or --dict without any values after)
    #[error("no configuration given for argument {0}\nUsage: worgenx_cli <command> [options]\nTry 'worgenx_cli --help' for more information.")]
    MissingConfiguration(String),
    /// This error is raised if the user has specified both -o and -O arguments
    #[error("cannot specify both -o and -O arguments\nUsage: worgenx_cli <command> [options]\nTry 'worgenx_cli --help' for more information.")]
    BothOutputArguments,
}

#[derive(Debug, Clone, Error)]
pub enum SystemError {
    /// This error is raised if the user hasn't specified a valid path for the -o or --output argument
    #[error("invalid path `{0}` for output file.\nPlease specify a valid path")]
    InvalidPath(String),
}
