//! OpenSCAD subset tokenizer + recursive descent parser.

/// AST node for OpenSCAD subset.
#[derive(Debug, Clone)]
pub enum ScadNode {
    Sphere {
        r: f32,
    },
    Cube {
        size: [f32; 3],
        center: bool,
    },
    Cylinder {
        h: f32,
        r1: f32,
        r2: f32,
        center: bool,
    },
    Translate {
        v: [f32; 3],
        children: Vec<ScadNode>,
    },
    Rotate {
        v: [f32; 3],
        children: Vec<ScadNode>,
    },
    Scale {
        v: [f32; 3],
        children: Vec<ScadNode>,
    },
    Color {
        rgba: [f32; 4],
        children: Vec<ScadNode>,
    },
    Union {
        children: Vec<ScadNode>,
    },
    Difference {
        children: Vec<ScadNode>,
    },
    Intersection {
        children: Vec<ScadNode>,
    },
    ModuleDef {
        name: String,
        body: Vec<ScadNode>,
    },
    ModuleCall {
        name: String,
    },
    Block {
        children: Vec<ScadNode>,
    },
}

#[derive(Debug, Clone, PartialEq)]
enum Token {
    Ident(String),
    Number(f32),
    LParen,
    RParen,
    LBracket,
    RBracket,
    LBrace,
    RBrace,
    Comma,
    Semi,
    Eq,
    Eof,
}

struct Lexer {
    chars: Vec<char>,
    pos: usize,
}

impl Lexer {
    fn new(src: &str) -> Self {
        Self {
            chars: src.chars().collect(),
            pos: 0,
        }
    }

    fn peek_char(&self) -> Option<char> {
        self.chars.get(self.pos).copied()
    }

    fn advance(&mut self) -> Option<char> {
        let c = self.chars.get(self.pos).copied();
        self.pos += 1;
        c
    }

    fn skip_ws_comments(&mut self) {
        loop {
            match self.peek_char() {
                Some(c) if c.is_whitespace() => {
                    self.advance();
                }
                Some('/') if self.chars.get(self.pos + 1) == Some(&'/') => {
                    while let Some(c) = self.advance() {
                        if c == '\n' {
                            break;
                        }
                    }
                }
                Some('/') if self.chars.get(self.pos + 1) == Some(&'*') => {
                    self.advance();
                    self.advance();
                    loop {
                        match self.advance() {
                            Some('*') if self.peek_char() == Some('/') => {
                                self.advance();
                                break;
                            }
                            None => break,
                            _ => {}
                        }
                    }
                }
                _ => break,
            }
        }
    }

    fn next_token(&mut self) -> Token {
        self.skip_ws_comments();
        match self.peek_char() {
            None => Token::Eof,
            Some('(') => {
                self.advance();
                Token::LParen
            }
            Some(')') => {
                self.advance();
                Token::RParen
            }
            Some('[') => {
                self.advance();
                Token::LBracket
            }
            Some(']') => {
                self.advance();
                Token::RBracket
            }
            Some('{') => {
                self.advance();
                Token::LBrace
            }
            Some('}') => {
                self.advance();
                Token::RBrace
            }
            Some(',') => {
                self.advance();
                Token::Comma
            }
            Some(';') => {
                self.advance();
                Token::Semi
            }
            Some('=') => {
                self.advance();
                Token::Eq
            }
            Some(c) if c == '-' || c == '.' || c.is_ascii_digit() => {
                let start = self.pos;
                if c == '-' {
                    self.advance();
                }
                while let Some(c) = self.peek_char() {
                    if c.is_ascii_digit() || c == '.' {
                        self.advance();
                    } else {
                        break;
                    }
                }
                let s: String = self.chars[start..self.pos].iter().collect();
                Token::Number(s.parse().unwrap_or(0.0))
            }
            Some(c) if c.is_ascii_alphabetic() || c == '_' || c == '#' => {
                let start = self.pos;
                while let Some(c) = self.peek_char() {
                    if c.is_ascii_alphanumeric() || c == '_' {
                        self.advance();
                    } else {
                        break;
                    }
                }
                Token::Ident(self.chars[start..self.pos].iter().collect())
            }
            Some('"') => {
                // String literal (for color names) — parse as ident
                self.advance();
                let start = self.pos;
                while let Some(c) = self.peek_char() {
                    if c == '"' {
                        break;
                    } else {
                        self.advance();
                    }
                }
                let s: String = self.chars[start..self.pos].iter().collect();
                self.advance(); // closing "
                Token::Ident(s)
            }
            Some(_) => {
                self.advance();
                self.next_token()
            } // skip unknown
        }
    }
}

struct Parser {
    tokens: Vec<Token>,
    pos: usize,
}

impl Parser {
    fn new(tokens: Vec<Token>) -> Self {
        Self { tokens, pos: 0 }
    }

    fn peek(&self) -> &Token {
        self.tokens.get(self.pos).unwrap_or(&Token::Eof)
    }

    fn advance(&mut self) -> Token {
        let t = self.tokens.get(self.pos).cloned().unwrap_or(Token::Eof);
        self.pos += 1;
        t
    }

    fn expect(&mut self, expected: &Token) {
        let t = self.advance();
        if &t != expected {
            // lenient: skip
        }
    }

    fn parse_number(&mut self) -> f32 {
        match self.advance() {
            Token::Number(n) => n,
            _ => 0.0,
        }
    }

    fn parse_vec3(&mut self) -> [f32; 3] {
        self.expect(&Token::LBracket);
        let x = self.parse_number();
        self.expect(&Token::Comma);
        let y = self.parse_number();
        self.expect(&Token::Comma);
        let z = self.parse_number();
        self.expect(&Token::RBracket);
        [x, y, z]
    }

    fn parse_color_arg(&mut self) -> [f32; 4] {
        match self.peek() {
            Token::LBracket => {
                self.advance();
                let r = self.parse_number();
                self.expect(&Token::Comma);
                let g = self.parse_number();
                self.expect(&Token::Comma);
                let b = self.parse_number();
                let a = if self.peek() == &Token::Comma {
                    self.advance();
                    self.parse_number()
                } else {
                    1.0
                };
                self.expect(&Token::RBracket);
                [r, g, b, a]
            }
            Token::Ident(name) => {
                let c = parse_color_name(&self.advance().to_string());
                c
            }
            _ => [0.5, 0.5, 0.5, 1.0],
        }
    }

    fn parse_named_args(&mut self) -> Vec<(String, Token)> {
        let mut args = Vec::new();
        self.expect(&Token::LParen);
        while self.peek() != &Token::RParen && self.peek() != &Token::Eof {
            match self.peek().clone() {
                Token::Ident(name) => {
                    let name = name.clone();
                    self.advance();
                    if self.peek() == &Token::Eq {
                        self.advance();
                        let val = self.advance();
                        args.push((name, val));
                    }
                }
                Token::Number(_) => {
                    let val = self.advance();
                    args.push(("".into(), val));
                }
                _ => {
                    self.advance();
                }
            }
            if self.peek() == &Token::Comma {
                self.advance();
            }
        }
        self.expect(&Token::RParen);
        args
    }

    fn parse_children(&mut self) -> Vec<ScadNode> {
        if self.peek() == &Token::LBrace {
            self.advance();
            let mut children = Vec::new();
            while self.peek() != &Token::RBrace && self.peek() != &Token::Eof {
                if let Some(node) = self.parse_statement() {
                    children.push(node);
                }
            }
            self.expect(&Token::RBrace);
            children
        } else if let Some(node) = self.parse_statement() {
            vec![node]
        } else {
            vec![]
        }
    }

    fn parse_statement(&mut self) -> Option<ScadNode> {
        match self.peek().clone() {
            Token::Ident(ref name) => {
                let name = name.clone();
                match name.as_str() {
                    "sphere" => {
                        self.advance();
                        self.expect(&Token::LParen);
                        let mut r = 0.5;
                        while self.peek() != &Token::RParen && self.peek() != &Token::Eof {
                            match self.peek() {
                                Token::Ident(k) if k == "r" => {
                                    self.advance();
                                    self.expect(&Token::Eq);
                                    r = self.parse_number();
                                }
                                Token::Number(_) => {
                                    r = self.parse_number();
                                }
                                _ => {
                                    self.advance();
                                }
                            }
                            if self.peek() == &Token::Comma {
                                self.advance();
                            }
                        }
                        self.expect(&Token::RParen);
                        self.skip_semi();
                        Some(ScadNode::Sphere { r })
                    }
                    "cube" => {
                        self.advance();
                        self.expect(&Token::LParen);
                        let mut size = [1.0, 1.0, 1.0];
                        let mut center = false;
                        while self.peek() != &Token::RParen && self.peek() != &Token::Eof {
                            match self.peek() {
                                Token::LBracket => {
                                    size = self.parse_vec3();
                                }
                                Token::Number(_) => {
                                    let s = self.parse_number();
                                    size = [s, s, s];
                                }
                                Token::Ident(k) if k == "center" => {
                                    self.advance();
                                    self.expect(&Token::Eq);
                                    center = matches!(self.advance(), Token::Ident(ref v) if v == "true");
                                }
                                Token::Ident(k) if k == "size" => {
                                    self.advance();
                                    self.expect(&Token::Eq);
                                    size = self.parse_vec3();
                                }
                                _ => {
                                    self.advance();
                                }
                            }
                            if self.peek() == &Token::Comma {
                                self.advance();
                            }
                        }
                        self.expect(&Token::RParen);
                        self.skip_semi();
                        Some(ScadNode::Cube { size, center })
                    }
                    "cylinder" => {
                        self.advance();
                        self.expect(&Token::LParen);
                        let mut h = 1.0;
                        let mut r1 = 0.5;
                        let mut r2 = 0.5;
                        let mut center = false;
                        while self.peek() != &Token::RParen && self.peek() != &Token::Eof {
                            match self.peek() {
                                Token::Ident(k) if k == "h" => {
                                    self.advance();
                                    self.expect(&Token::Eq);
                                    h = self.parse_number();
                                }
                                Token::Ident(k) if k == "r" => {
                                    self.advance();
                                    self.expect(&Token::Eq);
                                    let r = self.parse_number();
                                    r1 = r;
                                    r2 = r;
                                }
                                Token::Ident(k) if k == "r1" => {
                                    self.advance();
                                    self.expect(&Token::Eq);
                                    r1 = self.parse_number();
                                }
                                Token::Ident(k) if k == "r2" => {
                                    self.advance();
                                    self.expect(&Token::Eq);
                                    r2 = self.parse_number();
                                }
                                Token::Ident(k) if k == "center" => {
                                    self.advance();
                                    self.expect(&Token::Eq);
                                    center = matches!(self.advance(), Token::Ident(ref v) if v == "true");
                                }
                                Token::Number(_) => {
                                    h = self.parse_number();
                                }
                                _ => {
                                    self.advance();
                                }
                            }
                            if self.peek() == &Token::Comma {
                                self.advance();
                            }
                        }
                        self.expect(&Token::RParen);
                        self.skip_semi();
                        Some(ScadNode::Cylinder { h, r1, r2, center })
                    }
                    "translate" => {
                        self.advance();
                        self.expect(&Token::LParen);
                        let v = self.parse_vec3();
                        self.expect(&Token::RParen);
                        let children = self.parse_children();
                        Some(ScadNode::Translate { v, children })
                    }
                    "rotate" => {
                        self.advance();
                        self.expect(&Token::LParen);
                        let v = self.parse_vec3();
                        self.expect(&Token::RParen);
                        let children = self.parse_children();
                        Some(ScadNode::Rotate { v, children })
                    }
                    "scale" => {
                        self.advance();
                        self.expect(&Token::LParen);
                        let v = self.parse_vec3();
                        self.expect(&Token::RParen);
                        let children = self.parse_children();
                        Some(ScadNode::Scale { v, children })
                    }
                    "color" => {
                        self.advance();
                        self.expect(&Token::LParen);
                        let rgba = self.parse_color_arg();
                        self.expect(&Token::RParen);
                        let children = self.parse_children();
                        Some(ScadNode::Color { rgba, children })
                    }
                    "union" => {
                        self.advance();
                        self.expect(&Token::LParen);
                        self.expect(&Token::RParen);
                        let children = self.parse_children();
                        Some(ScadNode::Union { children })
                    }
                    "difference" => {
                        self.advance();
                        self.expect(&Token::LParen);
                        self.expect(&Token::RParen);
                        let children = self.parse_children();
                        Some(ScadNode::Difference { children })
                    }
                    "intersection" => {
                        self.advance();
                        self.expect(&Token::LParen);
                        self.expect(&Token::RParen);
                        let children = self.parse_children();
                        Some(ScadNode::Intersection { children })
                    }
                    "module" => {
                        self.advance();
                        let mname = match self.advance() {
                            Token::Ident(n) => n,
                            _ => "unnamed".into(),
                        };
                        self.expect(&Token::LParen);
                        self.expect(&Token::RParen);
                        let body = self.parse_children();
                        Some(ScadNode::ModuleDef { name: mname, body })
                    }
                    _ => {
                        // module call: name();
                        self.advance();
                        if self.peek() == &Token::LParen {
                            self.advance();
                            self.expect(&Token::RParen);
                            self.skip_semi();
                            Some(ScadNode::ModuleCall { name })
                        } else {
                            self.skip_semi();
                            None
                        }
                    }
                }
            }
            Token::Eof => None,
            _ => {
                self.advance();
                None
            }
        }
    }

    fn skip_semi(&mut self) {
        if self.peek() == &Token::Semi {
            self.advance();
        }
    }
}

impl std::fmt::Display for Token {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Token::Ident(s) => write!(f, "{s}"),
            Token::Number(n) => write!(f, "{n}"),
            _ => write!(f, "{self:?}"),
        }
    }
}

fn parse_color_name(name: &str) -> [f32; 4] {
    match name.to_lowercase().as_str() {
        "red" => [1.0, 0.0, 0.0, 1.0],
        "green" => [0.0, 1.0, 0.0, 1.0],
        "blue" => [0.0, 0.0, 1.0, 1.0],
        "white" => [1.0, 1.0, 1.0, 1.0],
        "black" => [0.0, 0.0, 0.0, 1.0],
        "yellow" => [1.0, 1.0, 0.0, 1.0],
        "cyan" => [0.0, 1.0, 1.0, 1.0],
        "magenta" => [1.0, 0.0, 1.0, 1.0],
        "orange" => [1.0, 0.5, 0.0, 1.0],
        "purple" => [0.5, 0.0, 0.5, 1.0],
        "gray" | "grey" => [0.5, 0.5, 0.5, 1.0],
        "pink" => [1.0, 0.75, 0.8, 1.0],
        _ => [0.5, 0.5, 0.5, 1.0],
    }
}

/// Parse OpenSCAD source code into an AST.
pub fn parse(src: &str) -> Vec<ScadNode> {
    let mut lexer = Lexer::new(src);
    let mut tokens = Vec::new();
    loop {
        let tok = lexer.next_token();
        if tok == Token::Eof {
            break;
        }
        tokens.push(tok);
    }
    tokens.push(Token::Eof);

    let mut parser = Parser::new(tokens);
    let mut nodes = Vec::new();
    while parser.peek() != &Token::Eof {
        if let Some(node) = parser.parse_statement() {
            nodes.push(node);
        }
    }
    nodes
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn parse_sphere() {
        let nodes = parse("sphere(r=1.5);");
        assert_eq!(nodes.len(), 1);
        match &nodes[0] {
            ScadNode::Sphere { r } => assert!((*r - 1.5).abs() < 0.001),
            _ => panic!("expected Sphere"),
        }
    }

    #[test]
    fn parse_translate_cube() {
        let nodes = parse("translate([1,2,3]) cube([4,5,6]);");
        assert_eq!(nodes.len(), 1);
        match &nodes[0] {
            ScadNode::Translate { v, children } => {
                assert_eq!(*v, [1.0, 2.0, 3.0]);
                assert_eq!(children.len(), 1);
                match &children[0] {
                    ScadNode::Cube { size, .. } => assert_eq!(*size, [4.0, 5.0, 6.0]),
                    _ => panic!("expected Cube"),
                }
            }
            _ => panic!("expected Translate"),
        }
    }

    #[test]
    fn parse_color_union() {
        let src = r#"
            union() {
                color([0.34, 0.80, 0.01]) sphere(r=1.5);
                translate([0, 2.8, 0]) color([1,1,1]) sphere(r=0.5);
            }
        "#;
        let nodes = parse(src);
        assert_eq!(nodes.len(), 1);
        match &nodes[0] {
            ScadNode::Union { children } => assert_eq!(children.len(), 2),
            _ => panic!("expected Union"),
        }
    }

    #[test]
    fn parse_cylinder() {
        let nodes = parse("cylinder(h=3, r1=1, r2=0.5);");
        assert_eq!(nodes.len(), 1);
        match &nodes[0] {
            ScadNode::Cylinder { h, r1, r2, .. } => {
                assert!((*h - 3.0).abs() < 0.001);
                assert!((*r1 - 1.0).abs() < 0.001);
                assert!((*r2 - 0.5).abs() < 0.001);
            }
            _ => panic!("expected Cylinder"),
        }
    }

    #[test]
    fn parse_module_def_call() {
        let src = "module yoro() { sphere(r=1); } yoro();";
        let nodes = parse(src);
        assert_eq!(nodes.len(), 2);
        match &nodes[0] {
            ScadNode::ModuleDef { name, body } => {
                assert_eq!(name, "yoro");
                assert_eq!(body.len(), 1);
            }
            _ => panic!("expected ModuleDef"),
        }
        match &nodes[1] {
            ScadNode::ModuleCall { name } => assert_eq!(name, "yoro"),
            _ => panic!("expected ModuleCall"),
        }
    }
}
