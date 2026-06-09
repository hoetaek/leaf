use std::process::ExitCode;

fn main() -> ExitCode {
    match leaf::run() {
        Ok(code) => code,
        Err(err) => {
            eprintln!("error: {err:#}");
            ExitCode::FAILURE
        }
    }
}
