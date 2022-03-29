use num_format::{Locale, ToFormattedString};
use rand::{self, Rng};
use std::{
    process::exit,
    sync::mpsc::{sync_channel, Receiver, SyncSender},
    thread,
    time::SystemTime,
};

const CHARACTERS: &str = "abcdefghijklmnopqrstuvwxyz";

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

const NUM_THREADS: i32 = 8;

fn main() {
    println!("");

    let mut attempts: u128 = 0;
    let mut best_length: i32 = 0;

    let start_time = SystemTime::now();

    // For threads sending their results to the main thread
    // Touple of (last_index, saved_attempts, thread_index)
    let (tx, rx) = sync_channel(64) as (SyncSender<(i32, u128, i32)>, Receiver<(i32, u128, i32)>);
    // Spawn threads
    for i in 0..NUM_THREADS {
        // Clone transmitter so I can put it in the thread
        let thread_tx = tx.clone();

        thread::spawn(move || {
            // Save the number of attempts that need to be sent to the main thread
            let mut saved_attempts: u128 = 0;
            loop {
                let last_index = run_attempt();
                saved_attempts += 1;
                // Check if the thread should send info to the main thread
                // This should happen if the thread reaches a new best length or every 1 million attempts
                if last_index > best_length || last_index == -1 || saved_attempts >= 1000000 {
                    if last_index > best_length {
                        // Update saved best length
                        best_length = last_index;
                    };

                    thread_tx.send((last_index, saved_attempts, i)).ok();

                    // Reset saved attempts
                    saved_attempts = 0;
                }
            }
        });

        println!("Spawned monkey #{}", i + 1);
    }

    println!(
        "\nAll monkeys spawned! {} monkeys are typing the lyrics of \"Never gonna give you up\" ({} characters)\n",
        NUM_THREADS,
        SCRIPT.len().to_formatted_string(&Locale::en),
    );

    loop {
        match rx.recv() {
            // Data recieved from a thread
            Ok((last_index, saved_attempts, i)) => {
                // Update attempts
                attempts += saved_attempts;

                // Check if the thread reached the end of the lyrics
                if last_index == -1 {
                    // The monkey did it!
                    println!("MONKEY {} DID IT! All {} characters of the lyrics of \"Never gonna give you up\" were correctly typed. This took {} attempts ({}).",
					i+1,
					SCRIPT.len().to_formatted_string(&Locale::en),
					attempts.to_formatted_string(&Locale::en),
					duration_string(start_time));
                    exit(0);
                }

                // Check if the thread reached a new best length
                if last_index > best_length {
                    // New best
                    println!(
                        "Monkey {} got a new best of {} characters on attempt {} ({}):\n{}\n",
                        i + 1,
                        last_index.to_formatted_string(&Locale::en),
                        attempts.to_formatted_string(&Locale::en),
                        duration_string(start_time),
                        SCRIPT.chars().take(last_index as usize).collect::<String>()
                    );
                    best_length = last_index;
                }
            }
            Err(_) => (),
        };
    }
}

/// Runs an attempt at typing the lyrics
/// Returns the amount of characters correctly typed, or -1 if it was completed
fn run_attempt() -> i32 {
    let mut last_index: i32 = -1;
    for (i, char) in SCRIPT.to_lowercase().chars().enumerate() {
        if CHARACTERS.contains(char) && choose_character().ne(&char.to_string()) {
            last_index = i as i32;
            break;
        }
    }

    last_index
}

/// Choose a random character from CHARACTERS
fn choose_character() -> String {
    let mut rng = rand::thread_rng();
    CHARACTERS
        .chars()
        .nth(rng.gen_range(0..CHARACTERS.len()))
        .unwrap()
        .to_string()
}

/// Generate duration string
fn duration_string(start_time: SystemTime) -> String {
    let end_time = SystemTime::now();
    let duration = end_time.duration_since(start_time).unwrap().as_secs();

    let hours = (duration / 60) / 60;
    let minutes = (duration / 60) % 60;
    let seconds = duration % 60;

    format!("{}h {}m {}s", hours, minutes, seconds)
}
