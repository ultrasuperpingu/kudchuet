// Automatically generated from source file: sgf.tpg
// By TinyPG v1.6 available at https://github.com/ultrasuperpingu/TinyPG


use super::{parse_tree::{IParseNode, IParserTree, ParseError, ParseTree}, scanner::{Scanner, Token, TokenType}};
pub struct Parser
{
	scanner : Scanner
}
impl Parser {
	pub fn new(scanner:Scanner) -> Self
	{
		Self {scanner}
	}

	pub fn parse(&mut self, input: &str) -> ParseTree
	{
		self.parse_with_tree(input, ParseTree::new())
	}

	pub fn parse_with_tree(&mut self, input: &str, mut tree: ParseTree) -> ParseTree
	{
		self.scanner.init(input);

		self.parse_node_start(&mut tree, None);
		tree.skipped = self.scanner.skipped.clone();
		tree
	}

	pub fn parse_start(&mut self, input: &str, mut tree : ParseTree) -> ParseTree // NonTerminalSymbol: Start
	{
		self.scanner.init(input);
		let mut node = tree.root.take().unwrap();
		self.parse_node_start(&mut tree, Some(&mut node));
		tree.skipped = self.scanner.skipped.clone();
		tree.root = Some(node);
		tree
	}

	fn parse_node_start(&mut self, tree:&mut ParseTree, parent : Option<&mut Box<dyn IParseNode>>) // NonTerminalSymbol: Start
	{
		#[allow(unused_variables, unused_mut)]
		let mut tok: Token;
		#[allow(unused_variables, unused_mut)]
		let mut n: Box<dyn IParseNode>;
		let mut node = tree.create_node(self.scanner.get_token(TokenType::Start), "Start".to_string());

		self.parse_node_tree(tree, Some(&mut node)); // NonTerminal Rule: Tree

		if let Some(p) = parent {
			p.get_token_mut().update_range(node.get_token());
			p.add_node(node);
		} else {
			tree.root = Some(node);
		}
	} // NonTerminalSymbol: Start

	pub fn parse_tree(&mut self, input: &str, mut tree : ParseTree) -> ParseTree // NonTerminalSymbol: Tree
	{
		self.scanner.init(input);
		let mut node = tree.root.take().unwrap();
		self.parse_node_tree(&mut tree, Some(&mut node));
		tree.skipped = self.scanner.skipped.clone();
		tree.root = Some(node);
		tree
	}

	fn parse_node_tree(&mut self, tree:&mut ParseTree, parent : Option<&mut Box<dyn IParseNode>>) // NonTerminalSymbol: Tree
	{
		#[allow(unused_variables, unused_mut)]
		let mut tok: Token;
		#[allow(unused_variables, unused_mut)]
		let mut n: Box<dyn IParseNode>;
		let mut node = tree.create_node(self.scanner.get_token(TokenType::Tree), "Tree".to_string());


		 // Concat Rule
		tok = self.scanner.scan(vec![TokenType::PAROPEN]); // Terminal Rule: PAROPEN
		n = tree.create_node(tok.clone(), tok.to_string() );
		node.get_token_mut().update_range(&tok);
		node.add_node(n);
		if tok._type != TokenType::PAROPEN {
			tree.errors.push(ParseError::from_token("Unexpected token '".to_string() + tok.text.replace(&"\n".to_string(), "").as_str() + "' found. Expected 'PAROPEN'.", 0x1001, &tok, false));
			if let Some(p) = parent {
				p.add_node(node);
			} else {
				tree.root = Some(node);
			}
			return;
		}

		 // Concat Rule
		loop { // OneOrMore Rule
			tok = self.scanner.look_ahead(vec![TokenType::SEMICOLON, TokenType::PAROPEN]);
			match tok._type
			{ // Choice Rule
				TokenType::SEMICOLON => {
					loop { // OneOrMore Rule
						self.parse_node_treenode(tree, Some(&mut node)); // NonTerminal Rule: TreeNode
						tok = self.scanner.look_ahead(vec![TokenType::SEMICOLON]);
						if tok._type == TokenType::SEMICOLON {
							break;
						}
					} // OneOrMore Rule
				},
				TokenType::PAROPEN => {
					self.parse_node_tree(tree, Some(&mut node)); // NonTerminal Rule: Tree
				},
				_ => {
					tree.errors.push(ParseError::from_token("Unexpected token '".to_string() + tok.text.replace("\n", "").as_str() + "' found. Expected SEMICOLON or PAROPEN.", 0x0002, &tok, false));
				}
			} // Choice Rule
			tok = self.scanner.look_ahead(vec![TokenType::SEMICOLON, TokenType::PAROPEN]);
			if tok._type == TokenType::SEMICOLON
		    || tok._type == TokenType::PAROPEN {
				break;
			}
		} // OneOrMore Rule

		 // Concat Rule
		tok = self.scanner.scan(vec![TokenType::PARCLOSE]); // Terminal Rule: PARCLOSE
		n = tree.create_node(tok.clone(), tok.to_string() );
		node.get_token_mut().update_range(&tok);
		node.add_node(n);
		if tok._type != TokenType::PARCLOSE {
			tree.errors.push(ParseError::from_token("Unexpected token '".to_string() + tok.text.replace(&"\n".to_string(), "").as_str() + "' found. Expected 'PARCLOSE'.", 0x1001, &tok, false));
			if let Some(p) = parent {
				p.add_node(node);
			} else {
				tree.root = Some(node);
			}
			return;
		}

		if let Some(p) = parent {
			p.get_token_mut().update_range(node.get_token());
			p.add_node(node);
		} else {
			tree.root = Some(node);
		}
	} // NonTerminalSymbol: Tree

	pub fn parse_treenode(&mut self, input: &str, mut tree : ParseTree) -> ParseTree // NonTerminalSymbol: TreeNode
	{
		self.scanner.init(input);
		let mut node = tree.root.take().unwrap();
		self.parse_node_treenode(&mut tree, Some(&mut node));
		tree.skipped = self.scanner.skipped.clone();
		tree.root = Some(node);
		tree
	}

	fn parse_node_treenode(&mut self, tree:&mut ParseTree, parent : Option<&mut Box<dyn IParseNode>>) // NonTerminalSymbol: TreeNode
	{
		#[allow(unused_variables, unused_mut)]
		let mut tok: Token;
		#[allow(unused_variables, unused_mut)]
		let mut n: Box<dyn IParseNode>;
		let mut node = tree.create_node(self.scanner.get_token(TokenType::TreeNode), "TreeNode".to_string());


		 // Concat Rule
		tok = self.scanner.scan(vec![TokenType::SEMICOLON]); // Terminal Rule: SEMICOLON
		n = tree.create_node(tok.clone(), tok.to_string() );
		node.get_token_mut().update_range(&tok);
		node.add_node(n);
		if tok._type != TokenType::SEMICOLON {
			tree.errors.push(ParseError::from_token("Unexpected token '".to_string() + tok.text.replace(&"\n".to_string(), "").as_str() + "' found. Expected 'SEMICOLON'.", 0x1001, &tok, false));
			if let Some(p) = parent {
				p.add_node(node);
			} else {
				tree.root = Some(node);
			}
			return;
		}

		 // Concat Rule
		loop { // OneOrMore Rule
			self.parse_node_attribute(tree, Some(&mut node)); // NonTerminal Rule: Attribute
			tok = self.scanner.look_ahead(vec![TokenType::IDENT]);
			if tok._type == TokenType::IDENT {
				break;
			}
		} // OneOrMore Rule

		if let Some(p) = parent {
			p.get_token_mut().update_range(node.get_token());
			p.add_node(node);
		} else {
			tree.root = Some(node);
		}
	} // NonTerminalSymbol: TreeNode

	pub fn parse_attribute(&mut self, input: &str, mut tree : ParseTree) -> ParseTree // NonTerminalSymbol: Attribute
	{
		self.scanner.init(input);
		let mut node = tree.root.take().unwrap();
		self.parse_node_attribute(&mut tree, Some(&mut node));
		tree.skipped = self.scanner.skipped.clone();
		tree.root = Some(node);
		tree
	}

	fn parse_node_attribute(&mut self, tree:&mut ParseTree, parent : Option<&mut Box<dyn IParseNode>>) // NonTerminalSymbol: Attribute
	{
		#[allow(unused_variables, unused_mut)]
		let mut tok: Token;
		#[allow(unused_variables, unused_mut)]
		let mut n: Box<dyn IParseNode>;
		let mut node = tree.create_node(self.scanner.get_token(TokenType::Attribute), "Attribute".to_string());


		 // Concat Rule
		tok = self.scanner.scan(vec![TokenType::IDENT]); // Terminal Rule: IDENT
		n = tree.create_node(tok.clone(), tok.to_string() );
		node.get_token_mut().update_range(&tok);
		node.add_node(n);
		if tok._type != TokenType::IDENT {
			tree.errors.push(ParseError::from_token("Unexpected token '".to_string() + tok.text.replace(&"\n".to_string(), "").as_str() + "' found. Expected 'IDENT'.", 0x1001, &tok, false));
			if let Some(p) = parent {
				p.add_node(node);
			} else {
				tree.root = Some(node);
			}
			return;
		}

		 // Concat Rule
		loop { // OneOrMore Rule

			 // Concat Rule
			tok = self.scanner.scan(vec![TokenType::BROPEN]); // Terminal Rule: BROPEN
			n = tree.create_node(tok.clone(), tok.to_string() );
			node.get_token_mut().update_range(&tok);
			node.add_node(n);
			if tok._type != TokenType::BROPEN {
				tree.errors.push(ParseError::from_token("Unexpected token '".to_string() + tok.text.replace(&"\n".to_string(), "").as_str() + "' found. Expected 'BROPEN'.", 0x1001, &tok, false));
				if let Some(p) = parent {
					p.add_node(node);
				} else {
					tree.root = Some(node);
				}
				return;
			}

			 // Concat Rule
			tok = self.scanner.scan(vec![TokenType::CONTENT]); // Terminal Rule: CONTENT
			n = tree.create_node(tok.clone(), tok.to_string() );
			node.get_token_mut().update_range(&tok);
			node.add_node(n);
			if tok._type != TokenType::CONTENT {
				tree.errors.push(ParseError::from_token("Unexpected token '".to_string() + tok.text.replace(&"\n".to_string(), "").as_str() + "' found. Expected 'CONTENT'.", 0x1001, &tok, false));
				if let Some(p) = parent {
					p.add_node(node);
				} else {
					tree.root = Some(node);
				}
				return;
			}

			 // Concat Rule
			tok = self.scanner.scan(vec![TokenType::BRCLOSE]); // Terminal Rule: BRCLOSE
			n = tree.create_node(tok.clone(), tok.to_string() );
			node.get_token_mut().update_range(&tok);
			node.add_node(n);
			if tok._type != TokenType::BRCLOSE {
				tree.errors.push(ParseError::from_token("Unexpected token '".to_string() + tok.text.replace(&"\n".to_string(), "").as_str() + "' found. Expected 'BRCLOSE'.", 0x1001, &tok, false));
				if let Some(p) = parent {
					p.add_node(node);
				} else {
					tree.root = Some(node);
				}
				return;
			}
			tok = self.scanner.look_ahead(vec![TokenType::BROPEN]);
			if tok._type == TokenType::BROPEN {
				break;
			}
		} // OneOrMore Rule

		if let Some(p) = parent {
			p.get_token_mut().update_range(node.get_token());
			p.add_node(node);
		} else {
			tree.root = Some(node);
		}
	} // NonTerminalSymbol: Attribute



}
