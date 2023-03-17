use crate::repo::{ChangeError, Insertable, Validatable};
use sea_query::{Alias, Iden, PostgresQueryBuilder, Query, SimpleExpr};
use std::mem;
use std::vec::Vec;

pub trait TableRef {
    fn table_ref() -> Alias;
}
pub trait Valuable {
    fn value(&self) -> SimpleExpr;
}

macro_rules! panic_if_not_match {
    (
        <$enum:ty>::$variant:ident($inner_val:ident), $value:ident
    ) => {{
        type Enum = $enum;
        if let Enum::$variant($inner_val) = $value {
            $inner_val
        } else {
            panic!("Pattern does not match")
        }
    }};
}

// TODO see if you can find a way to keep the closures off the heap
type Validations<T> = Vec<Box<dyn Fn(&T) -> Result<(), &'static str>>>;
struct Change<Attr>(Attr, Validations<Attr>);
impl<Attr> Change<Attr> {
    fn new(attr: Attr, validations: Option<Validations<Attr>>) -> Self {
        if let Some(validation_vec) = validations {
            Self(attr, validation_vec)
        } else {
            Self(attr, vec![])
        }
    }
}
pub struct Changeset<Attr, Data> {
    changes: Vec<Change<Attr>>,
    data: Option<Data>,
    valid: bool,
}

impl<'a, Attr, Data> Changeset<Attr, Data> {
    pub fn new(backing_data: Option<Data>) -> Self {
        Changeset {
            changes: Vec::new(),
            data: backing_data,
            valid: false,
        }
    }
    pub fn add_change(
        &mut self,
        change: Attr,
        validations: Option<Validations<Attr>>,
    ) -> &mut Self {
        match self
            .changes
            .iter()
            .find(|&cur_change| mem::discriminant(&cur_change.0) == mem::discriminant(&change))
        {
            Some(_) => panic!("tried to add two of the same type of change"),
            None => self.changes.push(Change::new(change, validations)),
        }
        self
    }
}
impl<Attr, Data> Validatable for Changeset<Attr, Data> {
    fn validate(&self) -> Result<(), ChangeError> {
        let mut errors_acc = vec![];

        let errors = self.changes.iter().fold(
            &mut errors_acc,
            |error_list, Change(attribute, validations)| {
                let mut errors_for_element: Vec<&str> = validations
                    .iter()
                    .filter_map(|validation| validation(attribute).err())
                    .collect();
                error_list.append(&mut errors_for_element);
                error_list
            },
        );
        if errors.len() > 0 {
            Err(ChangeError::ValidationError(errors_acc))
        } else {
            Ok(())
        }
    }
}
impl<Attr, Data> Insertable for Changeset<Attr, Data>
where
    Attr: Iden + Valuable + Clone + 'static + std::fmt::Debug,
    Data: TableRef,
{
    fn to_sql(&self) -> String {
        let changes: Vec<Attr> = self
            .changes
            .iter()
            .map(|Change(attr, _)| attr.clone())
            .collect();

        Query::insert()
            .into_table(Data::table_ref())
            .columns(changes.clone())
            .values_panic(changes.clone().iter().map(|x| x.value()))
            .returning(Query::returning().all())
            .to_string(PostgresQueryBuilder)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    #[derive(Debug, Clone, PartialEq, Eq, Iden)]
    enum BogusAttrs {
        Field(String),
        OtherField(String),
    }
    impl Valuable for BogusAttrs {
        fn value(&self) -> SimpleExpr {
            match self {
                Self::Field(val) => val.into(),
                Self::OtherField(val) => val.into(),
            }
        }
    }

    #[derive(Debug, sqlx::FromRow)]
    struct Bogus {}
    impl TableRef for Bogus {
        fn table_ref() -> Alias {
            Alias::new("boguses")
        }
    }
    fn setup() -> Changeset<BogusAttrs, Bogus> {
        let cs: Changeset<BogusAttrs, Bogus> = Changeset::new(None);
        cs
    }
    mod add_change_tests {
        use super::*;

        #[test]
        #[should_panic]
        fn will_not_accept_duplicate_change_types() {
            let mut cs = super::setup();
            cs.add_change(BogusAttrs::Field("".to_string()), None);
            cs.add_change(BogusAttrs::Field("".to_string()), None);
        }

        #[test]
        fn will_accept_distinct_change_types() {
            let mut cs = super::setup();
            cs.add_change(BogusAttrs::Field("Whatever".to_string()), None);
            cs.add_change(BogusAttrs::OtherField("Whatever".to_string()), None);
            let changes: Vec<BogusAttrs> =
                cs.changes.iter().map(|change| change.0.clone()).collect();

            assert_eq!(
                vec![
                    BogusAttrs::Field("Whatever".to_string()),
                    BogusAttrs::OtherField("Whatever".to_string())
                ],
                changes
            )
        }
    }
    mod validate_tests {
        use super::*;

        #[test]
        fn fails_if_validations_not_passing() {
            let mut cs = super::setup();
            let too_long = |given_field: &BogusAttrs| {
                let field = panic_if_not_match!(<BogusAttrs>::Field(field), given_field);
                if field.len() > 1 {
                    Err("field too long")
                } else {
                    Ok(())
                }
            };
            cs.add_change(
                BogusAttrs::Field("Whatever".to_string()),
                Some(vec![Box::new(too_long)]),
            );
            if let Err(ChangeError::ValidationError(messages)) = cs.validate() {
                assert_eq!(messages, vec!["field too long"]);
            } else {
                panic!()
            }
        }

        #[test]
        fn fails_if_there_are_no_changes() {}
    }
}
