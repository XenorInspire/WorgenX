use thiserror::Error;

#[derive(Debug, Error)]
pub enum WorgenXError {
    #[cfg(feature = "cli")]
    #[error("{0}")]
    ArgError(ArgError),
    #[error("{0}")]
    SystemError(SystemError),
}

#[cfg(feature = "cli")]
#[derive(Debug, Error)]
pub enum ArgError {
    /// This error is raised if the user doesn't specify any argument
    #[error("Error: no argument specified\nUsage: worgenx_cli <command> [options]\nTry 'worgenx_cli --help' for more information.")]
    NoArgument,
    /// This error is raised if the user specifies an unknown argument
    #[error("Error: unknown argument {0}\nUsage: worgenx_cli <command> [options]\nTry 'worgenx_cli --help' for more information.")]
    UnknownArgument(String),
    /// This error is raised if the user specifies an argument that requires a value but doesn't give it
    #[error("Error: missing value for {0}")]
    MissingValue(String),
    //// This error is raised if the user doesn't specify a mandatory argument
    #[error("Error: missing argument {0}\nUsage: worgenx_cli <command> [options]\nTry 'worgenx_cli --help' for more information.")]
    MissingArgument(String),
    /// This error is raised if the user specified and invalid numerical value for an argument
    #[error("Error: invalid value `{0}` for argument `{1}`\nPlease specify a valid numerical value between 1 and {}\nUsage: worgenx_cli <command> [options]\nTry 'worgenx_cli --help' for more information.", u64::MAX)]
    InvalidNumericalValue(String, String),
    /// This error is raised if there isn't any configuration given by the user (for example just -d or --dict without any values after, or none of the -n, -s, -u, -l arguments)
    #[error("Error: no configuration given for argument. Please specify the mandatory parameters and at least one type of characters.\nUsage: worgenx_cli <command> [options]\nTry 'worgenx_cli --help' for more information.")]
    MissingConfiguration(String),
    /// This error is raised if the user has specified both -o and -O arguments
    #[error("Error: cannot specify both -o and -O arguments\nUsage: worgenx_cli <command> [options]\nTry 'worgenx_cli --help' for more information.")]
    BothOutputArguments,
}

#[derive(Debug, Error)]
pub enum SystemError {
    /// This error is raised if the user hasn't specified a valid path for the -o or --output argument
    #[error("Error: invalid path `{0}`.\nPlease specify a valid path for the output file")]
    InvalidPath(String),
    /// This error is raised if parent folder doesn't exist
    #[error("Error: the folder `{0}` doesn't exist.\nPlease specify a valid path")]
    ParentFolderDoesntExist(String),
    /// This error is raised if the path given by the user is a too long (Windows only)
    #[cfg(target_family = "windows")]
    #[error("Error: the path `{0}` is too long (>260).\nPlease specify a valid path")]
    PathTooLong(String),
    /// This error is raised if the file can't be created
    #[error("Error: unable to create file `{0}`.\n{1}")]
    UnableToCreateFile(String, String),
    /// This error is raised if there is an error while writing to the file
    #[error("Error: unable to write to file `{0}`.\n[{1}]")]
    UnableToWriteToFile(String, String),
    /// This error is raised if the passwords or wordlists folder can't be created (for GUI mode only)
    #[cfg(feature = "gui")]
    #[error("Error: unable to create folder `{0}`.\n{1}")]
    UnableToCreateFolder(String, String),
    /// This error is raised when the file name contains invalid characters
    #[error("Error: The file name `{0}` is invalid")]
    InvalidFilename(String),
}
