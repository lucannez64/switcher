#[cfg(target_os = "windows")]
static CODE: &str = "code.cmd";
#[cfg(target_os = "linux")]
static CODE: &str = "code";
#[cfg(target_os = "macos")]
static CODE: &str = "code";

fn input(message: &'_ dyn std::fmt::Display) -> crossterm::Result<String> {
    use crossterm::cursor::{RestorePosition, SavePosition};
    use crossterm::execute;
    use crossterm::terminal::{Clear, ClearType};
    use std::io::{self, stdin, Write};
    execute!(io::stdout(), SavePosition)?;
    print!("{}", message);
    io::stdout().flush().unwrap();
    let mut ret = String::new();
    stdin().read_line(&mut ret)?;
    execute!(io::stdout(), RestorePosition, Clear(ClearType::CurrentLine))?;
    Ok(ret)
}

fn keys(projects: &mut Vec<Project>) -> crossterm::Result<()> {
    use crossterm::cursor::MoveTo;
    use crossterm::event::{read, Event, KeyCode, KeyEvent, KeyModifiers};
    use crossterm::execute;
    use crossterm::style::Stylize;
    use crossterm::style::{Color, Print, ResetColor, SetBackgroundColor, SetForegroundColor};
    use crossterm::terminal::{Clear, ClearType, EnterAlternateScreen, LeaveAlternateScreen};
    use std::io::stdout;
    let mut cursor: usize = 0;
    let mut alternater_screen = false;
    loop {
        execute!(stdout(), Clear(ClearType::All)).unwrap();
        if alternater_screen {
            execute!(stdout(), MoveTo(0, 0), Print("Alternate Screen"))?;
        } else {
            for (y, project) in projects.iter().enumerate() {
                execute!(stdout(), MoveTo(0, y as u16))?;
                if cursor == y {
                    execute!(
                        stdout(),
                        SetForegroundColor(Color::Blue),
                        SetBackgroundColor(Color::White),
                        Print(project.name.clone() + &" : ".to_string() + &project.path.clone()),
                        ResetColor,
                    )?;
                } else {
                    println!(
                        "{} : {}\n",
                        project.name.clone(),
                        project.path.clone().italic()
                    )
                }
            }
        }

        match read()? {
            Event::Key(event) => match event {
                KeyEvent {
                    code: KeyCode::Char('q'),
                    modifiers: KeyModifiers::NONE,
                } => {
                    break;
                }
                KeyEvent {
                    code: KeyCode::Char('k'),
                    modifiers: KeyModifiers::NONE,
                } => {
                    if cursor >= 1 {
                        cursor -= 1;
                    };
                }
                KeyEvent {
                    code: KeyCode::Char('j'),
                    modifiers: KeyModifiers::NONE,
                } => {
                    if cursor <= projects.len() - 2 {
                        cursor += 1;
                    };
                }
                KeyEvent {
                    code: KeyCode::Tab,
                    modifiers: KeyModifiers::NONE,
                } => {
                    use std::ops::Not;
                    alternater_screen = alternater_screen.not();
                    if !alternater_screen {
                        execute!(stdout(), LeaveAlternateScreen)?;
                    } else {
                        execute!(stdout(), EnterAlternateScreen)?;
                    }
                }
                KeyEvent {
                    code: KeyCode::Enter,
                    modifiers: KeyModifiers::NONE,
                } => {
                    std::env::set_current_dir(projects[cursor].path.clone())?;
                    std::process::Command::new(CODE).arg("--reuse-window").arg(".").output()?;
                    let path = &std::path::PathBuf::from(projects[cursor].path.clone());
                    let (type_proj,file_path) =
                        type_project(path);
                    compile(
                        type_proj,
                        &std::path::PathBuf::from(projects[cursor].path.clone()),
                        file_path
                    )?;
                    break;
                }
                KeyEvent {
                    code: KeyCode::Char('i'),
                    modifiers: KeyModifiers::NONE,
                } => {
                    let name = input(&"\nName: ".to_string())?
                        .replace("\n", "")
                        .trim()
                        .to_string();
                    let path = input(&"Path: ".to_string())?
                        .replace("\n", "")
                        .trim()
                        .to_string();
                    projects.push(Project {
                        name: name,
                        path: path,
                    });
                }
                KeyEvent {
                    code: KeyCode::Char('d'),
                    modifiers: KeyModifiers::NONE,
                } => {
                    projects.remove(cursor);
                }
                _ => {}
            },
            _ => {}
        }
    }
    Ok(())
}

#[derive(PartialEq)]
enum Languages {
    RUST = 0,
    PYTHON = 1,
    CPP = 2,
    C = 3,
    JAVASCRIPT = 4,
    GO = 5,
    UNKNOWN = 6,
}

fn type_project(path: &std::path::PathBuf) -> (Languages,String) {
    if std::path::Path::new(&path.join("src/main.rs")).exists() {
        return (Languages::RUST, std::path::Path::new(&path.join("src/main.rs")).to_str().unwrap().to_string());
    } else if std::path::Path::new(&path.join("main.go")).exists() {
        return (Languages::GO,std::path::Path::new(&path.join("main.go")).to_str().unwrap().to_string());
    } else if std::path::Path::new(&path.join("main.py")).exists() {
        return (Languages::PYTHON,std::path::Path::new(&path.join("main.py")).to_str().unwrap().to_string());
    } else if std::path::Path::new(&path.join("main.cpp")).exists() {
        return (Languages::CPP,std::path::Path::new(&path.join("main.cpp")).to_str().unwrap().to_string());
    } else if std::path::Path::new(&path.join("main.c")).exists() {
        return (Languages::C,std::path::Path::new(&path.join("main.c")).to_str().unwrap().to_string());
    } else if std::path::Path::new(&path.join("main.js")).exists() {
        return (Languages::JAVASCRIPT,  std::path::Path::new(&path.join("main.js")).to_str().unwrap().to_string());
    } else {
        return (Languages::UNKNOWN, std::path::Path::new(path).to_str().unwrap().to_string());
    }
}

fn compile(language: Languages, path: &std::path::PathBuf, file_path: String) -> std::io::Result<()> {
    use std::process::Command;
    std::env::set_current_dir(path)?;
    match language {
        Languages::RUST => {
            let mut command = Command::new("cargo");
            command.arg("build").arg("--release").output()?;
        }
        Languages::PYTHON => {
            let mut command = Command::new("python");
            command.arg("main.py");
            command.output()?;
        }
        Languages::CPP => {
            let mut command = Command::new("g++");
            command.arg("main.cpp");
            command.output()?;
     }
        Languages::C => {
            let mut command = Command::new("gcc");
            command.arg("main.c");
            command.output()?;
        }
        Languages::JAVASCRIPT => {
            let mut command = Command::new("node");
            command.arg("main.js");
            command.output()?;
        }
        Languages::GO => {
            let mut command = Command::new("go");
            command.arg("build");
            command.output()?;
        }
        Languages::UNKNOWN => {
            println!("Unknown language");
        }
    }
    if language != Languages::UNKNOWN{Command::new(CODE).arg("--reuse-window").arg(file_path).output()?;}
    Ok(())
}

fn set_up() -> crossterm::Result<()> {
    use crossterm::cursor::{Hide, MoveTo};
    use crossterm::execute;
    use crossterm::terminal::{Clear, ClearType, SetTitle};
    use std::io::stdout;

    let mut stdout = stdout();
    execute!(
        stdout,
        Clear(ClearType::All),
        SetTitle("Project Switcher"),
        MoveTo(0, 0),
        Hide
    )?;

    Ok(())
}

fn save(projects: Vec<Project>) {
    use std::env::var;
    use std::fs::OpenOptions;
    use std::io::Write;
    let mut file = OpenOptions::new()
        .write(true)
        .open(var("PROJECTS").unwrap())
        .unwrap();
    let mut s: String = String::new();
    for project in projects {
        s.push_str(&format!("{}{{}}{}\n", project.name, project.path));
    }
    file.write_all(s.as_bytes()).unwrap();
}
#[derive(Clone)]
struct Project {
    name: String,
    path: String,
}

fn parse_line(line: &str) -> Option<Project> {
    let mut iter = line.split("{}");
    let key = iter.next().unwrap().to_string();
    let value = iter.next().unwrap().to_string();
    Some(Project {
        name: key,
        path: value,
    })
}

fn parse_file(path: &str) -> Result<Vec<Project>, std::io::Error> {
    use std::fs::File;
    use std::io::{BufRead, BufReader};
    let mut projects: Vec<Project> = Vec::new();
    let file = File::open(path).unwrap();
    let reader = BufReader::new(file);

    for line in reader.lines() {
        let p = match parse_line(&line?) {
            Some(p) => p,
            None => continue,
        };
        projects.push(p);
    }
    Ok(projects)
}

fn clean_up() -> crossterm::Result<()> {
    use crossterm::cursor::{MoveTo, Show};
    use crossterm::execute;
    use crossterm::terminal::{Clear, ClearType};
    use std::io::stdout;
    execute!(stdout(), Show, Clear(ClearType::All), MoveTo(0, 0))?;
    Ok(())
}

fn main() -> crossterm::Result<()> {
    use std::env::{var, set_var};
    if var("PROJECTS").is_err() {
        set_var("PROJECTS", dirs::config_dir().unwrap().join("switcher").join("PROJECTS"));
        if !std::path::Path::new(&var("PROJECTS").unwrap()).exists(){
            std::fs::create_dir_all(var("PROJECTS").unwrap()).unwrap();
            std::fs::File::create(&var("PROJECTS").unwrap())?;
        }
    }
    let mut projects =
        parse_file(&var("PROJECTS").expect("var PROJECTS is not defined"))?;
    set_up()?;
    keys(&mut projects)?;
    clean_up()?;
    save(projects);
    Ok(())
}
