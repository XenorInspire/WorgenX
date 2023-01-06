// External crates
use rand::Rng;

// Internal crates
use crate::dict;
use crate::system;

// This struct refers to a random password structure
struct PasswordConfig {
    pub numbers: bool,
    pub special_characters: bool,
    pub uppercase: bool,
    pub lowercase: bool,
    pub length: u64,
}

// This is the main function of the random password generation module
pub fn main_passwd_generation() {
    let mut again = String::from("y");

    while again.eq("y") {
        let password_config = allocate_passwd_config();

        println!("How many passwords do you want to generate ?");
        let number_of_passwords = system::get_user_choice_int();

        println!("Do you want to save the passwords in a file ? (y/n)");
        let choice = system::get_user_choice_yn();
        let passwords = generate_random_passwords(&password_config, number_of_passwords);
        if choice.eq("y") {
            system::save_passwords_into_a_file(&passwords)
        };

        println!("You can find your password(s) below :");
        for password in passwords {
            println!("{}", password);
        }

        println!("\nDo you want to generate another password(s) ? (y/n)");
        again = system::get_user_choice_yn();
    }
}

// This function is charged to allocate the password config structure
fn allocate_passwd_config() -> PasswordConfig {
    let mut password_config = PasswordConfig {
        numbers: false,
        special_characters: false,
        uppercase: false,
        lowercase: false,
        length: 0,
    };
    let mut choice;
    let mut is_option_chosen = false;

    while !is_option_chosen {
        println!("\nChoose what your password is composed of :");
        println!("Uppercase letters (A-Z) ? (y/n)");
        choice = system::get_user_choice_yn();
        match &*choice {
            "y" => {
                password_config.uppercase = true;
                is_option_chosen = true;
            }
            _ => (),
        }

        println!("Lowercase letters (a-z) ? (y/n)");
        choice = system::get_user_choice_yn();
        match &*choice {
            "y" => {
                password_config.lowercase = true;
                is_option_chosen = true;
            }
            _ => (),
        }

        println!("Numbers (0-9) ? (y/n)");
        choice = system::get_user_choice_yn();
        match &*choice {
            "y" => {
                password_config.numbers = true;
                is_option_chosen = true;
            }
            _ => (),
        }

        println!("Special characters ? (y/n)");
        choice = system::get_user_choice_yn();
        match &*choice {
            "y" => {
                password_config.special_characters = true;
                is_option_chosen = true;
            }
            _ => (),
        }

        if !is_option_chosen {
            println!("You must choose at least one option !");
        }
    }

    println!("How long do you want your password to be ?");
    password_config.length = system::get_user_choice_int();

    password_config
}

fn create_passwd_content(password_config: &PasswordConfig) -> Vec<u8> {
    let mut password_content = Vec::new();

    if password_config.uppercase {
        for character in dict::UPPERCASE {
            password_content.push(*character);
        }
    }

    if password_config.lowercase {
        for character in dict::LOWERCASE {
            password_content.push(*character);
        }
    }

    if password_config.numbers {
        for character in dict::NUMBERS {
            password_content.push(*character);
        }
    }

    if password_config.special_characters {
        for character in dict::SPECIAL_CHARACTERS {
            password_content.push(*character);
        }
    }

    password_content
}

// This function is charged to generate an array of random passwords
fn generate_random_passwords(
    password_config: &PasswordConfig,
    number_of_passwords: u64,
) -> Vec<String> {
    let mut passwords: Vec<String> = Vec::new();
    let password_content: Vec<u8> = create_passwd_content(password_config);

    for _ in 0..number_of_passwords {
        let mut rng = rand::thread_rng();
        let password: String = (0..password_config.length)
            .map(|_| {
                let idx = rng.gen_range(0..password_content.len());
                password_content[idx] as char
            })
            .collect();
        passwords.push(password);
    }

    passwords
}
