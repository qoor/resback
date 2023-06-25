use rand::{rngs::ThreadRng, Rng};

/// List of Korean adjective words
pub const ADJECTIVES: &[&str] = &include!("adjectives.in");

/// List of animals in Korean
pub const ANIMALS: &[&str] = &include!("animals.in");

/// List of characters in Korean
pub const CHARACTERS: &[&str] = &include!("characters.in");

/// List of heros in Korean
pub const HEROS: &[&str] = &include!("heros.in");

/// List of monsters in Korean
pub const MONSTERS: &[&str] = &include!("monsters.in");

/// A noun type for the `Generator`
pub enum NounType {
    Animal,
    Character,
    Hero,
    Monster,
}

/// A custom version of `names::Generator`, providing Korean names
pub struct KoreanGenerator<'a> {
    animal_generator: names::Generator<'a>,
    character_generator: names::Generator<'a>,
    hero_generator: names::Generator<'a>,
    monster_generator: names::Generator<'a>,

    rng: ThreadRng,
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
            character_generator: names::Generator::with_noun_type(
                NounType::Character,
                names::Name::from(naming),
            ),
            hero_generator: names::Generator::with_noun_type(
                NounType::Hero,
                names::Name::from(naming),
            ),
            monster_generator: names::Generator::with_noun_type(
                NounType::Monster,
                names::Name::from(naming),
            ),

            rng: ThreadRng::default(),
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
        match self.rng.gen_range(0..4) {
            0 => self.animal_generator.next_pretty(),
            1 => self.character_generator.next_pretty(),
            2 => self.hero_generator.next_pretty(),
            3 => self.monster_generator.next_pretty(),
            _ => None,
        }
    }
}

pub trait KoreanName<'a> {
    /// Returns new `Generator` that can generate names using the list of nouns
    /// we've defined
    fn with_noun_type(noun_type: NounType, naming: names::Name) -> names::Generator<'a>;

    /// Returns Korean nickname with pretty format (ex. "외제차를 뽑은 꼬봉")
    /// instead of `names::Generator`'s name (ex. "외제차를 뽑은-꼬봉")
    fn next_pretty(self: &mut Self) -> Option<String>;
}

impl<'a> KoreanName<'a> for names::Generator<'a> {
    fn with_noun_type(noun_type: NounType, naming: names::Name) -> names::Generator<'a> {
        match noun_type {
            NounType::Animal => names::Generator::new(ADJECTIVES, ANIMALS, naming),
            NounType::Character => names::Generator::new(ADJECTIVES, CHARACTERS, naming),
            NounType::Hero => names::Generator::new(ADJECTIVES, HEROS, names::Name::Plain),
            NounType::Monster => names::Generator::new(ADJECTIVES, MONSTERS, names::Name::Plain),
        }
    }

    fn next_pretty(self: &mut Self) -> Option<String> {
        match self.next() {
            Some(name) => Some(name.replacen('-', " ", 1)),
            None => None,
        }
    }
}
