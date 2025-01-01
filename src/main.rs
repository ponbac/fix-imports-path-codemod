use clap::Parser;
use std::fs;
use std::io::Write;
use std::path::{Path, PathBuf};
use walkdir::WalkDir;

#[derive(Parser, Debug)]
#[command(author, version, about, long_about = None)]
struct Args {
    /// The root path to search for .ts(x) files from
    #[arg(default_value = ".")]
    path: PathBuf,
}

fn main() {
    let args = Args::parse();

    for entry in WalkDir::new(dbg!(args.path))
        .into_iter()
        .filter_entry(|e| !is_node_modules(e))
        .filter_map(|e| e.ok())
        .filter(is_ts_file)
    {
        let path = entry.path();
        let content = fs::read_to_string(path).unwrap();
        let mut modified = false;

        // Process the file line by line, collecting the possibly modified lines
        let new_content: Vec<String> = content
            .lines()
            .map(|line| {
                if line.starts_with("import") {
                    let depth = import_depth(line);
                    let src_dist = src_distance(path);

                    if depth > 0 && depth == src_dist {
                        modified = true;
                        let cleaned_line = {
                            let parts: Vec<&str> = line.split("'").collect();
                            let path = parts[1].replace("../", "");
                            format!("{}'@/{}'{}", parts[0], path, parts[2])
                        };
                        println!("{} --> {}", line, cleaned_line);
                        return cleaned_line;
                    }
                }
                line.to_string()
            })
            .collect();

        if modified {
            let mut file = fs::File::create(path).unwrap();
            write!(file, "{}", new_content.join("\n")).unwrap();
        }
    }
}

fn is_node_modules(entry: &walkdir::DirEntry) -> bool {
    entry.file_name() == "node_modules"
}

fn is_ts_file(entry: &walkdir::DirEntry) -> bool {
    entry
        .path()
        .extension()
        .map_or(false, |ext| ext == "ts" || ext == "tsx")
}

fn import_depth(line: &str) -> usize {
    let path = line.split_whitespace().last().unwrap();
    let depth = path.matches("../").count();
    depth
}

fn src_distance(path: &Path) -> usize {
    let mut depth = 0;
    let mut current = path;
    while let Some(parent) = current.parent() {
        if parent.file_name().map_or(false, |name| name == "src") {
            break;
        }
        current = parent;
        depth += 1;
    }
    depth
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::path::Path;

    #[test]
    fn test_depth_from_src_dir() {
        let path = Path::new("C:/Users/ponbac/Dev/spinit/hexagon/src/app/src/components/test.ts");
        assert_eq!(src_distance(path), 1);

        let path =
            Path::new("C:/Users/ponbac/Dev/spinit/hexagon/src/app/src/components/nested/test.ts");
        assert_eq!(src_distance(path), 2);

        let path = Path::new("C:/Users/ponbac/Dev/spinit/hexagon/src/test.ts");
        assert_eq!(src_distance(path), 0);
    }

    #[test]
    fn test_import_depth() {
        let line = "import { ConfirmRemoveModal } from '../../../../../../../Components/ConfirmRemoveModal/ConfirmRemoveModal';";
        assert_eq!(import_depth(line), 7);
    }
}
