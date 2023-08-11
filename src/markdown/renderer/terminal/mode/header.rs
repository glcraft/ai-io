use super::InlineStyles;
use super::super::utils;

use crossterm::{
    queue, 
    style::*,
    ErrorKind,
};
use super::super::token;

pub struct Header {
    level: usize,
    tokens: Vec<token::Token>,
    styles: InlineStyles,
}

impl Default for Header {
    fn default() -> Self {
        Self {
            level: 1,
            tokens: Vec::new(),
            styles: InlineStyles::new(Attributes::from([Attribute::Reverse, Attribute::Bold].as_slice())),
        }
    }
}

impl Header {
    pub fn new(level: usize) -> Self {
        Self {
            level,
            ..Default::default()
        }
    }
    pub fn init(&self) -> Result<(), ErrorKind> {
        let line_length = self.header_width()?;
        self.styles.apply_styles()?;
        queue!(std::io::stdout(), 
            Print(utils::repeat_char(utils::CODE_BLOCK_LINE_CHAR[0], utils::CODE_BLOCK_MARGIN.max(line_length as usize)))
        )?;
        Ok(())
    }
    fn len(&self) -> usize {
        self.tokens.iter()
            .fold(0, |acc, token| {
                acc + if let token::Token::Text(v) = token { v.len() } else { 0 }
            })
    }
    pub fn push_token(&mut self, token: token::Token) -> Result<(), ErrorKind> {
        self.tokens.push(token);
        self.draw_text()
    }
    fn draw_text(&mut self) -> Result<(), ErrorKind> {
        let pos_cursor = crossterm::cursor::position()?;
        let new_cursor_pos = (0.max((self.header_width()? - self.len() as isize) / 2) as u16 , pos_cursor.1);
        
        queue!(std::io::stdout(), 
            crossterm::cursor::MoveTo(new_cursor_pos.0, new_cursor_pos.1),
        )?;
        let mut istyles = InlineStyles::new(Attributes::from([Attribute::Reverse, Attribute::Bold].as_slice()));
        istyles.reset_styles()?;
        for token in &self.tokens {
            match token {
                token::Token::Text(s) => queue!(std::io::stdout(), crossterm::style::Print(s))?,
                token::Token::InlineStyle(token::Marker::Begin(v)) => istyles.push_style(v.clone())?,
                token::Token::InlineStyle(token::Marker::End(_)) => istyles.pop_style()?,
                _ => (),
            }
        }
        Ok(())
        
    }
    fn header_width(&self) -> Result<isize, ErrorKind> {
        let term_width = crossterm::terminal::size()?.0 as isize;
        Ok(term_width / (1<<(self.level-1)))
    }
}