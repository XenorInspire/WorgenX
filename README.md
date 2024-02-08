# WorgenX-2.0

WorgenX-2.0 is a powerful Rust wordlist generator with many functionalities.<br>
> [!NOTE]  
> This project is a fully rewritten Rust version of <a href="https://github.com/XenorInspire/WorgenX">WorgenX</a>, initially developed in C code.



## Features

- [x] Generate a relevant wordlist with a custom mask and charset
- [x] Generate a list of random passwords with a strong entropy
- [x] Benchmark the performance of your CPU
- [ ] Benchmark the performance of your GPU (not available yet)


## Installation


### Install the packaged versions

//WIP

### Install from the source code

#### In a directory of your choice, clone the repository :  
```
git clone https://github.com/XenorInspire/WorgenX-2.0.git
```
Move in the directory :  
```
cd WorgenX-2.0/
```
#### Compile the project :

For CLI version : 
```
cargo build --features cli
```

For GUI version : 
```
cargo build --features gui
```

## Use WorgenX

### In CLI mode :

```
worgenX <command> [options]
```

Use the '-h' or '--help' argument to display the help menu :

```
$ worgenX --help

Usage: worgenx <command> [options]
Commands:
  -w, --wordlist        Generate a wordlist
  -p, --passwd          Generate random password(s)
  -b, --benchmark       Benchmark CPU
  -v, --version         Display the version of WorgenX
  -h, --help            Display this help message


You can find below the options for the main features of WorgenX:

  --- Dictionary generation ---
  You must specify at least one of the following options: -l, -u, -n, -s
    -l, --lowercase                     Add lowercase characters to the words
    -u, --uppercase                     Add uppercase characters to the words
    -n, --numbers                       Add numbers to the words
    -x, --special-characters            Add special characters to the words

  This parameter is mandatory:
    -m <mask>, --mask <mask>            Mask used to generate the words
    -o <path>, --output <path>          Save the wordlist in a text file

  The following options are optional:
    -d, --disable-loading-bar           Disable the loading bar when generating the wordlist
    -t <threads>, --threads <threads>   Number of threads to use to generate the passwords
                                        By default, the number of threads is based on the number of physical cores of the CPU

  --- Password generation ---
  You must specify at least one of the following options: -l, -u, -n, -s
    -l, --lowercase                     Add lowercase characters to the words
    -u, --uppercase                     Add uppercase characters to the words
    -n, --numbers                       Add numbers to the words
    -x, --special-characters            Add special characters to the words

  These parameters are mandatory:
    -s <size>, --size <size>            Size of the passwords in characters
    -c <count>, --count <count>         Number of passwords to generate

  The following options are optional:
    -o <path>, --output <path>          Save the passwords in a file
    -O <path>, --output-only <path>     Save the passwords only in a file, not in stdout
    -j, --json                          Output in JSON format
                                        Combine with -o to save the json output in a file

```

### In GUI mode :

Just start worgenX_gui :
```bash
$ worgenX_gui
```

# Mirrors

- <a href="https://github.com/XenorInspire/WorgenX-2.0">Github</a>

# Licence

This application is licensed under [GNU General Public License, Version 3.0].

[GNU General Public License, Version 3.0]:
 http://www.gnu.org/licenses/gpl-3.0-standalone.html
