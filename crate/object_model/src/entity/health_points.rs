use std::{
    cmp::Ordering,
    ops::{Add, AddAssign, Sub, SubAssign},
};

use amethyst::ecs::{storage::VecStorage, Component};
use derivative::Derivative;
use derive_deref::{Deref, DerefMut};
use derive_more::{Add, AddAssign, Display, From, Sub, SubAssign};
use specs_derive::Component;

/// Health points of an object.
#[derive(
    Add,
    AddAssign,
    Sub,
    SubAssign,
    Clone,
    Component,
    Copy,
    Debug,
    Deref,
    DerefMut,
    Derivative,
    Display,
    From,
    PartialEq,
    Eq,
    PartialOrd,
    Ord,
)]
#[storage(VecStorage)]
#[derivative(Default)]
pub struct HealthPoints(#[derivative(Default(value = "100"))] pub u32);

impl Add<u32> for HealthPoints {
    type Output = Self;

    fn add(self, other: u32) -> Self {
        HealthPoints(self.0 + other)
    }
}

impl AddAssign<u32> for HealthPoints {
    fn add_assign(&mut self, other: u32) {
        *self = HealthPoints(self.0 + other);
    }
}

impl Sub<u32> for HealthPoints {
    type Output = Self;

    fn sub(self, other: u32) -> Self {
        HealthPoints(self.0 - other)
    }
}

impl SubAssign<u32> for HealthPoints {
    fn sub_assign(&mut self, other: u32) {
        *self = HealthPoints(self.0 - other);
    }
}

impl PartialOrd<u32> for HealthPoints {
    fn partial_cmp(&self, other: &u32) -> Option<Ordering> {
        Some(self.0.cmp(other))
    }
}

impl PartialEq<u32> for HealthPoints {
    fn eq(&self, other: &u32) -> bool {
        self.0 == *other
    }
}