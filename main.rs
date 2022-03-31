use crossterm::{
    cursor::MoveToColumn, execute, style::Print, terminal::Clear, terminal::ClearType,
};
use fastrand::{self, Rng};
use num_format::{Locale, ToFormattedString};
use std::{
    env,
    io::{stdout, Write},
    process::exit,
    sync::mpsc::{sync_channel, Receiver, SyncSender},
    thread,
    time::Instant,
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
    println!();

    let num_threads = env::var("THREADS")
        .unwrap_or_else(|_| String::from("2"))
        .parse::<u16>()
        .unwrap();

    let mut attempts: u64 = 0;
    let mut best_length: u16 = 0;

    let start_time = Instant::now();

    let mut progress_vec: Vec<(Instant, u64)> = vec![(start_time, 0)];

    // For threads sending their results to the main thread
    // Touple of (last_index, saved_attempts, thread_index)
    let (tx, rx) = sync_channel(64) as (SyncSender<(u16, u64, u16)>, Receiver<(u16, u64, u16)>);
    // Spawn threads
    for i in 0..num_threads {
        // Clone transmitter so I can put it in the thread
        let thread_tx = tx.clone();

        thread::spawn(move || {
            // Save the number of attempts that need to be sent to the main thread
            let mut saved_attempts: u64 = 0;

            let chars = gen_char_vec();

            let rng = fastrand::Rng::new();

            loop {
                let last_index = run_attempt(&chars, &rng);
                saved_attempts += 1;
                // Check if the thread should send info to the main thread
                // This should happen if the thread reaches a new best length or every second
                if saved_attempts > 1000000 || last_index > best_length {
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

    let total = num_chars(SCRIPT);

    println!(
        "\nAll monkeys spawned! {} monkeys are typing the lyrics of \"Never gonna give you up\" ({} letters)\n",
        num_threads,
        total.to_formatted_string(&Locale::en),
    );

    loop {
        let mut stdout = stdout();
        if let Ok((last_index, saved_attempts, i)) = rx.recv() {
            // Update attempts
            attempts += saved_attempts;

            // Check if the thread reached a new best length
            if last_index > best_length {
                // New best
                let text = &SCRIPT[..last_index as usize];
                execute!(
                    stdout,
                    Clear(ClearType::CurrentLine),
                    MoveToColumn(1),
                    Print(format!(
                        "Monkey #{} got a new best of {} letter(s) on attempt {} ({}):\n{}\n\n",
                        i + 1,
                        num_chars(text).to_formatted_string(&Locale::en),
                        attempts.to_formatted_string(&Locale::en),
                        duration_string(start_time),
                        text
                    )),
                )
                .ok();
                best_length = last_index;
            }

            // Check if the thread reached the end of the lyrics
            if last_index == total {
                // The monkey did it!
                execute!(stdout, Clear(ClearType::CurrentLine), MoveToColumn(1), Print(format!(
        						"All {} letters of the lyrics of \"Never gonna give you up\" were correctly typed! This took {} attempts ({}).",
        						num_chars(SCRIPT).to_formatted_string(&Locale::en),
        						attempts.to_formatted_string(&Locale::en),
        						duration_string(start_time))
        					)).ok();
                stdout.flush().ok();
                exit(0);
            }

            // Remove elements older than 5 minutes
            progress_vec.retain(|&(time, _)| time.elapsed().as_secs() < 60);

            if progress_vec.last().unwrap().0.elapsed().as_secs() >= 1 {
                // Record new progress
                progress_vec.push((Instant::now(), attempts));
            }

            let first_progress = progress_vec.first().unwrap();

            let seconds_elaped = first_progress.0.elapsed().as_secs() as u64;
            let attempts_elapsed = attempts - first_progress.1;
            let attemps_per_second = if seconds_elaped > 0 {
                attempts_elapsed / seconds_elaped
            } else {
                0
            };

            execute!(
                stdout,
                Clear(ClearType::CurrentLine),
                MoveToColumn(1),
                Print(format!(
                    "Ran {} attempts in {} ({}/s)",
                    sig_figs(attempts, 3, 11).to_formatted_string(&Locale::en),
                    duration_string(start_time),
                    sig_figs(attemps_per_second, 3, 0).to_formatted_string(&Locale::en),
                ))
            )
            .ok();
        };
    }
}

/// Runs an attempt at typing the lyrics
/// Returns the amount of characters correctly typed
fn run_attempt(chars: &[char], rng: &Rng) -> u16 {
    let mut last_index: u16 = 0;
    for char in chars {
        if choose_character(rng).ne(char) {
            break;
        }
        last_index += 1;
    }

    last_index
}

/// Choose a random character from CHARACTERS
fn choose_character(rng: &Rng) -> char {
    CHARACTERS[rng.usize(0..CHARACTERS.len())]
}

/// Generate duration string
fn duration_string(start_time: Instant) -> String {
    let duration = start_time.elapsed().as_secs();

    let hours = (duration / 60) / 60;
    let minutes = (duration / 60) % 60;
    let seconds = duration % 60;

    format!("{:02}h {:02}m {:02}s", hours, minutes, seconds)
}

/// Calculate number of characters that are in the character set
fn num_chars(s: &str) -> u16 {
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
        if !CHARACTERS.contains(char) {
            chars.remove(i);
        }
    }
    chars
}

/// Round number to significant figures
fn sig_figs<T: Into<u64>>(number: T, sig_figs: u32, max_digits: u32) -> u64 {
    let number: u64 = number.into();
    let mut digits = number.to_string().len() as u32;
    if max_digits != 0 && digits > max_digits {
        digits = max_digits;
    }
    let divide_by = 10u64.pow(digits - sig_figs);
    if divide_by == 0 {
        return number;
    }

    (number / divide_by) * divide_by
}
