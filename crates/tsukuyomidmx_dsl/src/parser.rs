use std::path::Path;

use tree_sitter::Parser;

pub fn parse_file(file: impl AsRef<Path>) {
    let mut parser = Parser::new();
    parser
        .set_language(&tree_sitter_tsukuyomidmx::LANGUAGE.into())
        .unwrap();

    let mut tree = parser
        .parse("import { blink } from \"../blink.tkd\"", None)
        .unwrap();
    dbg!(tree);
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn parser_works() {
        let mut parser = Parser::new();
        parser
            .set_language(&tree_sitter_tsukuyomidmx::LANGUAGE.into())
            .unwrap();

        let mut tree = parser
            .parse("import { blink } from \"../blink.tkd\"", None)
            .unwrap();
        dbg!(&tree);
    }
}
