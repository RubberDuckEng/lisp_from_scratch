use rustyline::error::ReadlineError;
use rustyline::{Editor, Result};

mod vm;

fn main() -> Result<()> {
    let mut rl = Editor::<()>::new()?;
    if rl.load_history("history.txt").is_err() {
        println!("No previous history.");
    }
    let scope = vm::Scope::builtin();
    loop {
        let readline = rl.readline(">> ");
        match readline {
            Ok(line) => {
                rl.add_history_entry(line.as_str());
                match vm::parse(&line).and_then(|value| vm::eval(&scope, &value)) {
                    Ok(value) => {
                        println!("{}", vm::to_string(&value));
                    }
                    Err(err) => {
                        println!("Error: {:?}", err);
                    }
                }
            }
            Err(ReadlineError::Interrupted) => {
                println!("CTRL-C");
                break;
            }
            Err(ReadlineError::Eof) => {
                println!("CTRL-D");
                break;
            }
            Err(err) => {
                println!("Error: {:?}", err);
                break;
            }
        }
    }
    rl.save_history("history.txt")
}

#[cfg(test)]
mod test {
    use std::fs;
    use std::str::Lines;

    use crate::vm;

    const INPUT_MARKER: &str = "#input";
    const OUTPUT_MARKER: &str = "#output";
    const TERMINATOR: &str = "==========";

    struct TestCase {
        input: String,
        output: String,
    }

    impl TestCase {
        fn parse(content: &str) -> Vec<TestCase> {
            let mut tests = Vec::new();
            let mut lines = content.lines();

            fn collect_until(lines: &mut Lines, marker: &str) -> String {
                let mut result = Vec::new();
                while let Some(line) = lines.next() {
                    if line == marker {
                        break;
                    }
                    result.push(line);
                }
                result.join("\n")
            }

            while let Some(first_line) = lines.next() {
                assert_eq!(first_line, INPUT_MARKER);
                let input = collect_until(&mut lines, OUTPUT_MARKER);
                let output = collect_until(&mut lines, TERMINATOR);

                tests.push(TestCase { input, output });
            }

            tests
        }
    }

    struct TestSuite {
        name: String,
        tests: Vec<TestCase>,
    }

    impl TestSuite {
        fn read(path: &str) -> TestSuite {
            let content = fs::read_to_string(path).unwrap();
            let tests = TestCase::parse(&content);
            TestSuite {
                name: path.split('/').last().unwrap().to_string(),
                tests,
            }
        }

        fn scan(dir: &str) -> Vec<TestSuite> {
            let mut suites = Vec::new();
            for entry in fs::read_dir(dir).unwrap() {
                let entry = entry.unwrap();
                let path = entry.path();
                if path.is_file() {
                    suites.push(TestSuite::read(path.to_str().unwrap()));
                }
            }
            suites
        }
    }

    #[test]
    fn golden_test() {
        let suites = TestSuite::scan("tests");
        for suite in suites {
            println!("{}: {} test cases", suite.name, suite.tests.len());
            for test in suite.tests {
                let scope = vm::Scope::builtin();
                let result = vm::parse(&test.input).and_then(|value| vm::eval(&scope, &value));
                let output = match result {
                    Ok(value) => vm::to_string(&value),
                    Err(err) => {
                        format!("#error: {:?}", err)
                    }
                };
                assert_eq!(
                    output, test.output,
                    "Test failed: {}: {}",
                    suite.name, test.input
                );
            }
        }
    }
}
