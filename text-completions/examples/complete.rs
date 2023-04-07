use dialoguer::{theme::ColorfulTheme, Input};
use text_completions::{EnvCompletion, MultiCompletion, PathCompletion};

fn main() {
    let completion = MultiCompletion::default()
        .with(EnvCompletion::default())
        .with(PathCompletion::default());
    let input: String = Input::with_theme(&ColorfulTheme::default())
        .with_prompt("type a path or environment variable in")
        .completion_with(&completion)
        .interact_text()
        .unwrap();

    println!("input is {input}");
}
