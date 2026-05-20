// Automatically generated from source file: sgf.tpg
// By TinyPG v1.6 available at https://github.com/ultrasuperpingu/TinyPG
//use fancy_regex::{Regex, RegexBuilder};
//use regex::{Regex, RegexBuilder};
use regex::{Regex, RegexBuilder};
use std::collections::HashMap;

#[derive(Debug)]
pub struct Scanner
{
	pub input: String,
	pub startpos : usize,
	pub endpos : usize,
	pub currentline : usize,
	pub currentcolumn : usize,
	pub currentposition : usize,
	pub skipped : Vec<Token>, // tokens that were skipped
	pub patterns : HashMap<TokenType, Regex>,

	/// Already read next token (probably a source of bugs)
	look_ahead_token : Option<Token>,
	/// All tokens types
	tokens : Vec<TokenType>,
	/// Tokens that should be ignored
	skip_list : Vec<TokenType>
}
impl Default for Scanner {
	fn default() -> Self {
		Self::new()
	}
}
impl Scanner {
	pub fn new() -> Self
	{
		let mut regex : Regex;
		let mut ret=Self {
			input: String::from(""),
			startpos : 0,
			endpos : 0,
			currentline : 0,
			currentcolumn : 0,
			currentposition : 0,
			skipped : Vec::new(), // tokens that were skipped
			patterns : HashMap::new(),
			look_ahead_token : None,
			tokens : Vec::new(),
			skip_list : Vec::new() // tokens to be skipped
		};
		ret.skip_list.push(TokenType::WHITESPACE);


		regex = RegexBuilder::new("^(?:\\s+)").build().expect("Invalid regex");
		ret.patterns.insert(TokenType::WHITESPACE, regex);
		ret.tokens.push(TokenType::WHITESPACE);

		regex = RegexBuilder::new("^(?:$)").build().expect("Invalid regex");
		ret.patterns.insert(TokenType::EOF, regex);
		ret.tokens.push(TokenType::EOF);

		regex = RegexBuilder::new("^(?:\\[)").build().expect("Invalid regex");
		ret.patterns.insert(TokenType::BROPEN, regex);
		ret.tokens.push(TokenType::BROPEN);

		regex = RegexBuilder::new("^(?:\\])").build().expect("Invalid regex");
		ret.patterns.insert(TokenType::BRCLOSE, regex);
		ret.tokens.push(TokenType::BRCLOSE);

		regex = RegexBuilder::new("^(?:\\()").build().expect("Invalid regex");
		ret.patterns.insert(TokenType::PAROPEN, regex);
		ret.tokens.push(TokenType::PAROPEN);

		regex = RegexBuilder::new("^(?:\\))").build().expect("Invalid regex");
		ret.patterns.insert(TokenType::PARCLOSE, regex);
		ret.tokens.push(TokenType::PARCLOSE);

		regex = RegexBuilder::new("^(?:;)").build().expect("Invalid regex");
		ret.patterns.insert(TokenType::SEMICOLON, regex);
		ret.tokens.push(TokenType::SEMICOLON);

		regex = RegexBuilder::new("^(?::)").build().expect("Invalid regex");
		ret.patterns.insert(TokenType::COLON, regex);
		ret.tokens.push(TokenType::COLON);

		regex = RegexBuilder::new("^(?:[a-zA-Z][a-zA-Z0-9]*)").build().expect("Invalid regex");
		ret.patterns.insert(TokenType::IDENT, regex);
		ret.tokens.push(TokenType::IDENT);

		regex = RegexBuilder::new("^(?:([^\\\\\\]:]|\\\\(.|\\n))*)").build().expect("Invalid regex");
		ret.patterns.insert(TokenType::CONTENT, regex);
		ret.tokens.push(TokenType::CONTENT);


		ret
	}


	pub fn init(&mut self, input : &str)
	{
		self.input = input.to_owned();
		self.startpos = 0;
		self.endpos = 0;
		self.currentline = 1;
		self.currentcolumn = 1;
		self.currentposition = 0;
		self.look_ahead_token = None;
	}

	pub fn get_token(&self, _type : TokenType) -> Token
	{
		let mut t = Token::new_with_start_end(self.startpos, self.endpos);
		t._type = _type;
		t
	}

	/// Executes a lookahead of the next token
	/// and will advance the scan on the input string
	pub fn scan(&mut self, expectedtokens: Vec<TokenType>) -> Token
	{
		let tok = self.look_ahead(expectedtokens); // temporarely retrieve the lookahead
		self.startpos = tok.endpos;
		self.endpos = tok.endpos; // set the tokenizer to the new scan position
		self.currentline = tok.line + (tok.text.len() - tok.text.replace("\n", "").len());
		self.look_ahead_token.take().unwrap() // reset lookahead token, so scanning will continue
	}

	/// Returns token with longest best match
	pub fn look_ahead(&mut self,expectedtokens : Vec<TokenType>) -> Token
	{
		let mut startpos = self.startpos;
		let mut endpos = self.endpos;
		let mut currentline = self.currentline;
		let mut scantokens : Vec<TokenType>;
		let mut tok : Token;

		// this prevents double scanning and matching
		// increased performance
		// TODO: check this, what if the expected token are different since last call?
		// Check at least that LookAheadToken is part of the expected tokens
		if let Some(token) = self.look_ahead_token.clone()
		{
			if token._type != TokenType::_UNDETERMINED_
				&& token._type != TokenType::_NONE_ {
				return token;
			}
		}

		// if no scantokens specified, then scan for all of them (= backward compatible)
		if expectedtokens.is_empty()
		{
			scantokens = self.tokens.clone();
		}
		else
		{
			scantokens = expectedtokens.clone();
			scantokens.append(&mut self.skip_list.clone());
		}

		loop
		{
			let mut len = 0_usize;
			let mut index : TokenType = TokenType::_END_;
			//string input = Input.Substring(startpos);

			tok = Token::new_with_start_end(startpos, endpos);

			for scantoken in &scantokens
			{
				let r = &self.patterns[scantoken];
				//let m = r.match(Input, startpos);
				//if (m.Success && m.Index == startpos && ((m.Length > len) || (scantokens[i] < index && m.Length == len)))
				//if let Some(caps) = r.captures(&self.input[startpos..]) 
				//if let Ok(Some(caps)) = r.captures(&self.input[startpos..]) 
				if let Some(caps) = r.captures(&self.input[startpos..])
				{
					//if (m.Index == startpos && ((m.Length > len) || (scantokens[i] < index && m.Length == len)))
					{
						len = caps[0].len();
						index = *scantoken;
					}
				}
			}

			if index != TokenType::_END_ //&& len >= 0
			{
				tok.endpos = startpos + len;
				tok.text = self.input[tok.startpos..(tok.startpos+len)].to_string();
				tok._type = index;
			}
			else if tok.startpos == tok.endpos
			{
				if tok.startpos < self.input.len() {
					tok.text = self.input[tok.startpos..tok.startpos+1].to_string();
				} else {
					tok.text = "EOF".to_string();
				}
			}

			// Update the line and column count for error reporting.
			tok.line = currentline;
			if tok.startpos < self.input.len() {
				tok.column = tok.startpos - self.input[tok.startpos..].to_string().rfind('\n').unwrap_or_default();
			}
			if self.skip_list.contains(&tok._type)
			{
				startpos = tok.endpos;
				endpos = tok.endpos;
				currentline = tok.line + (tok.text.len() - tok.text.replace("\n", "").len());
				self.skipped.push(tok.clone());
			}
			else
			{
				// only assign to non-skipped tokens
				tok.skipped = self.skipped.clone(); // assign prior skips to this token
				self.skipped = Vec::new (); //reset skips
			}
			if !self.skip_list.contains(&tok._type) {
				break;
			}
		}
		

		self.look_ahead_token = Some(tok);
		self.look_ahead_token.clone().unwrap()
	}
}



#[derive(PartialEq, Eq, Hash, Clone, Debug, Copy, PartialOrd)]
#[allow(clippy::upper_case_acronyms, non_camel_case_types)]
#[repr(u32)]
pub enum TokenType
{

		//Non terminal tokens:
	_NONE_             = 0,
	_UNDETERMINED_     = 1,

		//Non terminal tokens:
	Start              = 2,
	Tree               = 3,
	TreeNode           = 4,
	LeafNode           = 5,
	Attribute          = 6,
	Tuple              = 7,

		//Terminal tokens:
	WHITESPACE         = 8,
	EOF                = 9,
	BROPEN             = 10,
	BRCLOSE            = 11,
	PAROPEN            = 12,
	PARCLOSE           = 13,
	SEMICOLON          = 14,
	COLON              = 15,
	IDENT              = 16,
	CONTENT            = 17,


	//End
	_END_ = u32::MAX
}

#[derive(Debug, Clone)]
pub struct Token
{
	pub line : usize,
	pub column : usize,
	pub startpos : usize,
	pub endpos : usize,
	pub text : String,

	// contains all prior skipped symbols
	pub skipped : Vec<Token>,
	pub _type : TokenType
}
impl Default for Token {
	fn default() -> Self {
		Self::new()
	}
}

impl Token {
	pub fn new() -> Self
	{
		Self::new_with_start_end(0,0)
	}

	pub fn new_with_start_end(start: usize, end : usize) -> Self
	{
		Self {
			_type : TokenType::_UNDETERMINED_,
			startpos : start,
			endpos : end,
			text : String::from(""),
			line: 1,
			column: 1,
			skipped: vec![],
		}
	}

	pub fn update_range(&mut self, token:&Token)
	{
		if token.startpos < self.startpos {
			self.startpos = token.startpos;
		}
		if token.endpos > self.endpos {
			self.endpos = token.endpos;
		}
	}

}
impl std::fmt::Display for Token {
	fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
//		if (Text != null)
//			return Type.ToString() + " '" + Text + "'";
//		else
//			return Type.ToString();
		write!(f, "{:?} '{}'", self._type, self.text)
	}
}


