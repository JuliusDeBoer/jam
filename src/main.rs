use std::{env, process, fs};
use anyhow::{self, bail};
use colored::Colorize;
use dialoguer::{Select, MultiSelect};
use lazy_static::lazy_static;
use std::io::Write;
use reqwest::Url;

#[derive(Clone)]
pub struct Project {
    pub name: String,
    pub root: String,
    pub public: String,
    pub index: String,
    pub snippets: String,
    pub queue: Vec<&'static Choice>,
    use_root: bool
}

impl Project {
    pub fn build() {
        todo!();
    }

    pub fn create_dir(&self, path: &str) -> &Project {
        fs::create_dir(format!(".//{}", path)).unwrap();
        return self;
    }

    pub fn create_file(&self, data: &str, path: &str) -> &Project {
        println!("Creating file {}", path.yellow().bold());
        let file = fs::File::create(format!("./{}", path)).unwrap();
        write!(&file, "{}", data).unwrap();
        return self;
    }

    pub fn append_snippet(&mut self, snippet: &str) -> &Project {
        self.snippets.push_str(snippet);
        return self;
    }

    pub fn download_file(&self, url: &str, file: &str) -> &Project {
        let url = Url::parse(url).unwrap();
        let response = reqwest::blocking::get(url).unwrap();
        Project::create_file(self, response.text().unwrap().as_str(), file);
        return self;
    }

    pub fn use_pub(&mut self, use_pub: bool) -> &Project {
        self.use_root = !use_pub;
        return self;
    }
}

struct Module {
    pub prompt: &'static str,
    pub default: Option<&'static str>,
    pub choices: &'static [&'static Choice]
}

// #[derive(Debug)]
struct Choice {
    pub prompt: &'static str,
    // pub exec: Box<dyn Fn(&mut Project) + Send + 'static>
    pub exec: Box<dyn Fn(&mut Project) + Sync>
}

#[derive(Debug)]
struct FileAndDest {
    pub data: &'static str,
    pub dest: &'static str,
    pub download: bool
}

lazy_static! {
static ref PHP_DIRS: Module = Module {
    prompt: "Use php",
    default: Some("No"),
    choices: &[
        &Choice {
            prompt: "Yes",
            exec: Box::new(| project: &mut Project | {
                project.use_pub(false)
                    .create_dir("public")
                    .create_dir("src")
                    .create_dir("config")
                    .public = String::from("public");
            })
        }
    ]
};

static ref PHP: Module = Module {
    prompt: "Php boilerplate",
    default: None,
    choices: &[
        &Choice {
            prompt: "Database",
            exec: Box::new(| project: &mut Project | {
                project.use_pub(false)
                    .create_dir("src")
                    .create_dir("config")
                    .create_file(include_str!("templates/db.php"), "src/db.php")
                    .create_file(include_str!("templates/db_conf.php"), "config/db.php");
            })
        },
        &Choice {
            prompt: "Jwt",
            exec: Box::new(| project: &mut Project | {
                project.use_pub(false)
                    .create_dir("src")
                    .create_dir("config")
                    .create_file(include_str!("templates/jwt.php"), "src/jwt.php")
                    .create_file(include_str!("templates/jwt_conf.php"), "config/jwt.php");
            })
        }
    ]
};

static ref CSS: Module = Module {
    prompt: "Create css file",
    default: Some("No"),
    choices: &[
        &Choice {
            prompt: "style.css",
            exec: Box::new(| project: &mut Project | {
                project.use_pub(true)
                    .create_file(include_str!("templates/style.css"), "style.css")
                    .append_snippet("\t<link rel=\"stylesheet\" href=\"style.css\">\n");
            })
        },
        &Choice {
            prompt: "style.scss",
            exec: Box::new(| project | {
                project.use_pub(true)
                    .create_file("templates/style.css", "style.scss")
                    .append_snippet("\t<link rel=\"stylesheet\" href=\"style.scss\">\n");
            })
        }
    ],
};

static ref CSS_FRAMEWORK: Module = Module {
    prompt: "Use css framework",
    default: None,
    choices: &[
        &Choice {
            prompt: "Bootstrap",
            exec: Box::new(| project: &mut Project | {
                project.use_pub(true)
                    .create_dir("framework")
                    .download_file("https://cdn.jsdelivr.net/npm/bootstrap@5.3.0-alpha1/dist/css/bootstrap.min.css", "framework/bootstrap.min.css")
                    .download_file("https://cdn.jsdelivr.net/npm/bootstrap@5.3.0-alpha1/dist/js/bootstrap.bundle.min.js", "framework/bootstrap.bundle.min.css")
                    .append_snippet(include_str!("templates/bootstrap.html"));
            })
        },
        &Choice {
            prompt: "Tailwind",
            exec: Box::new(| project | {
                project.use_pub(true)
                    .create_dir("framework")
                    .download_file("https://cdn.tailwindcss,com", "framework/cdn.tailwindcss.com")
                    .append_snippet(include_str!("templates/tailwind.html"));
            })
        }
    ],
};


}
fn prompt_module(project: &mut Project, module: Module) -> Result<usize, anyhow::Error> {
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
                // lock.insert(0, module.choices[v - 1]);
                project.queue.push(module.choices[v - 1]);
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
                for index in v.clone() {
                    project.queue.insert(0, module.choices[index]);
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

// fn execute_choice(project: &mut Project, choice: &Choice) -> Option<anyhow::Error> {
//     // if choice.overwrite_public.is_some() {
//     //     project.public = String::from(choice.overwrite_public.unwrap());
//     // }
//     //
//     // if choice.overwrite_index.is_some() {
//     //     project.index = String::from(choice.overwrite_index.unwrap());
//     // }
//     //
//     // if choice.snippet.is_some() {
//     //     project.snippets.push_str(choice.snippet.unwrap());
//     // }
//
//     let path: String = if choice.use_root_dir {
//         project.root.to_string()
//     } else {
//         format!("{}/{}", project.root, project.public)
//     };
//     
//     #[allow(unused_must_use)]
//     for dir in choice.dirs {
//         fs::create_dir(format!("./{}/{}", path, dir));
//     }
//
//     for file in choice.files {
//         if file.download {
//             write_new(&format!("./{}/{}", path, file.dest), download(file.data).as_str());
//         } else {
//             write_new(&format!("./{}/{}", path, file.dest), file.data);
//         }
//     }
//
//     None
// }

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
    println!("Creating file {}", path.yellow().bold());
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
        snippets: String::new(),
        queue: Vec::new(),
        use_root: true
    };

    match prompt_module(&mut project, PHP_DIRS) {
        Ok(v) => {
            if v == 1 {
                if let Err(e) = prompt_module(&mut project, PHP) {
                    return Some(e);
                }
            }
        },
        Err(e) => {
            return Some(e);
        }
    }

    if let Err(e) = prompt_module(&mut project, CSS) {
        return Some(e);
    }

    if let Err(e) = prompt_module(&mut project, CSS_FRAMEWORK) {
        return Some(e);
    }

    if let Err(e) = initialize_project(&project) {
        return Some(e);
    }

    for choice in project.queue.clone() {
        // execute_choice(&mut project, choice);
        (choice.exec)(&mut project);
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
