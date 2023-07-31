// Copyright 2023. The resback authors all rights reserved.

use rand::rngs::ThreadRng;

/// List of Korean adjective words
pub const ADJECTIVES: &[&str] = &include!("adjectives.in");

/// List of animals in Korean
pub const ANIMALS: &[&str] = &include!("animals.in");

/// A noun type for the `Generator`
pub enum NounType {
    Animal,
}

/// A custom version of `names::Generator`, providing Korean names
pub struct KoreanGenerator<'a> {
    animal_generator: names::Generator<'a>,

    _rng: ThreadRng,
}

/// A naming strategy for `Generator`
/// It redefines non-copyable `users::Name` to implement the `Clone` and `Copy`
/// traits.
#[derive(Debug, Clone, Copy)]
pub enum Naming {
    Plain,
    Numbered,
}

impl From<Naming> for names::Name {
    fn from(value: Naming) -> Self {
        match value {
            Naming::Plain => names::Name::Plain,
            Naming::Numbered => names::Name::Numbered,
        }
    }
}

impl<'a> KoreanGenerator<'a> {
    pub fn new(naming: Naming) -> Self {
        Self {
            animal_generator: names::Generator::with_noun_type(
                NounType::Animal,
                names::Name::from(naming),
            ),

            _rng: ThreadRng::default(),
        }
    }
}

impl<'a> Default for KoreanGenerator<'a> {
    fn default() -> Self {
        KoreanGenerator::new(Naming::Plain)
    }
}

impl<'a> Iterator for KoreanGenerator<'a> {
    type Item = String;

    fn next(&mut self) -> Option<Self::Item> {
        self.animal_generator.next_pretty()
    }
}

pub trait KoreanName<'a> {
    /// Returns new `Generator` that can generate names using the list of nouns
    /// we've defined
    fn with_noun_type(noun_type: NounType, naming: names::Name) -> names::Generator<'a>;

    /// Returns Korean nickname with pretty format (ex. "외제차를 뽑은 꼬봉")
    /// instead of `names::Generator`'s name (ex. "외제차를 뽑은-꼬봉")
    fn next_pretty(&mut self) -> Option<String>;
}

impl<'a> KoreanName<'a> for names::Generator<'a> {
    fn with_noun_type(noun_type: NounType, naming: names::Name) -> names::Generator<'a> {
        match noun_type {
            NounType::Animal => names::Generator::new(ADJECTIVES, ANIMALS, naming),
        }
    }

    fn next_pretty(&mut self) -> Option<String> {
        self.next().map(|name| name.replacen('-', " ", 1))
    }
}
