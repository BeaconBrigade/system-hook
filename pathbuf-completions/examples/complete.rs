use dialoguer::{Input, theme::ColorfulTheme};
use pathbuf_completions::PathBufCompletion;

fn main() {
    let completion = PathBufCompletion::default();
    let input: String = Input::with_theme(&ColorfulTheme::default())
        .with_prompt("type something in")
        .completion_with(&completion)
        .interact_text().unwrap();

    println!("input is {input}");
}