use clap::Parser;
use std::collections::HashMap;
use std::fs;
use std::io::{self, Read, Write};
use std::path::PathBuf;

/// Convert a Markdown list into a Mermaid flowchart.
///
/// Node ID naming rules:
///   Root nodes : N1, N2, N3, ...
///   Children   : {parentId}_{siblingIndex}   e.g. N1_2_1
///   depth      = number of underscores + 1
///   parent     = drop the last _{n} segment
#[derive(Parser, Debug)]
#[command(name = "md2mermaid", version = "1.0.0", about, long_about = None)]
struct Cli {
    /// Input Markdown file (omit to read from stdin)
    input: Option<PathBuf>,

    /// Flow direction: LR (default), TD, RL, BT
    #[arg(short, long, default_value = "LR", value_parser = parse_direction)]
    direction: String,

    /// Write output to file instead of stdout
    #[arg(short, long)]
    output: Option<PathBuf>,

    /// Print node ID legend after the diagram
    #[arg(long)]
    legend: bool,
}

fn parse_direction(s: &str) -> Result<String, String> {
    let upper = s.to_uppercase();
    match upper.as_str() {
        "LR" | "TD" | "TB" | "RL" | "BT" => Ok(upper),
        _ => Err(format!(
            "'{}' is not a valid direction. Use LR, TD, RL, or BT.",
            s
        )),
    }
}

#[derive(Debug, Clone)]
struct Node {
    id: String,
    label: String,
    depth: usize,
    parent_id: Option<String>,
}

#[derive(Debug, Clone)]
struct Edge {
    from: String,
    to: String,
}

fn strip_marker(s: &str) -> &str {
    let trimmed = s.trim_start();
    let after = if trimmed.starts_with('-') || trimmed.starts_with('*') {
        &trimmed[1..]
    } else {
        let dot = trimmed.find('.').unwrap_or(0);
        if dot > 0 && trimmed[..dot].chars().all(|c| c.is_ascii_digit()) {
            &trimmed[dot + 1..]
        } else {
            trimmed
        }
    };
    after.trim()
}

fn indent_of(line: &str) -> usize {
    line.chars()
        .take_while(|c| *c == ' ' || *c == '\t')
        .map(|c| if c == '\t' { 2 } else { 1 })
        .sum()
}

fn parse_list(text: &str) -> (Vec<Node>, Vec<Edge>) {
    let mut nodes: Vec<Node> = Vec::new();
    let mut edges: Vec<Edge> = Vec::new();
    let mut stack: Vec<(usize, String)> = Vec::new();
    let mut sibling_count: HashMap<String, usize> = HashMap::new();

    for line in text.lines() {
        if line.trim().is_empty() {
            continue;
        }
        let indent = indent_of(line);
        let label = strip_marker(line);
        if label.is_empty() {
            continue;
        }

        while let Some((top_indent, _)) = stack.last() {
            if *top_indent >= indent {
                stack.pop();
            } else {
                break;
            }
        }

        let parent_id: Option<String> = stack.last().map(|(_, id)| id.clone());
        let depth = stack.len() + 1;
        let sib_key = parent_id.clone().unwrap_or_else(|| "__root__".into());
        let seq = sibling_count.entry(sib_key).or_insert(0);
        *seq += 1;
        let seq_val = *seq;

        let id = match &parent_id {
            Some(pid) => format!("{}_{}", pid, seq_val),
            None => format!("N{}", seq_val),
        };

        if let Some(pid) = &parent_id {
            edges.push(Edge {
                from: pid.clone(),
                to: id.clone(),
            });
        }
        nodes.push(Node {
            id: id.clone(),
            label: label.to_string(),
            depth,
            parent_id,
        });
        stack.push((indent, id));
    }

    (nodes, edges)
}

fn sanitize_label(s: &str) -> String {
    s.replace('"', "'")
        // .replace(['{', '}', '[', ']', '|', '<', '>'], "")
        .replace(['{', '}', '[', ']', '|'], "")
}

fn generate_mermaid(nodes: &[Node], edges: &[Edge], direction: &str) -> String {
    let mut out = format!("flowchart {}\n", direction);
    for n in nodes {
        out.push_str(&format!("    {}[\"{}\"]\n", n.id, sanitize_label(&n.label)));
    }
    if !edges.is_empty() {
        out.push('\n');
        for e in edges {
            out.push_str(&format!("    {} --> {}\n", e.from, e.to));
        }
    }
    out
}

fn print_legend(nodes: &[Node]) -> String {
    let mut out = String::new();
    out.push_str("\n-- Node ID legend ------------------------------------------\n");
    out.push_str("Format : N{root}_{child}_{grandchild}_...\n");
    out.push_str("depth  = number of underscores + 1\n");
    out.push_str("parent = drop the last _{n} segment\n\n");
    out.push_str(&format!(
        "{:<20} {:<6} {:<20} {}\n",
        "ID", "depth", "parent", "label"
    ));
    out.push_str(&"-".repeat(70));
    out.push('\n');
    for n in nodes {
        let parent = n.parent_id.as_deref().unwrap_or("(root)");
        out.push_str(&format!(
            "{:<20} {:<6} {:<20} {}\n",
            n.id, n.depth, parent, n.label
        ));
    }
    out.push_str(&"-".repeat(70));
    out.push('\n');
    out
}

fn main() {
    let cli = Cli::parse();

    let input_text = match &cli.input {
        Some(path) => fs::read_to_string(path).unwrap_or_else(|e| {
            eprintln!("error: cannot read '{}': {}", path.display(), e);
            std::process::exit(1);
        }),
        None => {
            let mut buf = String::new();
            io::stdin().read_to_string(&mut buf).unwrap_or_else(|e| {
                eprintln!("error: cannot read stdin: {}", e);
                std::process::exit(1);
            });
            buf
        }
    };

    let (nodes, edges) = parse_list(&input_text);
    if nodes.is_empty() {
        eprintln!("error: no list items found in input.");
        std::process::exit(1);
    }

    let mut result = generate_mermaid(&nodes, &edges, &cli.direction);
    if cli.legend {
        result.push_str(&print_legend(&nodes));
    }

    match &cli.output {
        Some(path) => {
            fs::write(path, &result).unwrap_or_else(|e| {
                eprintln!("error: cannot write '{}': {}", path.display(), e);
                std::process::exit(1);
            });
            eprintln!("Wrote {} nodes -> {}", nodes.len(), path.display());
        }
        None => {
            print!("{}", result);
            io::stdout().flush().ok();
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn ids(text: &str) -> Vec<String> {
        let (nodes, _) = parse_list(text);
        nodes.iter().map(|n| n.id.clone()).collect()
    }

    #[test]
    fn single_root() {
        assert_eq!(ids("- Alpha"), vec!["N1"]);
    }

    #[test]
    fn two_roots() {
        assert_eq!(ids("- A\n- B"), vec!["N1", "N2"]);
    }

    #[test]
    fn depth2_child_ids() {
        assert_eq!(ids("- A\n  - B\n  - C"), vec!["N1", "N1_1", "N1_2"]);
    }

    #[test]
    fn depth3_child_ids() {
        assert_eq!(ids("- A\n  - B\n    - C"), vec!["N1", "N1_1", "N1_1_1"]);
    }

    #[test]
    fn depth_equals_underscores_plus_one() {
        let (nodes, _) = parse_list("- A\n  - B\n    - C\n      - D");
        for n in &nodes {
            let u = n.id.chars().filter(|&c| c == '_').count();
            assert_eq!(n.depth, u + 1, "id={}", n.id);
        }
    }

    #[test]
    fn parent_derivable_from_id() {
        let (nodes, _) = parse_list("- A\n  - B\n    - C\n  - D");
        for n in nodes.iter().filter(|n| n.parent_id.is_some()) {
            let derived = n.id.rsplit_once("_").unwrap().0;
            assert_eq!(derived, n.parent_id.as_deref().unwrap(), "id={}", n.id);
        }
    }

    #[test]
    fn edges_correct() {
        let (_, edges) = parse_list("- A\n  - B\n  - C");
        assert_eq!(edges[0].from, "N1");
        assert_eq!(edges[0].to, "N1_1");
        assert_eq!(edges[1].to, "N1_2");
    }

    #[test]
    fn ignores_empty_lines() {
        let (nodes, _) = parse_list("\n- A\n\n  - B\n\n");
        assert_eq!(nodes.len(), 2);
    }

    #[test]
    fn supports_star_bullets() {
        let (nodes, _) = parse_list("* A\n  * B");
        assert_eq!(nodes.len(), 2);
    }

    #[test]
    fn supports_numbered_lists() {
        let (nodes, _) = parse_list("1. A\n   2. B");
        assert_eq!(nodes.len(), 2);
    }
}
