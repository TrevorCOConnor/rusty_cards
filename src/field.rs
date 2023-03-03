mod cards {
    pub trait CreatureGen {
        fn gen(&self) -> Creature;
    }
    pub struct Generator {
        pub gen_cost: u32,
        creature_gen: Box<dyn CreatureGen>,
        primed: bool
    }
    pub enum ConstructType {
        Wall {blocks_friendly: bool},
        Trap {damage: u32},
    }
    pub struct Construct {
        construct_type: ConstructType,
        max_health: u32,
        health: u32,
        erosion: u32,
    }

    pub trait Evokable {
       fn evoke(&self, creature: Creature) -> Creature;
       fn evoke_mut(&self, creature: &mut Creature);
    }
    pub enum EvocationType {
        /// I am thinking this will be more for MetaData
        /// Like on the card it might say "Dmg(3)", "Stn(2)"
        /// It may be best to just have a displayable attribute to indicate this stuff... that
        /// would make it more dynamic...
        Damage{ dmg: u32 },
        Stun{ duration: u32},
    }
    pub struct Evocation {
        pub evocation_type: EvocationType,
        pub evocation: Box<dyn Evokable>
    }

    #[derive(Clone)]
    pub enum CreatureType {
        Flyer,
        Plague,
        Scavanger,
        Destroyer,
    }
    #[derive(Clone)]
    pub struct Creature {
        pub name: String,
        pub creature_type: CreatureType,
        pub health: u32,
        pub damage: u32,
    }

    pub struct Card {
        pub cost: u32,
        pub name: String,
        pub description: String,
        pub content: CardType,
    }

    pub enum CardType {
        Generator(Generator),
        Construct(Construct),
        Evocation(Evocation),
    }

    impl Evocation {
        pub fn new(evocation_type: EvocationType, evocation: Box<dyn Evokable>) -> Evocation {
            Evocation { evocation_type, evocation}
        }
    }

    impl Generator {
        pub fn new(cost: u32, creature_gen: Box<dyn CreatureGen>) -> Generator {
            Generator { gen_cost: cost, creature_gen, primed: false }
        }

        pub fn is_primed(&self) -> bool {
            self.primed
        }

        pub fn prime(&mut self) {
            self.primed = true
        }

        pub fn gen(&self) -> Creature {
            self.creature_gen.gen()
        }
    }

    impl Construct {
        pub fn new(construct_type: ConstructType, max_health: u32, starting_health: u32, erosion: u32) -> Construct {
            Construct { construct_type, max_health, health: starting_health, erosion }
        }

        pub fn apply_erosion(&mut self) {
            self.health -= self.erosion
        }

        pub fn erode(self) -> Self {
            Self {health: self.health - self.erosion, ..self}
        }

        pub fn broken(&self) -> bool {
            self.health == 0
        }

        pub fn blocks_friendly(&self) -> bool {
            if let ConstructType::Wall { blocks_friendly } = self.construct_type {
                blocks_friendly
            } else {
                false
            }
        }

        pub fn blocks(&self) -> bool {
            if let ConstructType::Wall { .. } = self.construct_type {
                true
            } else {
                false
            }
        }
    }

    impl Creature {
        pub fn flies(&self) -> bool {
            if let CreatureType::Flyer = self.creature_type {
                true
            } else {
                false
            }
        }

        pub fn alive(&self) -> bool {
            self.health > 0
        }

        pub fn inflict(self, dmg: u32) -> Self {
            Self {health: self.health - dmg, ..self}
        }

        pub fn inflict_mut(&mut self, dmg: u32) {
            self.health -= dmg
        }
    }
}

mod zones {
    use std::collections::VecDeque;

    use super::cards::{Card, Creature, Generator, Construct, Evocation, CreatureType};

    pub struct ResourcePool(u32);
    pub struct MaterialPool(u32);

    pub struct Hand {
        cards: VecDeque<Card>
    }

    pub struct Discard {
        cards: Vec<Card>
    }

    pub struct GeneratorZone {
        generators: VecDeque<Generator>
    }

    pub struct ConstructZone {
        constructs: VecDeque<Construct>,
        scheduled: Vec<Construct>,
    }

    pub struct EvocationZone {
        evocations: VecDeque<Evocation>,
        creatures: VecDeque<Creature>
    }

    pub struct CreatureZone {
        creatures: Vec<Creature>
    }

    pub struct Deck {
        cards: VecDeque<Card>
    }

    impl Hand {
        pub fn take(&mut self, index: usize) -> Card {
            self.cards.remove(index).expect("Card index should exist for hand")
        }

        pub fn borrow(&self, index: usize) -> Option<&Card> {
            self.cards.get(index)
        }

        pub fn add(&mut self, card: Card) {
            self.cards.push_front(card)
        }
    }

    impl Deck {
        pub fn draw(&mut self) -> Option<Card> {
            self.cards.pop_front()
        }

        pub fn shuffle(&mut self) {
            // implement later
        }
    }

    impl GeneratorZone {
        pub fn add(&mut self, gen: Generator) {
            self.generators.push_back(gen)
        }

        pub fn borrow(&mut self, index: usize) -> Option<&mut Generator> {
            self.generators.get_mut(index)
        }

        pub fn run(&self) -> Vec<Creature> {
            self.generators.iter().filter(|c| c.is_primed()).map(|c| c.gen()).collect()
        }
    }

    impl CreatureZone {
        pub fn drain(&mut self) -> Vec<Creature> {
            self.creatures.drain(..).collect()
        }

        pub fn load(&mut self, creatures: Vec<Creature>) {
            self.creatures.extend(creatures.into_iter());
        }

        pub fn add(&mut self, creature: Creature) {
            self.creatures.push(creature)
        }
    }

    impl ConstructZone {
        pub fn add(&mut self, con: Construct) {
            self.constructs.push_back(con)
        }

        pub fn run(&mut self) {
            self.constructs.extend(self.scheduled.drain(..))
        }

        pub fn apply_erosion(&mut self) {
            self.constructs = self.constructs.drain(..)
                .map(Construct::erode)
                .filter(|c| !c.broken()).collect()
        }

        pub fn deploy_creatures(&self, creatures: Vec<Creature>) -> (Vec<Creature>, Vec<Creature>) {
            let blockers: Vec<&Construct> = self.constructs.iter().filter(|c| c.blocks_friendly()).collect();
            if blockers.len() == 0 {
                (creatures, Vec::new())
            } else {
                creatures.into_iter().partition(Creature::flies)
            }
        }

        pub fn retreat_creatures(&self, creatures: Vec<Creature>) -> (Vec<Creature>, Vec<Creature>) {
            let blockers: Vec<&Construct> = self.constructs.iter().filter(|c| c.blocks()).collect();
            if blockers.len() == 0 {
                (creatures, Vec::new())
            } else {
                creatures.into_iter().partition(Creature::flies)
            }
        }

        pub fn invade_creatures(&self, creatures: Vec<Creature>) -> (Vec<Creature>, Vec<Creature>) {
            let blockers: Vec<&Construct> = self.constructs.iter().filter(|c| c.blocks_friendly()).collect();
            if blockers.len() == 0 {
                (creatures, Vec::new())
            } else {
                creatures.into_iter().partition(Creature::flies)
            }
        }
    }

    impl EvocationZone {
        pub fn add(&mut self, evo: Evocation) {
            self.evocations.push_back(evo)
        }

        pub fn load_creatures(&mut self, creatures: Vec<Creature>) {
            self.creatures.extend(creatures.into_iter())
        }

        pub fn drain_creatures(&mut self) -> Vec<Creature> {
            self.creatures.drain(..).collect()
        }

        fn apply_evocation(evo: Evocation, creatures: VecDeque<Creature>) -> VecDeque<Creature> {
            creatures.into_iter()
                .map(|c| evo.evocation.evoke(c))
                .filter(|c| !c.alive()).collect()
        }

        pub fn run(&mut self) {
            let creatures: VecDeque<Creature> = self.creatures.drain(..).collect();
            self.creatures = self.evocations.drain(..)
                .fold(creatures, |acc, evo| {
                    acc.into_iter()
                        .map(|c| evo.evocation.evoke(c))
                        .filter(|c| !c.alive()).collect()
                });
        }
    }

    impl ResourcePool {
        pub fn pay_cost(&mut self, cost: u32) -> bool {
            if cost <= self.0 {
                self.0 -= cost;
                true
            }
            else {
                false
            }
        }
    }
}

pub mod field {
    use super::{
        zones::{GeneratorZone, ConstructZone, EvocationZone, Discard, Hand, ResourcePool, MaterialPool, Deck, CreatureZone},
        cards::{Card, CardType}
    };

    pub struct Half {
        evocations: EvocationZone,
        constructs: ConstructZone,
        generators: GeneratorZone,
        creatures: CreatureZone,
        discard: Discard,
        hand: Hand,
        deck: Deck,
        materials: MaterialPool,
        resources: ResourcePool,
    }

    pub struct TheirHalf(Half); 

    pub struct MyHalf(Half);

    pub struct Field {
        theirs: TheirHalf,
        mine: MyHalf,
    }
    
    impl MyHalf {
        fn draw_card(&mut self) {
            if let Some(card) = self.0.deck.draw() {
                self.0.hand.add(card)
            }
        }

        fn place_card(&mut self, card: Card) {
            match card.content {
               CardType::Generator(gen) => self.0.generators.add(gen),
               CardType::Construct(con) => self.0.constructs.add(con),
               CardType::Evocation(evo) => self.0.evocations.add(evo),
            }
        }

        fn play_card_from_hand(&mut self, index: usize) {
            if let Some(card_ref) = self.0.hand.borrow(index) {
                if let true = self.0.resources.pay_cost(card_ref.cost) {
                    let card = self.0.hand.take(index);
                    self.place_card(card)
                }
            }
        }

        fn allocate_resources_to_generator(&mut self, index: usize) {
            if let Some(gen_ref) = self.0.generators.borrow(index) {
                if !gen_ref.is_primed() {
                    if let true = self.0.resources.pay_cost(gen_ref.gen_cost) {
                        gen_ref.prime();
                    }
                }
            }
        }

        fn deploy_creatures(&mut self) {
            let creatures = self.0.creatures.drain();
            let (advanced, blocked) = self.0.constructs.deploy_creatures(creatures);
            self.0.creatures.load(blocked);
            self.0.evocations.load_creatures(advanced);
        }

        fn retreat_creatures(&mut self) {
            let creatures = self.0.evocations.drain_creatures();
            let (retreated, blocked) = self.0.constructs.retreat_creatures(creatures);
            self.0.creatures.load(retreated);
            self.0.evocations.load_creatures(blocked);
        }

        fn run(&mut self) {
            self.0.constructs.run();
            self.deploy_creatures();
            self.0.evocations.run();
            self.0.generators.run();
            self.retreat_creatures();
            self.0.constructs.apply_erosion();
        }
    }
}

mod internal_generators {
    use super::cards::{CreatureGen, Creature, CreatureType};

    pub struct BirdGenerator {}

    impl CreatureGen for BirdGenerator {
        fn gen(&self) -> Creature {
            Creature {
                name: String::from("Bird"),
                creature_type: CreatureType::Flyer,
                damage: 1,
                health: 1,
            }
        }
    }

    struct RatGenerator {}

    impl CreatureGen for RatGenerator {
        fn gen(&self) -> Creature {
            Creature {
                name: String::from("Rat"),
                creature_type: CreatureType::Plague,
                damage: 1,
                health: 1,
            }
        }
    }

    struct SquirrelGenerator {}

    impl CreatureGen for SquirrelGenerator {
        fn gen(&self) -> Creature {
            Creature {
                name: String::from("Squirrel"),
                creature_type: CreatureType::Scavanger,
                damage: 1,
                health: 1,
            }
        }
    }

    struct MonkeyGenerator {}

    impl CreatureGen for MonkeyGenerator {
        fn gen(&self) -> Creature {
            Creature {
                name: String::from("Monkey"),
                creature_type: CreatureType::Destroyer,
                damage: 1,
                health: 1,
            }
        }
    }
}

pub mod instances {
    use crate::field::cards::Evokable;

    use super::cards::{CardType, Card, Generator, CreatureGen, Construct, ConstructType, Evocation, Creature};
    use super::internal_generators;

    fn simple_generator(
        name: String,
        description: String,
        internal_generator: Box<dyn CreatureGen>,
        cost: u32,
        gen_cost: u32
    ) -> Card {
        // Reduces boiler plate for simple generators
        let generator = Generator::new(gen_cost, internal_generator);
        Card {
            cost,
            name,
            description,
            content: CardType::Generator(generator)
        }
    }

    pub fn bird_generator() -> Card {
        let internal = internal_generators::BirdGenerator {};
        simple_generator(
            String::from("Bird Generator"),
            String::from("A simple bird generator."),
            Box::new(internal),
            1,
            1
        )
    }

    pub fn rat_generator() -> Card {
        let internal = internal_generators::BirdGenerator {};
        simple_generator(
            String::from("Rat Generator"),
            String::from("A simple rat generator."),
            Box::new(internal),
            1,
            1
        )
    }

    pub fn squirrel_generator() -> Card {
        let internal = internal_generators::BirdGenerator {};
        simple_generator(
            String::from("Squirrel Generator"),
            String::from("A simple squirrel generator."),
            Box::new(internal),
            1,
            1
        )
    }

    pub fn monkey_generator() -> Card {
        let internal = internal_generators::BirdGenerator {};
        simple_generator(
            String::from("Monkey Generator"),
            String::from("A simple monkey generator"),
            Box::new(internal),
            1,
            1
        )
    }

    pub fn flimsy_wall() -> Card {
        let construct = Construct::new(
            ConstructType::Wall { blocks_friendly: true },
            5,
            5,
            5
        );
        Card {
            name: String::from("Flimsy Wall"),
            description: String::from("A flimsy wall that should hold for a bit."),
            cost: 1,
            content: CardType::Construct(construct)
        }
    }

    pub fn ramp_wall() -> Card {
        let construct = Construct::new(
            ConstructType::Wall { blocks_friendly: false },
            5,
            5,
            5
        );
        Card {
            name: String::from("Ramp Wall"),
            description: String::from("A wall with a ramp on the friendly side to allow creatures to pass over"),
            cost: 1,
            content: CardType::Construct(construct)
        }
    }

    pub fn spikes() -> Card {
        let construct = Construct::new(
            ConstructType::Trap { damage: 3 }, 
            5,
            5,
            5
        );
        Card {
            name: String::from("Spikes"),
            description: String::from("Small spikes that will damage some passing creatures."),
            cost: 1,
            content: CardType::Construct(construct)
        }
    }

    pub fn lightning() -> Card {
        struct Lightning {}
        impl Evokable for Lightning {
            fn evoke(&self, creature: Creature) -> Creature {
               creature.inflict(1)
            }

            fn evoke_mut(&self, creature: &mut Creature) {
                creature.inflict_mut(1);
            }
        }

        let evocation = Evocation::new(
            super::cards::EvocationType::Damage { dmg: 1 }, 
            Box::new(Lightning {})
        );
        Card {
            name: String::from("Lightning"),
            description: String::from("Evoke lightning amongst all!"),
            cost: 1,
            content: CardType::Evocation(evocation)
        }
    }

    pub fn thunder() -> Card {
        struct Thunder {}
        impl Evokable for Thunder {
            fn evoke(&self, creature: Creature) -> Creature {
               creature.stun(1)
            }

            fn evoke_mut(&self, creature: &mut Creature) {
                creature.stun_mut(1);
            }
        }

        let evocation = Evocation::new(
            super::cards::EvocationType::Damage { dmg: 1 }, 
            Box::new(Lightning {})
        );
        Card {
            name: String::from("Lightning"),
            description: String::from("Evoke lightning amongst all!"),
            cost: 1,
            content: CardType::Evocation(evocation)
        }
    }
}
