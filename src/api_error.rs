use crate::entities::vote::VoteError;
use crate::repo::ChangeError;

#[derive(Debug)]
pub enum APIError<'a> {
    ChangeError(ChangeError),
    VoteError(VoteError<'a>),
}

impl<'a> From<ChangeError> for APIError<'a> {
    fn from(value: ChangeError) -> Self {
        APIError::ChangeError(value)
    }
}

impl<'a> From<VoteError<'a>> for APIError<'a> {
    fn from(value: VoteError<'a>) -> Self {
        APIError::VoteError(value)
    }
}
