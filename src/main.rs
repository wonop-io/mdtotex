use clap::{Arg, Command};
use markdown::message::Message;
use markdown::{mdast::Node, to_mdast, ParseOptions};
use std::fs;
use std::io::Write;
use std::path::Path;

fn escape_tex(text: &str) -> String {
    let mut escaped = text
        .replace("\\", "\\textbackslash{}")
        .replace("_", "\\_")
        .replace("&", "\\&")
        .replace("%", "\\%")
        .replace("#", "\\#")
        .replace("$", "\\$")
        .replace("{", "\\{")
        .replace("}", "\\}")
        .replace("~", "\\~")
        .replace("^", "\\^");

    // Replace unicode characters with LaTeX equivalents
    escaped = escaped
        .replace("π", "$\\pi$")
        .replace("α", "$\\alpha$")
        .replace("β", "$\\beta$")
        .replace("γ", "$\\gamma$")
        .replace("δ", "$\\delta$")
        .replace("ε", "$\\epsilon$")
        .replace("θ", "$\\theta$")
        .replace("λ", "$\\lambda$")
        .replace("μ", "$\\mu$")
        .replace("σ", "$\\sigma$")
        .replace("τ", "$\\tau$")
        .replace("φ", "$\\phi$")
        .replace("ω", "$\\omega$")
        .replace("Δ", "$\\Delta$")
        .replace("Π", "$\\Pi$")
        .replace("Σ", "$\\Sigma$")
        .replace("Ω", "$\\Omega$");

    escaped
}

fn visit_node(node: &Node, indent: usize, output: &mut String) {
    let indent_str = " ".repeat(indent);
    match node {
        Node::Root(root) => {
            for child in &root.children {
                visit_node(child, indent, output);
                if let Node::Paragraph(_) = child {
                    output.push_str("\n");
                }
            }
        }
        Node::Heading(h) => {
            let cmd = match h.depth {
                1 => "\\part",
                2 => "\\chapter",
                3 => "\\section",
                4 => "\\subsection",
                5 => "\\subsubsection",
                _ => "\\paragraph",
            };
            output.push_str(&format!("{}{}{{{}", indent_str, cmd, ""));
            for child in &h.children {
                visit_node(child, indent + 2, output);
            }
            output.push_str("}\n");
        }
        Node::Paragraph(p) => {
            let mut first = true;
            // output.push_str(&indent_str);
            for child in &p.children {
                if first {
                    first = false;
                } else {
                    output.push_str(" ");
                }
                visit_node(child, indent + 2, output);
            }
            output.push_str("\n");
        }
        Node::Text(t) => {
            output.push_str(&escape_tex(&t.value));
        }
        Node::Strong(s) => {
            output.push_str("\\textbf{");
            for child in &s.children {
                visit_node(child, indent, output);
            }
            output.push('}');
        }
        Node::Delete(d) => {
            output.push_str("\\sout{");
            for child in &d.children {
                visit_node(child, indent, output);
            }
            output.push('}');
        }
        Node::Emphasis(e) => {
            output.push_str("\\emph{");
            for child in &e.children {
                visit_node(child, indent, output);
            }
            output.push('}');
        }
        Node::InlineCode(c) => {
            output.push_str("\\verb|");
            output.push_str(&c.value);
            output.push('|');
        }
        Node::Code(c) => {
            let lang = c.lang.as_deref().unwrap_or("text");
            output.push_str(&format!("{}\\begin{{minted}}[framesep=2mm,baselinestretch=1.2,bgcolor=CodeBackground,fontsize=\\footnotesize,]{{{}}}\n", indent_str, lang));
            output.push_str(&format!("{}{}\n", indent_str, c.value));
            output.push_str(&format!("{}\\end{{minted}}\n", indent_str));
        }
        Node::List(l) => {
            let env = if l.ordered { "enumerate" } else { "itemize" };
            output.push_str(&format!("{}\\begin{{{}}}\n", indent_str, env));
            for child in &l.children {
                visit_node(child, indent + 2, output);
            }
            output.push_str(&format!("{}\\end{{{}}}\n", indent_str, env));
        }
        Node::ListItem(li) => {
            output.push_str(&format!("{}\\item ", indent_str));
            for child in &li.children {
                visit_node(child, indent + 2, output);
            }
        }
        Node::Table(t) => {
            output.push_str(&format!("{}\\begin{{tabular}}\n", indent_str));
            for child in &t.children {
                visit_node(child, indent + 2, output);
            }
            output.push_str(&format!("{}\\end{{tabular}}\n", indent_str));
        }
        Node::ThematicBreak(_) => {
            output.push_str(&format!("{}\\hrule\n", indent_str));
        }
        _ => {}
    }
}

fn main() -> Result<(), Message> {
    let matches = Command::new("md-parser")
        .version("1.0")
        .about("Parses markdown files")
        .arg(
            Arg::new("inputs")
                .help("Input markdown files")
                .required(true)
                .num_args(1..),
        )
        .arg(
            Arg::new("output-dir")
                .help("Output directory")
                .required(true)
                .short('o')
                .long("output-dir"),
        )
        .get_matches();

    let output_dir = matches.get_one::<String>("output-dir").unwrap();
    fs::create_dir_all(output_dir).expect("Could not create output directory");

    let input_files: Vec<_> = matches.get_many::<String>("inputs").unwrap().collect();

    for input_filename in input_files {
        // Load markdown file
        let content = fs::read_to_string(input_filename).expect("Could not read input file");

        // Create output filename
        let input_path = Path::new(input_filename);
        let file_stem = input_path.file_stem().unwrap().to_str().unwrap();
        let output_filename = Path::new(output_dir).join(file_stem).with_extension("tex");

        // Parse markdown
        let ast = to_mdast(&content, &ParseOptions::default())?;

        // Walk the AST and generate TeX content
        let mut output = String::new();
        visit_node(&ast, 0, &mut output);

        // Write to output file
        let mut file = fs::File::create(output_filename).expect("Could not create output file");
        file.write_all(output.as_bytes())
            .expect("Could not write to output file");
    }

    Ok(())
}
