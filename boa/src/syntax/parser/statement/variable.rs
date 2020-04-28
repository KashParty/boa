// use super::lexical_declaration_continuation;
use crate::{
    syntax::{
        ast::{keyword::Keyword, node::Node, punc::Punctuator, token::TokenKind},
        parser::{
            expression::Initializer, AllowAwait, AllowIn, AllowYield, Cursor, ParseError,
            ParseResult, TokenParser,
        },
    },
    Interner, Sym,
};

/// Variable statement parsing.
///
/// A varible statement contains the `var` keyword.
///
/// More information:
///  - [MDN documentation][mdn]
///  - [ECMAScript specification][spec]
///
/// [mdn]: https://developer.mozilla.org/en-US/docs/Web/JavaScript/Reference/Statements/var
/// [spec]: https://tc39.es/ecma262/#prod-VariableStatement
#[derive(Debug, Clone, Copy)]
pub(in crate::syntax::parser::statement) struct VariableStatement {
    allow_yield: AllowYield,
    allow_await: AllowAwait,
}

impl VariableStatement {
    /// Creates a new `VariableStatement` parser.
    pub(in crate::syntax::parser::statement) fn new<Y, A>(allow_yield: Y, allow_await: A) -> Self
    where
        Y: Into<AllowYield>,
        A: Into<AllowAwait>,
    {
        Self {
            allow_yield: allow_yield.into(),
            allow_await: allow_await.into(),
        }
    }
}

impl TokenParser for VariableStatement {
    type Output = Node;

    fn parse(self, cursor: &mut Cursor<'_>, interner: &mut Interner) -> ParseResult {
        cursor.expect(Keyword::Var, "variable statement", interner)?;

        let decl_list = VariableDeclarationList::new(true, self.allow_yield, self.allow_await)
            .parse(cursor, interner)?;

        cursor.expect_semicolon(false, "variable statement", interner)?;

        Ok(decl_list)
    }
}

/// Variable declaration list parsing.
///
/// More information:
///  - [MDN documentation][mdn]
///  - [ECMAScript specification][spec]
///
/// [mdn]: https://developer.mozilla.org/en-US/docs/Web/JavaScript/Reference/Statements/var
/// [spec]: https://tc39.es/ecma262/#prod-VariableDeclarationList
#[derive(Debug, Clone, Copy)]
pub(in crate::syntax::parser::statement) struct VariableDeclarationList {
    allow_in: AllowIn,
    allow_yield: AllowYield,
    allow_await: AllowAwait,
}

impl VariableDeclarationList {
    /// Creates a new `VariableDeclarationList` parser.
    pub(in crate::syntax::parser::statement) fn new<I, Y, A>(
        allow_in: I,
        allow_yield: Y,
        allow_await: A,
    ) -> Self
    where
        I: Into<AllowIn>,
        Y: Into<AllowYield>,
        A: Into<AllowAwait>,
    {
        Self {
            allow_in: allow_in.into(),
            allow_yield: allow_yield.into(),
            allow_await: allow_await.into(),
        }
    }
}

impl TokenParser for VariableDeclarationList {
    type Output = Node;

    fn parse(self, cursor: &mut Cursor<'_>, interner: &mut Interner) -> ParseResult {
        let mut list = Vec::new();

        loop {
            list.push(
                VariableDeclaration::new(self.allow_in, self.allow_yield, self.allow_await)
                    .parse(cursor, interner)?,
            );

            match cursor.peek_semicolon(false) {
                (true, _) => break,
                (false, Some(tk)) if tk.kind == TokenKind::Punctuator(Punctuator::Comma) => {
                    let _ = cursor.next();
                }
                (false, Some(tk)) => {
                    return Err(ParseError::expected(
                        vec![
                            Punctuator::Semicolon.to_string(),
                            Punctuator::Comma.to_string(),
                        ],
                        tk.display(interner).to_string(),
                        tk.pos,
                        "lexical declaration",
                    ))
                }
                _ => unreachable!(),
            }
        }

        Ok(Node::VarDecl(list))
    }
}

/// Reads an individual variable declaration.
///
/// More information:
///  - [ECMAScript specification][spec]
///
/// [spec]: https://tc39.es/ecma262/#prod-VariableDeclaration
#[derive(Debug, Clone, Copy)]
struct VariableDeclaration {
    allow_in: AllowIn,
    allow_yield: AllowYield,
    allow_await: AllowAwait,
}

impl VariableDeclaration {
    /// Creates a new `VariableDeclaration` parser.
    fn new<I, Y, A>(allow_in: I, allow_yield: Y, allow_await: A) -> Self
    where
        I: Into<AllowIn>,
        Y: Into<AllowYield>,
        A: Into<AllowAwait>,
    {
        Self {
            allow_in: allow_in.into(),
            allow_yield: allow_yield.into(),
            allow_await: allow_await.into(),
        }
    }
}

impl TokenParser for VariableDeclaration {
    type Output = (Sym, Option<Node>);

    fn parse(
        self,
        cursor: &mut Cursor<'_>,
        interner: &mut Interner,
    ) -> Result<Self::Output, ParseError> {
        let tok = cursor.next().ok_or(ParseError::AbruptEnd)?;
        let name = if let TokenKind::Identifier(name) = tok.kind {
            name.clone()
        } else {
            return Err(ParseError::expected(
                vec![String::from("identifier")],
                tok.display(interner).to_string(),
                tok.pos,
                "variable declaration",
            ));
        };

        match cursor.peek(0) {
            Some(tk) if tk.kind == TokenKind::Punctuator(Punctuator::Assign) => Ok((
                name,
                Some(
                    Initializer::new(self.allow_in, self.allow_yield, self.allow_await)
                        .parse(cursor, interner)?,
                ),
            )),
            _ => Ok((name, None)),
        }
    }
}
