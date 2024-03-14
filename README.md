# WorgenX

WorgenX is a powerful Rust wordlist generator with many functionalities.<br>
You can generate efficient wordlists with a custom mask and charset, generate random passwords with a strong entropy, and benchmark the performance of your CPU.

[![License: GPL v3](https://img.shields.io/badge/License-GPLv3-blue.svg)](https://www.gnu.org/licenses/gpl-3.0)


> [!NOTE]  
> This project starts from a fork of <a href="https://github.com/XenorInspire/WorgenX-old">the original project</a>, but the code has been completely rewritten in Rust, and the functionalities have been seriously improved.

## Features

- [x] Generate a relevant wordlist with a custom mask and charset
- [x] Generate a list of random passwords with a strong entropy
- [x] Benchmark the performance of your CPU
- [ ] Benchmark the performance of your GPU (not available yet)
- [ ] Generate a relevant wordlist with a custom mask and charset, using hash algorithms like MD5, SHA1, SHA256, SHA512, etc. (not available yet)


## Installation


### Install the packaged versions

Download the latest version of WorgenX from the [releases page](https://github.com/XenorInspire/WorgenX-2.0/releases) corresponding to your operating system and your CPU architecture.

#### For Debian-based systems (like Ubuntu) :

```
sudo apt install ./worgenx_<version>_<arch>.deb
```

#### For RedHat-based systems (like Fedora) :

```
sudo dnf install ./worgenx_<version>_<arch>.rpm
```

#### For Windows :

Just download the .exe file and execute it. You can also add the path to the environment variables to use it in the command line.

### Install from the source code

First, if you don't have rustup installed, you can install it by following the instructions on the official website : [rustup.rs](https://rustup.rs/)

#### Choose a directory and clone the repository :  
```
git clone https://github.com/XenorInspire/WorgenX.git
```
Move in the directory :  
```
cd WorgenX/
```
#### Compile the project :

For CLI version : 
```
cargo build --release --features cli
```

For GUI version : 
```
cargo build --release --features gui
```

The binary will be in the 'target/release/' directory.

## Use WorgenX

### In CLI mode :

```
worgenX <command> [options]
```

Use the '-h' or '--help' argument to display the help menu :

```
$ worgenX --help

Usage: worgenX <command> [options]
Commands:
  -w, --wordlist        Generate a wordlist
  -p, --passwd          Generate random password(s)
  -b, --benchmark       CPU Benchmark
  -v, --version         Display the version of WorgenX
  -h, --help            Display this help message


You can find below the options for the main features of WorgenX:

  --- Wordlist generation ---
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

  --- CPU Benchmark ---
  The following option is optional:
    -t <threads>, --threads <threads>   Number of threads to use for the CPU benchmark
                                        By default, the number of threads is based on the number of physical cores of the CPU

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
