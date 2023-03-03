mod base_types {
    pub struct Resource(pub u32);
    pub struct ResourceToken(pub u32);
    pub struct Turn(pub u32);
}

mod base_traits {
    use crate::base_types::{Resource, ResourceToken};

    pub trait Buildable {
        fn build() -> Self;
    }

    pub trait Playable {
        fn play_cost(&self) -> Resource {Resource(1)}
    }

    pub trait Damageable {
        const BASE_HEALTH: u32 = 10;
        fn damage(&mut self, dmg: u32);
    }

    pub trait Repairable: Damageable {
        fn repair(self) -> Self;
        fn repair_amount(&self) -> u32 {5}
    }

    pub trait Scrappable {
        const SCRAP_YIELD: ResourceToken = ResourceToken(2);
        fn scrap(&self) -> ResourceToken {Self::SCRAP_YIELD}
    }
}

mod creatures {
    use crate::base_traits::Buildable;

    pub enum CreatureType {
        Flyer,
        Plague,
        Scavanger,
        Destroyer,
    }

    pub trait Creature: Buildable {
        fn creature_type(&self) -> CreatureType;
        fn damage(&self) -> u32;
    }

    pub struct Bird {}

    impl Buildable for Bird {
        fn build() -> Self {
            Bird {}
        }
    }

    impl Creature for Bird {
        fn creature_type(&self) -> CreatureType {CreatureType::Flyer}
        fn damage(&self) -> u32 {1}
    }

    pub struct Rat {}

    impl Buildable for Rat {
        fn build() -> Self {
            Rat {} 
        }
    }

    impl Creature for Rat {
        fn creature_type(&self) -> CreatureType {CreatureType::Plague}
        fn damage(&self) -> u32 {1}
    }

    pub struct Squirrel {}

    impl Buildable for Squirrel {
        fn build() -> Self {
            Squirrel {}
        }
    }

    impl Creature for Squirrel {
        fn creature_type(&self) -> CreatureType {
            CreatureType::Scavanger
        }

        fn damage(&self) -> u32 {
            1
        }
    }

    pub struct Monkey {}

    impl Buildable for Monkey {
        fn build() -> Self {
            Monkey {}
        }
    }

    impl Creature for Monkey {
        fn creature_type(&self) -> CreatureType {
            CreatureType::Destroyer
        }

        fn damage(&self) -> u32 {
            1
        }
    }
}

mod generators {
    use std::cmp::min;
    use crate::base_traits::{Buildable, Damageable, Repairable, Playable};
    use crate::base_types::{Resource, Turn};
    use crate::creatures;

    enum GenType {
        Creature
    }

    trait Generator: Damageable + Playable + Buildable + Repairable
    {
        const GEN_TYPE: GenType;
        type Item: Buildable;
        fn gen(&self) -> Self::Item {Self::Item::build()}
        fn gen_time(&self) -> Turn {Turn(1)}
        fn gen_cost(&self) -> Resource {Resource(1)}
    }

    pub struct BirdGenerator {
        health: u32
    }

    impl Buildable for BirdGenerator {
        fn build() -> Self {
            BirdGenerator { health: BirdGenerator::BASE_HEALTH }
        }
    }

    impl Damageable for BirdGenerator {
        fn damage(&mut self, dmg: u32) {
            self.health -= dmg
        }
    }

    impl Repairable for BirdGenerator {
        fn repair(self) -> Self {
            let repaired = self.health + self.repair_amount();
            Self { health: min(Self::BASE_HEALTH, repaired)} 
        }
    }

    impl Playable for BirdGenerator {}

    impl Generator for BirdGenerator {
        const GEN_TYPE: GenType = GenType::Creature;
        type Item = creatures::Bird;
    }

    pub struct RatGenerator {
        health: u32
    }

    impl Buildable for RatGenerator {
        fn build() -> Self {
            RatGenerator { health: Self::BASE_HEALTH }
        }
    }

    impl Damageable for RatGenerator {
        fn damage(&mut self, dmg: u32) {
            self.health -= dmg
        }
    }

    impl Repairable for RatGenerator {
        fn repair(self) -> Self {
            let repaired = self.health + self.repair_amount();
            Self { health: min(Self::BASE_HEALTH, repaired)} 
        }
    }

    impl Playable for RatGenerator {}

    impl Generator for RatGenerator {
        const GEN_TYPE: GenType = GenType::Creature;
        type Item = creatures::Rat;
    }

    pub struct SquirrelGenerator {
        health: u32
    }

    impl Buildable for SquirrelGenerator {
        fn build() -> Self {
             SquirrelGenerator { health: SquirrelGenerator::BASE_HEALTH }
        }
    }

    impl Damageable for SquirrelGenerator {
        fn damage(&mut self, dmg: u32) {
            self.health -= dmg
        }
    }

    impl Repairable for SquirrelGenerator {
        fn repair(self) -> Self {
            let repaired = self.health + self.repair_amount();
            Self { health: min(Self::BASE_HEALTH, repaired)} 
        }
    }

    impl Playable for SquirrelGenerator {}

    impl Generator for SquirrelGenerator {
        const GEN_TYPE: GenType = GenType::Creature;
        type Item = creatures::Squirrel;
    }

    pub struct MonkeyGenerator {
        health: u32
    }

    impl Buildable for MonkeyGenerator {
        fn build() -> Self {
            MonkeyGenerator { health: MonkeyGenerator::BASE_HEALTH }
        }
    }

    impl Damageable for MonkeyGenerator {
        fn damage(&mut self, dmg: u32) {
            self.health -= dmg
        }
    }

    impl Repairable for MonkeyGenerator {
        fn repair(self) -> Self {
            let repaired = self.health + self.repair_amount();
            Self { health: min(Self::BASE_HEALTH, repaired)} 
        }
    }

    impl Playable for MonkeyGenerator {}

    impl Generator for MonkeyGenerator {
        const GEN_TYPE: GenType = GenType::Creature;
        type Item = creatures::Monkey;
    }
}

mod constructs {
    use std::cmp::min;

    use crate::base_traits::{Buildable, Damageable, Repairable, Playable, Scrappable};

    trait Construct: Playable + Buildable + Damageable + Repairable + Scrappable{
        const EROSION: u32 = 1;
    }

    struct Wall {
        health: u32
    }

    impl Buildable for Wall {
        fn build() -> Self {
            Wall { health: Wall::BASE_HEALTH }
        }
    }

    impl Playable for Wall {}

    impl Damageable for Wall {
        const BASE_HEALTH: u32 = 10;
        fn damage(&mut self, dmg: u32) {
            self.health -= dmg
        }
    }

    impl Repairable for Wall {
        fn repair(self) -> Self {
            let repaired = self.health + self.repair_amount();
            Self { health: min(Self::BASE_HEALTH, repaired)} 
        }
    }

    impl Scrappable for Wall {}

    impl Construct for Wall {}
}

mod evocations {
    use crate::base_traits::{Buildable, Playable};
    use crate::base_types::Resource;

    enum EvocationType {
        Damage, 
    }

    trait Evocation: Playable + Buildable {
        const TYPE: EvocationType;
    }

    struct Lightning {
        damage: u32
    }

    impl Playable for Lightning {
        fn play_cost(&self) -> crate::base_types::Resource {
           Resource(2) 
        }
    }

    impl Buildable for Lightning {
        fn build() -> Self {
            Lightning { damage: 10 }
        }
    }

    impl Evocation for Lightning {
       const TYPE: EvocationType = EvocationType::Damage; 
    }
}
