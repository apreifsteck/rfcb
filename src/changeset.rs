use sea_query::{Alias, Iden, PostgresQueryBuilder, Query, SimpleExpr};
use sqlx::postgres::{PgPool, PgRow};
use sqlx::{Error, Executor};
use std::fmt::{Debug, Display};
use std::mem::{self, discriminant};
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

impl<'a, Attr, Data> Changeset<Attr, Data>
where
    Attr: Eq + Iden + Valuable + Clone + 'static + std::fmt::Debug,
    for<'b> Data: TableRef + sqlx::FromRow<'b, PgRow>,
{
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
    pub fn validate(&mut self) -> Result<&mut Self, Vec<&'static str>> {
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
            Err(errors_acc)
        } else {
            self.valid = true;
            Ok(self)
        }
    }
    // pub async fn insert(self, pool: &PgPool) -> Result<D, Error> {
    //     pool.fetch_one(self.build_query().as_str())
    //         .await
    //         .map(|row| Data::from_row(&row).unwrap())
    // }
    // fn build_query(&self) -> String {
    //     Query::insert()
    //         .into_table(Data::table_ref())
    //         .columns(self.changes.clone())
    //         .values_panic(self.changes.clone().iter().map(|x| x.value()))
    //         .to_string(PostgresQueryBuilder)
    // }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::entities::rfc::{RFCAttrs, Status, RFC};
    //TODO create a bogus enum and stuff here
    fn setup() -> Changeset<RFCAttrs, RFC> {
        let cs: Changeset<RFCAttrs, RFC> = Changeset::new(None);
        cs
    }
    mod add_change_tests {
        use super::super::*;
        use crate::entities::rfc::{RFCAttrs, Status, RFC};

        #[test]
        #[should_panic]
        fn will_not_accept_duplicate_change_types() {
            let mut cs = super::setup();
            cs.add_change(RFCAttrs::Status(Status::Active), None);
            cs.add_change(RFCAttrs::Status(Status::Active), None);
        }

        #[test]
        fn will_accept_distinct_change_types() {
            let mut cs = super::setup();
            cs.add_change(RFCAttrs::Status(Status::Active), None);
            cs.add_change(RFCAttrs::Proposal("Whatever".to_string()), None);
            let changes: Vec<RFCAttrs> = cs.changes.iter().map(|change| change.0.clone()).collect();

            assert_eq!(
                vec![
                    RFCAttrs::Status(Status::Active),
                    RFCAttrs::Proposal("Whatever".to_string())
                ],
                changes
            )
        }
    }
    mod validate_tests {
        use std::panic::panic_any;

        use super::super::*;
        use crate::{
            changeset,
            entities::rfc::{RFCAttrs, Status, RFC},
        };
        #[test]
        fn fails_if_validations_not_passing() -> Result<(), &'static str> {
            let mut cs = super::setup();
            let too_long = |proposal: &RFCAttrs| {
                let prop = panic_if_not_match!(<RFCAttrs>::Proposal(prop), proposal);
                if prop.len() > 1 {
                    Err("Proposal too long")
                } else {
                    Ok(())
                }
            };
            cs.add_change(
                RFCAttrs::Proposal("Whatever".to_string()),
                Some(vec![Box::new(too_long)]),
            );
            if let Err(messages) = cs.validate() {
                assert_eq!(messages, vec!["Proposal too long"]);
                Ok(())
            } else {
                panic!()
            }
        }
    }
    //
    // mod insert_tests {
    //     use sqlx::PgPool;
    //     #[sqlx::test]
    //     fn will_not_insert_unless_valid(_pool: PgPool) {
    //         assert!(false)
    //     }
    //     #[sqlx::test]
    //     fn renders_back_sane_error_if_borked(_pool: PgPool) {
    //         assert!(false)
    //     }
    // }
}
