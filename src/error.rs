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
}
