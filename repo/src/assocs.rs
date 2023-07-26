use super::attributes::{Entity, Queryable};
use sqlx::Database;
pub trait AssociationLoader<T, D>
where
    D: Database,
    T: Entity<D>,
{
    fn query(&self) -> &dyn Queryable<D, Output = T::Record>;
    fn multiplicity(&self) -> Multiplicity;
    fn load(&mut self, data: Loadable<T>);
}

pub trait Association<'a, T, D>: AssociationLoader<T, D> + PartialEq + Eq
where
    D: Database,
    T: Entity<D> + 'a,
{
    type Unwrapped;
    fn is_loaded(&self) -> bool;
    fn unwrap(&'a self) -> Self::Unwrapped;
}

#[derive(Debug)]
pub struct HasMany<T, D>
where
    D: Database,
    T: Entity<D>,
{
    data: Loadable<T>,
    load_query: Box<dyn Queryable<D, Output = T::Record>>,
}

impl<T, D> AssociationLoader<T, D> for HasMany<T, D>
where
    D: Database,
    T: Entity<D>,
{
    fn query(&self) -> &dyn Queryable<D, Output = T::Record> {
        self.load_query.as_ref()
    }

    fn multiplicity(&self) -> Multiplicity {
        Multiplicity::Many
    }
    fn load(&mut self, data: Loadable<T>) {
        self.data = data;
    }
}

impl<'a, T, D> Association<'a, T, D> for HasMany<T, D>
where
    D: Database,
    T: Entity<D> + PartialEq + Eq + 'a,
{
    type Unwrapped = Option<&'a [T]>;
    fn unwrap(&'a self) -> Self::Unwrapped {
        self.data.unwrap_to_inner_ref().unwrap_option_many()
    }

    fn is_loaded(&self) -> bool {
        match self.data {
            Loadable::NotLoaded => false,
            Loadable::Loaded(_) => true,
        }
    }
}

impl<T, D> HasMany<T, D>
where
    D: Database,
    T: Entity<D>,
{
    pub fn new(load_query: impl Queryable<D, Output = T::Record> + 'static) -> Self {
        Self {
            data: Loadable::NotLoaded,
            load_query: Box::new(load_query),
        }
    }
}
//
impl<T, D> PartialEq for HasMany<T, D>
where
    D: Database,
    T: Entity<D> + PartialEq + Eq,
{
    fn eq(&self, other: &Self) -> bool {
        self.data == other.data
    }
}

impl<T, D> Eq for HasMany<T, D>
where
    T: Entity<D> + Eq,
    D: Database,
{
}
#[derive(Debug, PartialEq, Eq)]
pub enum Loadable<T> {
    Loaded(Data<T>),
    NotLoaded,
}

impl<T> Loadable<T> {
    fn unwrap_to_inner_ref(&self) -> &Data<T> {
        if let Loadable::Loaded(t) = self {
            &t
        } else {
            panic!()
        }
    }
}
//
// Do I need both data and multiplicity?
#[derive(Clone, Copy, Debug)]
pub enum Multiplicity {
    Many,
    One,
}
//
#[derive(Debug, PartialEq, Eq)]
pub enum Data<T> {
    None,
    One(T),
    Many(Vec<T>),
}

impl<T> Data<T> {
    fn unwrap_option_many(&self) -> Option<&[T]> {
        match self {
            Self::None => None,
            Self::Many(ref d) => Some(d),
            _ => panic!(),
        }
    }
    fn unwrap_option_one(&self) -> Option<&T> {
        match self {
            Self::None => None,
            Self::One(ref d) => Some(d),
            _ => panic!(),
        }
    }
}
