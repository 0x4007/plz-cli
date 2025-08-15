use std::process::Command;
use std::collections::{HashSet, HashMap};
use std::path::{Path, PathBuf};
use std::fs;
use std::time::SystemTime;

pub fn get_system_context() -> String {
    let mut context = String::new();
    
    // OS information
    context.push_str(&format!("OS: {} {}\n", 
        std::env::consts::OS, 
        std::env::consts::ARCH
    ));
    
    // Shell information
    if let Ok(shell) = std::env::var("SHELL") {
        context.push_str(&format!("SHELL={}\n", shell));
    }
    
    // macOS version if on macOS
    if cfg!(target_os = "macos") {
        if let Ok(output) = Command::new("sw_vers").arg("-productVersion").output() {
            let version = String::from_utf8_lossy(&output.stdout).trim().to_string();
            context.push_str(&format!("macOS version: {}\n", version));
        }
    }
    
    // Linux distribution if on Linux
    if cfg!(target_os = "linux") {
        if let Ok(contents) = std::fs::read_to_string("/etc/os-release") {
            if let Some(line) = contents.lines().find(|l| l.starts_with("PRETTY_NAME=")) {
                let distro = line.trim_start_matches("PRETTY_NAME=").trim_matches('"');
                context.push_str(&format!("Linux distribution: {}\n", distro));
            }
        }
    }
    
    // Get environment variable names only - no values for security
    let mut env_vars: Vec<String> = std::env::vars()
        .map(|(key, _)| key)
        .collect();
    env_vars.sort();
    
    if !env_vars.is_empty() {
        context.push_str(&format!("Environment variables: {}\n", env_vars.join(", ")));
    }
    
    // Get binaries from PATH environment variable
    let mut all_binaries = HashSet::new();
    
    if let Ok(path_var) = std::env::var("PATH") {
        for path_dir in path_var.split(':') {
            if let Ok(entries) = std::fs::read_dir(path_dir) {
                for entry in entries.flatten() {
                    if let Ok(metadata) = entry.metadata() {
                        if metadata.is_file() {
                            // Check if file is executable
                            #[cfg(unix)]
                            {
                                use std::os::unix::fs::PermissionsExt;
                                if metadata.permissions().mode() & 0o111 != 0 {
                                    if let Ok(file_name) = entry.file_name().into_string() {
                                        all_binaries.insert(file_name);
                                    }
                                }
                            }
                            #[cfg(not(unix))]
                            {
                                if let Ok(file_name) = entry.file_name().into_string() {
                                    all_binaries.insert(file_name);
                                }
                            }
                        }
                    }
                }
            }
        }
    }
    
    let mut sorted_binaries: Vec<String> = all_binaries.into_iter().collect();
    sorted_binaries.sort();
    
    // Limit to reasonable size for context (first 200 or so)
    if sorted_binaries.len() > 200 {
        sorted_binaries.truncate(200);
        sorted_binaries.push("...".to_string());
    }
    
    if !sorted_binaries.is_empty() {
        context.push_str(&format!("Available commands: {}\n", sorted_binaries.join(", ")));
    }
    
    // Add current directory context
    if let Ok(current_dir) = std::env::current_dir() {
        context.push_str(&format!("\nCurrent directory: {}\n", current_dir.display()));
        
        // Get intelligent directory tree
        let tree = get_intelligent_directory_tree(&current_dir);
        if !tree.is_empty() {
            context.push_str("Directory structure:\n");
            context.push_str(&tree);
        }
    }
    
    context
}

fn get_intelligent_directory_tree(root_path: &Path) -> String {
    let mut tree = String::new();
    let mut dir_stats = analyze_directory_structure(root_path);
    
    // Build tree based on analyzed structure
    build_tree_from_stats(root_path, &mut tree, 0, 3, &mut dir_stats, &mut HashSet::new());
    
    tree
}

fn calculate_file_count_priority(file_count: usize) -> i32 {
    // Smooth exponential decay function
    // High priority for empty/small dirs, smoothly decreasing for larger dirs
    let count = file_count as f64;
    let priority = 20.0 * (-count / 50.0).exp();
    priority as i32
}

fn analyze_directory_structure(path: &Path) -> HashMap<PathBuf, DirStats> {
    let mut stats_map = HashMap::new();
    analyze_dir_recursive(path, &mut stats_map, 0, 5);
    stats_map
}

struct DirStats {
    total_size: u64,
    file_count: usize,
    dir_count: usize,
    last_modified: SystemTime,
    has_interesting_files: bool,
    is_likely_generated: bool,
    modification_times: Vec<SystemTime>,
    has_time_clustering: bool,
}

fn analyze_dir_recursive(path: &Path, stats_map: &mut HashMap<PathBuf, DirStats>, depth: usize, max_depth: usize) {
    if depth > max_depth {
        return;
    }
    
    let mut stats = DirStats {
        total_size: 0,
        file_count: 0,
        dir_count: 0,
        last_modified: SystemTime::UNIX_EPOCH,
        has_interesting_files: false,
        is_likely_generated: false,
        modification_times: Vec::new(),
        has_time_clustering: false,
    };
    
    if let Ok(entries) = fs::read_dir(path) {
        for entry in entries.flatten() {
            if let Ok(metadata) = entry.metadata() {
                if metadata.is_dir() {
                    stats.dir_count += 1;
                    let subpath = entry.path();
                    analyze_dir_recursive(&subpath, stats_map, depth + 1, max_depth);
                } else {
                    stats.file_count += 1;
                    stats.total_size += metadata.len();
                    
                    if let Ok(modified) = metadata.modified() {
                        stats.modification_times.push(modified);
                        if modified > stats.last_modified {
                            stats.last_modified = modified;
                        }
                    }
                    
                    // Check if directory has interesting files
                    if let Some(name) = entry.file_name().to_str() {
                        // User-created files often have meaningful names
                        if name.contains("README") || name.contains("LICENSE") ||
                           name.contains("config") || name.contains("main") ||
                           !name.contains('.') || // files without extensions might be scripts
                           (name.len() > 3 && !name.starts_with('.')) {
                            stats.has_interesting_files = true;
                        }
                    }
                }
            }
        }
        
        // Check for time clustering - even 2 files with exact same timestamp is suspicious
        if stats.modification_times.len() >= 2 {
            stats.modification_times.sort();
            let mut time_clusters: HashMap<u64, usize> = HashMap::new();
            
            for time in &stats.modification_times {
                if let Ok(duration) = time.duration_since(SystemTime::UNIX_EPOCH) {
                    let seconds = duration.as_secs();
                    *time_clusters.entry(seconds).or_insert(0) += 1;
                }
            }
            
            // Check for suspicious clustering patterns
            let max_cluster = time_clusters.values().max().unwrap_or(&0);
            let total_files = stats.modification_times.len();
            let cluster_ratio = *max_cluster as f64 / total_files as f64;
            
            // More nuanced detection:
            // 1. If ALL files have exact same timestamp, very likely auto-generated
            // 2. If most files (>80%) have same timestamp AND there are many files, likely auto-generated
            // 3. Small directories with 2-3 files at same time could be legitimate user actions
            
            if cluster_ratio >= 0.95 && total_files >= 5 {
                // Nearly all files have same timestamp - very suspicious
                stats.has_time_clustering = true;
            } else if cluster_ratio >= 0.8 && total_files >= 10 {
                // Most files clustered and many files - suspicious
                stats.has_time_clustering = true;
            } else if *max_cluster >= 20 {
                // Any directory with 20+ files at exact same second - definitely auto-generated
                stats.has_time_clustering = true;
            }
        }
        
        // Heuristics to detect generated/dependency directories
        let dir_name = path.file_name().and_then(|n| n.to_str()).unwrap_or("");
        stats.is_likely_generated = is_likely_generated_dir(&dir_name, &stats) || stats.has_time_clustering;
    }
    
    stats_map.insert(path.to_path_buf(), stats);
}

fn is_likely_generated_dir(name: &str, stats: &DirStats) -> bool {
    // Common patterns for generated directories
    let generated_patterns = [
        // Build/dist directories
        "node_modules", "vendor", "target", "dist", "build", "out",
        "output", "artifacts", "bundle", "public", "_site",
        
        // Python
        "__pycache__", ".pytest_cache", ".mypy_cache", ".tox",
        "venv", ".venv", "env", ".env", "virtualenv",
        
        // JS/Web
        ".cache", ".next", ".nuxt", ".output", ".parcel-cache",
        "coverage", ".nyc_output", ".turbo",
        
        // Package managers
        "bower_components", ".pnpm", ".yarn",
        
        // IDE/Tools
        ".idea", ".vscode", ".vs", ".gradle", ".mvn",
    ];
    
    // Check exact matches (case-insensitive for some patterns)
    let name_lower = name.to_lowercase();
    if generated_patterns.iter().any(|&pattern| {
        name == pattern || name_lower == pattern.to_lowercase()
    }) {
        return true;
    }
    
    // Check if name ends with common build suffixes
    if name_lower.ends_with("_build") || name_lower.ends_with("-build") ||
       name_lower.ends_with("_dist") || name_lower.ends_with("-dist") ||
       name_lower.ends_with("_output") || name_lower.ends_with("-output") {
        return true;
    }
    
    // Heuristics based on content
    // If a directory has many files (>100) and no interesting files, it's likely generated
    if stats.file_count > 100 && !stats.has_interesting_files {
        return true;
    }
    
    // If directory name starts with . and has many files, likely cache
    if name.starts_with('.') && stats.file_count > 20 {
        return true;
    }
    
    false
}

fn build_tree_from_stats(
    path: &Path,
    tree: &mut String,
    depth: usize,
    max_depth: usize,
    stats_map: &HashMap<PathBuf, DirStats>,
    visited: &mut HashSet<PathBuf>
) {
    if depth >= max_depth || visited.contains(path) {
        return;
    }
    visited.insert(path.to_path_buf());
    
    let indent = "  ".repeat(depth);
    
    struct Entry {
        name: String,
        path: PathBuf,
        is_dir: bool,
        priority: i32,
    }
    
    let mut entries = Vec::new();
    
    if let Ok(dir_entries) = fs::read_dir(path) {
        for entry in dir_entries.flatten() {
            if let Ok(name) = entry.file_name().into_string() {
                let entry_path = entry.path();
                let is_dir = entry_path.is_dir();
                
                let mut priority = 0;
                
                if is_dir {
                    if let Some(stats) = stats_map.get(&entry_path) {
                        // Mark generated directories (at any depth)
                        if stats.is_likely_generated {
                            // Still add the directory but with very low priority
                            entries.push(Entry {
                                name,
                                path: entry_path.clone(),
                                is_dir: true,
                                priority: -100, // Very low priority - will show with ... inside
                            });
                            continue;
                        }
                        
                        // Prioritize directories with interesting content
                        if stats.has_interesting_files {
                            priority += 10;
                        }
                        
                        // Continuous priority based on file count (smaller = higher priority)
                        priority += calculate_file_count_priority(stats.file_count);
                        
                        // Deprioritize if has time clustering (even if not filtered)
                        if stats.has_time_clustering {
                            priority -= 20;
                        }
                    }
                } else {
                    // For files, prioritize based on name patterns
                    if name.contains("README") || name.contains("LICENSE") {
                        priority += 20;
                    } else if name.contains("config") || name.contains("main") {
                        priority += 15;
                    } else if name.ends_with(".md") || name.ends_with(".txt") {
                        priority += 10;
                    } else if !name.starts_with('.') {
                        priority += 5;
                    }
                }
                
                entries.push(Entry {
                    name,
                    path: entry_path,
                    is_dir,
                    priority,
                });
            }
        }
    }
    
    // Sort by priority (higher first), then directories, then name
    entries.sort_by(|a, b| {
        match b.priority.cmp(&a.priority) {
            std::cmp::Ordering::Equal => {
                match (a.is_dir, b.is_dir) {
                    (true, false) => std::cmp::Ordering::Less,
                    (false, true) => std::cmp::Ordering::Greater,
                    _ => a.name.cmp(&b.name),
                }
            }
            other => other,
        }
    });
    
    // No longer filter out auto-generated entries - we'll show them with ellipses
    
    // Dynamic limit based on depth and content
    let max_entries = if depth == 0 { 30 } else { 15 };
    let total_entries = entries.len();
    
    for (i, entry) in entries.iter().take(max_entries).enumerate() {
        let is_last = i == entries.len() - 1 || i == max_entries - 1;
        let prefix = if is_last { "└──" } else { "├──" };
        
        if entry.is_dir {
            tree.push_str(&format!("{}{} {}/\n", indent, prefix, entry.name));
            
            // For auto-generated directories, just show ellipsis
            if entry.priority == -100 {
                let sub_indent = "  ".repeat(depth + 1);
                tree.push_str(&format!("{}└── ...\n", sub_indent));
            } else {
                // Recurse into normal directories
                build_tree_from_stats(
                    &entry.path,
                    tree,
                    depth + 1,
                    max_depth,
                    stats_map,
                    visited
                );
            }
        } else {
            tree.push_str(&format!("{}{} {}\n", indent, prefix, entry.name));
        }
    }
    
    if total_entries > max_entries {
        tree.push_str(&format!("{}└── ... ({} more items)\n", indent, total_entries - max_entries));
    }
}