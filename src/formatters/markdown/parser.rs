use thiserror::Error;
use super::super::Formatter;
use super::renderer::Renderer;
use super::token;
use anyhow::Result;

#[derive(Debug, Error)]
pub enum ParseError<RendererErr> {
    #[error("Renderer error: {0}")]
    RendererError(#[from] RendererErr),
}
type AnalyzerCallback<R> = fn(&mut Parser<R>, char) -> Result<(), ParseError<<R as Renderer>::Error>>;

pub struct Parser<R: Renderer> {
    renderer: R,
    current_text: String,
    current_token: String,
    previous_char: Option<char>,
    inline_style_tokens: Vec<token::InlineStyleToken>,
    mode_func: AnalyzerCallback<R>,
}

impl<R: Renderer> Formatter for Parser<R> {
    fn push(&mut self, text: &str) -> Result<()> {
        for c in text.chars() {
            (self.mode_func)(self, c)?;
        }
        self.push_current_text()?;
        Ok(self.renderer.flush()?)
    }
    fn end_of_document(&mut self) -> Result<()> {
        self.push_current_text()?;
        self.renderer.push_token(token::Token::EndDocument)?;
        Ok(())
    }
}

impl<R: Renderer> Parser<R> {
    // type Result<T> = Result<T, ParseError<R::Error>>;
    pub fn new(renderer: R) -> Self {
        Self {
            renderer,
            current_text: String::new(),
            current_token: String::with_capacity(3),
            previous_char: None,
            inline_style_tokens: Vec::new(),
            mode_func: Self::analyse_text,
        }
    }
    fn analyse_text(&mut self, c: char) -> Result<(), ParseError<R::Error>> {
        match c {
            '\n' => {
                self.apply_text_token(c, false)?;
                self.push_current_text()?;
                self.renderer.push_token(token::Token::Newline)?;
                self.previous_char = None;
            }
            '*' | '_' if self.inline_style_tokens.last() != Some(&token::InlineStyleToken::OneQuote) && (self.current_token.contains(c) || self.current_token.is_empty()) => {
                self.current_token.push(c);
            }
            '`' if self.current_token.contains(c) || self.current_token.is_empty() => {
                self.current_token.push(c);
            }
            '-' | '#' if self.previous_char.is_none() => self.current_token.push(c),
            _ => {
                self.apply_text_token(c, true)?;
                self.previous_char = Some(c);
            }
        }
        
        Ok(())
    }
    fn analyse_code_block(&mut self, c: char) -> Result<(), ParseError<R::Error>> {
        match c {
            '\n' => {
                self.apply_code_token(c)?;
                self.push_current_text()?;
                self.renderer.push_token(token::Token::Newline)?;
                self.previous_char = None;
            }
            '`' if self.previous_char.is_none() => self.current_token.push(c),
            _ => {
                self.apply_code_token(c)?;
                self.current_text.push(c);
                self.previous_char = Some(c);
            }
        }
        Ok(())
    }
    fn apply_code_token(&mut self, _: char) -> Result<(), ParseError<R::Error>> {
        if self.current_token == "```" {
            self.current_token.clear();
            self.mode_func = Self::analyse_text;
            self.renderer.push_token(token::Token::EndCode)?;
        }
        
        Ok(())
    }
    fn apply_text_token(&mut self, current_char: char, print_current_char: bool) -> Result<(), ParseError<R::Error>> {
        'skip: {
            if self.current_token.is_empty() {
                break 'skip;
            }

            if self.previous_char.is_none() && !self.current_token.is_empty() {
                if self.current_token == "```" {
                    self.renderer.push_token(token::Token::BeginCode)?;
                    self.current_token.clear();
                    self.mode_func = Self::analyse_code_block;
                    return Ok(());
                } else if self.current_token.len() >= 3 && current_char == '\n' && (self.current_token.chars().all(|c| c == '-') || self.current_token.chars().all(|c| c == '_')) {
                    self.renderer.push_token(token::Token::Line)?;
                    self.current_token.clear();
                    return Ok(());
                } else if self.current_token.chars().all(|c| c == '#' ) && current_char == ' ' {
                    let level = self.current_token.len().into();
                    self.renderer.push_token(token::Token::Heading(level))?;
                    self.current_token.clear();
                    return Ok(());
                }
                self.previous_char = self.current_token.chars().last();
            }
            let check_char = |c: char| !(c.is_alphanumeric() || ['*', '_', '`'].contains(&c));
            let is_begin = matches!(self.previous_char.map(check_char), Some(true) | None);
            let is_end = check_char(current_char); // note: newline MUST resets state, so no need to check
            if is_begin == is_end && !is_begin {
                break 'skip;
            }
            let inline_style = match self.current_token.as_str() {
                "*" => token::InlineStyleToken::OneStar,
                "**" => token::InlineStyleToken::TwoStars,
                "***" => token::InlineStyleToken::ThreeStars,
                "_" => token::InlineStyleToken::OneDash,
                "__" => token::InlineStyleToken::TwoDashes,
                "`" => token::InlineStyleToken::OneQuote,
                _ => break 'skip,
            };
            let inline_style = match self.inline_style_tokens.last() {
                Some(v) if v == &inline_style => token::Marker::End(inline_style),
                _ => token::Marker::Begin(inline_style),
            };
            match &inline_style {
                token::Marker::Begin(inline_style) => self.inline_style_tokens.push(inline_style.clone()),
                token::Marker::End(_) => { self.inline_style_tokens.pop(); },
            }
            self.push_current_text()?;
            self.renderer.push_token(token::Token::InlineStyle(inline_style))?;
            self.current_token.clear();
        }
        if !self.current_token.is_empty() {
            self.current_text.push_str(&self.current_token);
            self.current_token.clear();
        }
        if print_current_char {
            self.current_text.push(current_char);
        }
        Ok(())
    }
    

    fn push_current_text(&mut self) -> Result<(), ParseError<R::Error>> {
        if !self.current_text.is_empty() {
            let mut s = String::new();
            std::mem::swap(&mut s, &mut self.current_text);
            self.renderer.push_token(token::Token::Text(s))?
        }
        Ok(())
    }
}