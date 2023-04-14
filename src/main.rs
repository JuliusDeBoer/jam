use std::{env, process, fs};
use std::sync::Mutex;
use anyhow::{self, bail};
use dialoguer::{Select, MultiSelect};
use std::io::Write;
use reqwest::Url;

static QUEUE: Mutex<Vec<&Choice>> = Mutex::new(Vec::new());

#[derive(Clone)]
struct Project {
    pub name: String,
    pub root: String,
    pub public: String,
    pub index: String,
    pub snippets: String
}

struct Module {
    pub prompt: &'static str,
    pub default: Option<&'static str>,
    pub choices: &'static [&'static Choice],
}

#[derive(Debug)]
struct Choice {
    pub prompt: &'static str,
    pub dirs: &'static [&'static str],
    pub files: &'static [&'static FileAndDest],
    pub use_root_dir: bool,
    pub snippet: Option<&'static str>,
    pub overwrite_public: Option<&'static str>,
    pub overwrite_index: Option<&'static str>
}

#[derive(Debug)]
struct FileAndDest {
    pub data: &'static str,
    pub dest: &'static str,
    pub download: bool
}

const PHP_DIRS: Module = Module {
    prompt: "Use php",
    default: Some("No"),
    choices: &[
        &Choice {
            prompt: "Yes",
            dirs: &["public", "src", "config"],
            files: &[],
            use_root_dir: true,
            snippet: None,
            overwrite_public: Some("public"),
            overwrite_index: None
        }
    ]
};

const PHP: Module = Module {
    prompt: "Php boilerplate",
    default: None,
    choices: &[
        &Choice {
            prompt: "Database",
            dirs: &["src", "config"],
            files: &[
                &FileAndDest {
                    data: include_str!("templates/db.php"),
                    dest: "src/db.php",
                    download: false
                },
                &FileAndDest {
                    data: include_str!("templates/db_conf.php"),
                    dest: "config/db.php",
                    download: false
                }
            ],
            use_root_dir: true,
            snippet: None,
            overwrite_public: None,
            overwrite_index: None

        },
        &Choice {
            prompt: "Jwt",
            dirs: &["src", "config"],
            files: &[
                &FileAndDest {
                    data: include_str!("templates/jwt.php"),
                    dest: "src/jwt.php",
                    download: false
                },
                &FileAndDest {
                    data: include_str!("templates/jwt_conf.php"),
                    dest: "config/jwt.php",
                    download: false
                }
            ],
            use_root_dir: true,
            snippet: None,
            overwrite_public: None,
            overwrite_index: None

        }
    ]
};

const CSS: Module = Module {
    prompt: "Create css file",
    default: Some("No"),
    choices: &[
        &Choice {
            prompt: "style.css",
            dirs: &[],
            files: &[
                &FileAndDest {
                    data: include_str!("templates/style.css"),
                    dest: "style.css",
                    download: false
                }
            ],
            use_root_dir: false,
            snippet: Some("\t<link rel=\"stylesheet\" href=\"style.css\">\n"),
            overwrite_public: None,
            overwrite_index: None

        },
        &Choice {
            prompt: "style.scss",
            dirs: &[],
            files: &[
                &FileAndDest {
                    data: include_str!("templates/style.css"),
                    dest: "style.scss",
                    download: false
                }
            ],
            use_root_dir: false,
            snippet: Some("\t<link rel=\"stylesheet\" href=\"style.scss\">\n"),
            overwrite_public: None,
            overwrite_index: None

        }
    ],
};

const CSS_FRAMEWORK: Module = Module {
    prompt: "Use css framework",
    default: None,
    choices: &[
        &Choice {
            prompt: "Bootstrap",
            dirs: &["framework"],
            files: &[
                &FileAndDest {
                    data: "https://cdn.jsdelivr.net/npm/bootstrap@5.3.0-alpha1/dist/css/bootstrap.min.css",
                    dest: "framework/bootstrap.min.css",
                    download: true
                },
                &FileAndDest {
                    data: "https://cdn.jsdelivr.net/npm/bootstrap@5.3.0-alpha1/dist/js/bootstrap.bundle.min.js",
                    dest: "framework/bootstrap.bundle.min.js",
                    download: true,
                }
            ],
            use_root_dir: false,
            snippet: Some(include_str!("templates/bootstrap.html")),
            overwrite_public: None,
            overwrite_index: None

        },
        &Choice {
            prompt: "Tailwind",
            dirs: &["framework"],
            files: &[
                &FileAndDest {
                    data: "https://cdn.tailwindcss.com",
                    dest: "framework/tailwind.js",
                    download: true
                }
            ],
            use_root_dir: false,
            snippet: Some(include_str!("templates/tailwind.html")),
            overwrite_public: None,
            overwrite_index: None
        }
    ],
};

fn prompt_module(module: Module) -> Result<usize, anyhow::Error> {
    if module.default.is_some() {
        let mut select = Select::new();
        select.with_prompt(module.prompt);
        select.default(0);
        select.item(module.default.unwrap());
        for choice in module.choices {
            select.item(choice.prompt);
        }
        match select.interact() {
            Ok(v) => {
                if v == 0 { return Ok(0); };
                let mut lock = QUEUE.lock().unwrap();
                // lock.insert(0, module.choices[v - 1]);
                lock.push(module.choices[v - 1]);
                Ok(v)
            },
            Err(e) => Err(anyhow::format_err!(e))
        }
    } else {
        let mut multi_select = MultiSelect::new();
        multi_select.with_prompt(module.prompt);
        for choice in module.choices {
            multi_select.item(choice.prompt);
        }
        match multi_select.interact() {
            Ok(v) => {
                let mut lock = QUEUE.lock().unwrap();
                for index in v.clone() {
                    lock.insert(0, module.choices[index]);
                }

                if v.is_empty() {
                    Ok(1)
                } else {
                    Ok(0)
                }
            },
            Err(e) => Err(anyhow::format_err!(e))
        }
    }
}

fn execute_choice(project: &mut Project, choice: &Choice) -> Option<anyhow::Error> {
    if choice.overwrite_public.is_some() {
        project.public = String::from(choice.overwrite_public.unwrap());
    }

    if choice.overwrite_index.is_some() {
        project.index = String::from(choice.overwrite_index.unwrap());
    }

    if choice.snippet.is_some() {
        project.snippets.push_str(choice.snippet.unwrap());
    }

    let path: String = if choice.use_root_dir {
        project.root.to_string()
    } else {
        format!("{}/{}", project.root, project.public)
    };
    
    #[allow(unused_must_use)]
    for dir in choice.dirs {
        fs::create_dir(format!("./{}/{}", path, dir));
    }

    for file in choice.files {
        if file.download {
            write_new(&format!("./{}/{}", path, file.dest), download(file.data).as_str());
        } else {
            write_new(&format!("./{}/{}", path, file.dest), file.data);
        }
    }

    None
}

fn initialize_project(project: &Project) -> Result<&Project, anyhow::Error> {
    match fs::create_dir(project.root.as_str()) {
        Ok(_) => Ok(project),
        Err(e) => bail!(e)
    }
}

fn finalize_project(project: &Project) {
    write_new(&format!("./{}/{}/{}", project.root, project.public, project.index), &format!(include_str!("templates/index.html"), project.name, project.snippets));
}

fn write_new(path: &String, data: &str) {
    println!("Creating file: ./{}", path);
    let file = fs::File::create(format!("./{}", path)).unwrap();
    write!(&file, "{}", data).unwrap();
}

fn download(url: &'static str) -> String {
    let url = Url::parse(url).unwrap();
    let response = reqwest::blocking::get(url).unwrap();
    response.text().unwrap()
}

fn new(name: &str) -> Option<anyhow::Error> {
    let mut project = Project {
        name: String::from(name),
        root: String::from(name),
        public: String::new(),
        index: String::from("index.html"),
        snippets: String::new()
    };

    match prompt_module(PHP_DIRS) {
        Ok(v) => {
            if v == 1 {
                if let Err(e) = prompt_module(PHP) {
                    return Some(e);
                }
            }
        },
        Err(e) => {
            return Some(e);
        }
    }

    if let Err(e) = prompt_module(CSS) {
        return Some(e);
    }

    if let Err(e) = prompt_module(CSS_FRAMEWORK) {
        return Some(e);
    }

    if let Err(e) = initialize_project(&project) {
        return Some(e);
    }

    let queue = QUEUE.lock().unwrap();

    for choice in queue.clone() {
        // This is gonna go wrong. To bad
        execute_choice(&mut project, choice);
    }

    finalize_project(&project);

    None
}

fn main() {
    let args: Vec<String> = env::args().collect();
    if args.len() < 2 {
        eprintln!("No arguments given");
        process::exit(1);
    }
    if args[1] == "new" {
        if args.len() < 3 {
            eprintln!("No name given");
            process::exit(1);
        }

        if let Some(e) = new(&args[2]) {
            eprintln!("{}", e);
        }
    }
}
