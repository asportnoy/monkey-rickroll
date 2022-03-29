use crossterm::{
    cursor::MoveToColumn, execute, style::Print, terminal::Clear, terminal::ClearType,
};
use fastrand;
use num_format::{Locale, ToFormattedString};
use std::{
    env,
    io::stdout,
    process::exit,
    sync::mpsc::{sync_channel, Receiver, SyncSender},
    thread,
    time::SystemTime,
};

const CHARACTERS: [char; 27] = [
    'a', 'b', 'c', 'd', 'e', 'f', 'g', 'h', 'i', 'j', 'k', 'l', 'm', 'n', 'o', 'p', 'q', 'r', 's',
    't', 'u', 'v', 'w', 'x', 'y', 'z', ' ',
];

const SCRIPT: &str = "We're no strangers to love
You know the rules and so do I
A full commitment's what I'm thinking of
You wouldn't get this from any other guy
I just wanna tell you how I'm feeling
Gotta make you understand
Never gonna give you up
Never gonna let you down
Never gonna run around and desert you
Never gonna make you cry
Never gonna say goodbye
Never gonna tell a lie and hurt you
We've known each other for so long
Your heart's been aching but you're too shy to say it
Inside we both know what's been going on
We know the game and we're gonna play it
And if you ask me how I'm feeling
Don't tell me you're too blind to see
Never gonna give you up
Never gonna let you down
Never gonna run around and desert you
Never gonna make you cry
Never gonna say goodbye
Never gonna tell a lie and hurt you
Never gonna give you up
Never gonna let you down
Never gonna run around and desert you
Never gonna make you cry
Never gonna say goodbye
Never gonna tell a lie and hurt you
Never gonna give, never gonna give
(Give you up)
We've known each other for so long
Your heart's been aching but you're too shy to say it
Inside we both know what's been going on
We know the game and we're gonna play it
I just wanna tell you how I'm feeling
Gotta make you understand
Never gonna give you up
Never gonna let you down
Never gonna run around and desert you
Never gonna make you cry
Never gonna say goodbye
Never gonna tell a lie and hurt you
Never gonna give you up
Never gonna let you down
Never gonna run around and desert you
Never gonna make you cry
Never gonna say goodbye
Never gonna tell a lie and hurt you
Never gonna give you up
Never gonna let you down
Never gonna run around and desert you
Never gonna make you cry
Never gonna say goodbye
Never gonna tell a lie and hurt you
";

fn main() {
    println!("");

    let num_threads = env::var("THREADS")
        .unwrap_or("2".to_string())
        .parse::<i32>()
        .unwrap();

    let mut attempts: u128 = 0;
    let mut best_length: i32 = 0;

    let start_time = SystemTime::now();

    // For threads sending their results to the main thread
    // Touple of (last_index, saved_attempts, thread_index)
    let (tx, rx) = sync_channel(64) as (SyncSender<(i32, u128, i32)>, Receiver<(i32, u128, i32)>);
    // Spawn threads
    for i in 0..num_threads {
        // Clone transmitter so I can put it in the thread
        let thread_tx = tx.clone();

        thread::spawn(move || {
            // Save the number of attempts that need to be sent to the main thread
            let mut saved_attempts: u128 = 0;

            let chars = gen_char_vec();

            loop {
                let last_index = run_attempt(&chars);
                saved_attempts += 1;
                // Check if the thread should send info to the main thread
                // This should happen if the thread reaches a new best length or every second
                if last_index > best_length || last_index == -1 || saved_attempts >= 1000000 {
                    if last_index > best_length {
                        // Update saved best length
                        best_length = last_index;
                    };

                    thread_tx.send((last_index, saved_attempts, i)).ok();

                    // Reset saved attempts and last sent time
                    saved_attempts = 0;
                }
            }
        });

        println!("Spawned monkey #{}", i + 1);
    }

    println!(
        "\nAll monkeys spawned! {} monkeys are typing the lyrics of \"Never gonna give you up\" ({} letters)\n",
        num_threads,
        num_chars(&SCRIPT.to_string()).to_formatted_string(&Locale::en),
    );

    loop {
        let mut stdout = stdout();
        match rx.recv() {
            // Data recieved from a thread
            Ok((last_index, saved_attempts, i)) => {
                // Update attempts
                attempts += saved_attempts;

                // Check if the thread reached the end of the lyrics
                if last_index == -1 {
                    // The monkey did it!
                    execute!(stdout, Clear(ClearType::CurrentLine), MoveToColumn(1), Print(format!(
						"MONKEY #{} DID IT! All {} letters of the lyrics of \"Never gonna give you up\" were correctly typed. This took {} attempts ({}).",
						i+1,
						num_chars(&SCRIPT.to_string()).to_formatted_string(&Locale::en),
						attempts.to_formatted_string(&Locale::en),
						duration_string(start_time))
					)).ok();
                    exit(0);
                }

                // Check if the thread reached a new best length
                if last_index > best_length {
                    // New best
                    let text = SCRIPT.chars().take(last_index as usize).collect::<String>();
                    execute!(
                        stdout,
                        Clear(ClearType::CurrentLine),
                        MoveToColumn(1),
                        Print(format!(
                            "Monkey #{} got a new best of {} letter(s) on attempt {} ({}):\n{}\n\n",
                            i + 1,
                            num_chars(&text).to_formatted_string(&Locale::en),
                            attempts.to_formatted_string(&Locale::en),
                            duration_string(start_time),
                            text
                        )),
                    )
                    .ok();
                    best_length = last_index;
                }

                let seconds_elapsed = seconds_elapsed(start_time) as u128;

                execute!(
                    stdout,
                    Clear(ClearType::CurrentLine),
                    MoveToColumn(1),
                    Print(format!(
                        "Ran {} attempts in {} ({}/s)",
                        attempts.to_formatted_string(&Locale::en),
                        duration_string(start_time),
                        (attempts
                            / if seconds_elapsed > 0 {
                                seconds_elapsed
                            } else {
                                1
                            })
                        .to_formatted_string(&Locale::en),
                    ))
                )
                .ok();
            }
            Err(_) => (),
        };
    }
}

/// Runs an attempt at typing the lyrics
/// Returns the amount of characters correctly typed
fn run_attempt(chars: &Vec<char>) -> i32 {
    let mut last_index: i32 = -1;
    for char in chars {
        last_index += 1;
        if choose_character().ne(&char) {
            return last_index;
        }
    }

    -1
}

/// Choose a random character from CHARACTERS
fn choose_character() -> char {
    CHARACTERS[fastrand::usize(0..CHARACTERS.len())]
}

/// Get seconds elapsed since start time
fn seconds_elapsed(start_time: SystemTime) -> u64 {
    let end_time = SystemTime::now();
    end_time.duration_since(start_time).unwrap().as_secs()
}

/// Generate duration string
fn duration_string(start_time: SystemTime) -> String {
    let duration = seconds_elapsed(start_time);

    let hours = (duration / 60) / 60;
    let minutes = (duration / 60) % 60;
    let seconds = duration % 60;

    format!("{:02}h {:02}m {:02}s", hours, minutes, seconds)
}

/// Calculate number of characters that are in the character set
fn num_chars(s: &String) -> i32 {
    let mut i = 0;
    for c in s.to_lowercase().chars() {
        if CHARACTERS.contains(&c) {
            i += 1;
        }
    }
    i
}

/// Generate vector of characters in the script
fn gen_char_vec() -> Vec<char> {
    let mut chars = Vec::from_iter(SCRIPT.to_lowercase().chars());
    for (i, char) in chars.clone().iter().enumerate().rev() {
        if !CHARACTERS.contains(&char) {
            chars.remove(i);
        }
    }
    chars
}
