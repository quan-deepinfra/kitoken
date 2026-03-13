//! Encoder for the tokenizer.

use alloc::borrow::Cow;
use alloc::vec::Vec;
use core::borrow::Borrow;
use core::fmt::Debug;
use core::ops::Deref;

use crate::{Model, Token, TokenId};

mod bytepair;
mod unigram;
mod wordpiece;

pub(crate) use bytepair::*;
pub(crate) use unigram::*;
pub(crate) use wordpiece::*;

/// Errors encountered during encoding.
#[non_exhaustive]
#[derive(Debug, thiserror::Error)]
pub enum EncodeError {
    /// A piece could not be encoded.
    #[error("invalid piece {0:?}")]
    InvalidPiece(Vec<u8>),
}

/// Part of a text.
#[derive(Debug, Clone, PartialEq, Eq)]
pub(crate) struct TextPart<'a> {
    pub text:    Cow<'a, str>,
    pub special: TokenId,
}
impl Borrow<[u8]> for TextPart<'_> {
    #[inline(always)]
    fn borrow(&self) -> &[u8] {
        self.text.as_bytes()
    }
}
impl Deref for TextPart<'_> {
    type Target = [u8];

    #[inline(always)]
    fn deref(&self) -> &Self::Target {
        self.text.as_bytes()
    }
}

/// Encoder for the tokenizer.
pub(crate) trait Encoder: Debug + Send + Sync + 'static {
    /// Encodes the given parts into a sequence of tokens.
    ///
    /// If `encode_specials` is `true`, control tokens are tokenized with their ids, otherwise they are tokenized with the regular vocabulary.
    ///
    /// Returns an error if no token for a part exists in the encoder, and the configuration has no unknown token or skip fallback set.
    fn encode(&self, text: &str, parts: &mut [TextPart]) -> Result<Vec<TokenId>, EncodeError>;

    /// Returns the vocabulary and scores.
    fn model(&self) -> Model;

    /// Looks up a piece in the vocabulary as a single token.
    /// Returns `Some(token_id)` if the piece maps to exactly one token, `None` otherwise.
    fn lookup_token(&self, _bytes: &[u8]) -> Option<TokenId> {
        None
    }

    /// Encodes a single piece of bytes directly into the result buffer.
    /// This avoids the overhead of creating TextPart and allocating intermediate Vecs.
    fn encode_piece(&self, piece: &[u8], result: &mut Vec<TokenId>) -> Result<(), EncodeError> {
        let text = core::str::from_utf8(piece).unwrap_or("");
        let mut parts = [TextPart {
            text:    Cow::Borrowed(text),
            special: Token::INVALID,
        }];
        let tokens = self.encode(text, &mut parts)?;
        result.extend_from_slice(&tokens);
        Ok(())
    }
}
