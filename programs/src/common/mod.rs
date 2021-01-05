pub fn trim_line_comment(mut line: String) -> String {
    if let Some(i) = line.find("//") {
        line.truncate(i)
    }
    line
}
