// Notes:
// For hero/ card abilities, we should probably write all the systems into one file and then have
// the cards reference those systems.
//  E.g.: Dash IE's ability will be a system that runs at the start of the game. If a hero is
//  DashIE, they can choose an item. If not, the system continues. That could lead to cumbersome
//  checks though.
//  
//  Another example is Crank: How do we evaluate this? A card could have a keyword section, and
//  upon the event of a card entering the field, we could run the crank system

use std::{collections::VecDeque, ops::Sub, io};
use rand::Rng;

use bevy_ecs::prelude::*;

#[derive(Component)]
struct OnAttack(CardId);

#[derive(Component)]
struct OnHit(CardId);

// Cost to play card
#[derive(Component)]
struct Cost(u16);

// Card color
#[derive(Component)]
enum Color {
    Red,
    Yellow,
    Blue
}

impl Color {
    fn pitch(&self) -> u16 {
        match &self {
            Color::Red => 1,
            Color::Yellow => 2,
            Color::Blue => 3
        }
    }
}

#[derive(Component)]
struct GoAgain;

// Attack power
#[derive(Component)]
struct Attack(u16);

// Def
#[derive(Component)]
struct Defense(u16);

// Card Type
#[derive(Component, Eq, PartialEq, Debug)]
enum CardType {
    Action,
    Instant,
    Resource
}

impl CardType {
    fn is_action(&self) -> bool {
        *self == CardType::Action
    }

    fn is_playable(&self) -> bool{
        *self == CardType::Action
            || *self == CardType::Instant
    }
}

// Card Sub Type
#[derive(PartialEq, Eq)]
enum SubType {
    Attack,
}

// Card Sub Type Component
#[derive(Component, Default)]
struct CardSubTypes(Vec<SubType>);

impl CardSubTypes {
    fn requires_target(&self) -> bool {
        self.0.contains(&SubType::Attack)
    }

    fn has_attack(&self) -> bool {
        self.0.contains(&SubType::Attack)
    }
}

// Classes
#[derive(PartialEq, Eq)]
enum CardClassTypes {
    Assassin,
    Generic,
    Ranger,
}

// Card Class Options
#[derive(Component)]
enum CardClass {
    SingleClass(CardClassTypes),
    DualClass((CardClassTypes, CardClassTypes))
}

impl CardClass {
    fn contains(&self, card_class: CardClassTypes) -> bool {
        match &self {
            CardClass::SingleClass(class) => *class == card_class,
            CardClass::DualClass((class1, class2)) => 
                card_class == *class1 || card_class == *class2
        }
    }
}

// Card Name
#[derive(Component)]
struct CardName(String);

#[derive(Component)]
struct Hero;

#[derive(Component)]
struct GameEvent {
    target: Option<Entity>,
    card: Entity,
    actor: Entity,
    attack: bool
}

#[derive(Component)]
struct Intellect(u16);

impl Default for Intellect {
    fn default() -> Self {
        Intellect(4)
    }
}

#[derive(Component, Default)]
struct PitchZone(VecDeque<Entity>);

#[derive(Component, Default)]
struct HandZone(Vec<Entity>);

#[derive(Component, Default)]
struct Resources(u16);

#[derive(Component, Debug, Copy, Clone)]
struct Health(u16);

impl Sub for Health {
    type Output = Health;

    fn sub(self, rhs: Self) -> Self::Output {
        Health(self.0 - rhs.0)
    }
}


#[derive(Component)]
struct Life(u16);

#[derive(Component)]
struct Damage(u16);

#[derive(Component)]
enum HeroAge {
    Young,
    Adult
}

#[derive(Component)]
struct PlayerName(String);

impl PlayerName {
    fn from(string: &str) -> Self {
        PlayerName(String::from(string))
    }
}

#[derive(Component, Default)]
struct ActionPoints(u16);

#[derive(Bundle)]
struct HeroBundle {
    player_name: PlayerName,
    card_name: CardName,
    intellect: Intellect,
    health: Health,
    hero_class: CardClass,
    hero_age: HeroAge,
    pitch: PitchZone,
    hand: HandZone,
    resources: Resources,
    action_points: ActionPoints,
    hero: Hero
}

impl Default for HeroBundle {
    fn default() -> Self {
        HeroBundle {
            player_name: PlayerName(String::from("AI")),
            card_name: CardName(String::from("Gold Fish")),
            intellect: Intellect(4),
            health: Health(40),
            hero_class: CardClass::SingleClass(CardClassTypes::Generic),
            hero_age: HeroAge::Adult,
            pitch: PitchZone::default(),
            hand: HandZone::default(),
            resources: Resources::default(),
            action_points: ActionPoints::default(),
            hero: Hero
        }
    }
}

struct ChainLink {
    target: Entity,
    attacker: Entity,
    attack: Entity,
    blocks: Vec<Entity>,
    attack_reactions: Vec<Entity>,
    defense_reactions: Vec<Entity>,
    hit: bool,
    closed: bool
}

impl ChainLink {
    fn attack(target: Entity, attacker: Entity, attack: Entity) -> ChainLink {
        ChainLink {
            target,
            attacker,
            attack,
            blocks: Vec::new(),
            attack_reactions: Vec::new(),
            defense_reactions: Vec::new(),
            hit: false,
            closed: false
        }
    }
}

#[derive(Resource, Default)]
struct Chain {
    links: Vec<ChainLink>,
    open: bool
}

impl Chain {
    fn add_chain_link(&mut self, chain_link: ChainLink) {
        self.open = true;
        self.links.push(chain_link);
    }
}

#[derive(Resource, Default)]
struct Stack(VecDeque<GameEvent>);

impl Stack {
    fn is_empty(&self) -> bool {
        self.0.is_empty()
    }
}

#[derive(Resource, Default)]
struct AttackLayer(Option<GameEvent>);

#[derive(Component)]
struct Id(CardId);

#[derive(Eq, PartialEq, Hash)]
struct CardId(String);

#[derive(Resource, Default)]
struct Played(Option<Entity>);

#[derive(Resource, Hash, Eq, PartialEq, Clone, Debug, Default)]
struct GameState(GamePhases);

#[derive(Hash, Eq, PartialEq, Clone, Debug, Default)]
enum GamePhases {
    #[default]
    StartPhase,
    ActionPhase,
    EndPhase,
}

#[derive(Resource, Hash, Eq, PartialEq, Clone, Debug, Default)]
struct CombatState(Option<CombatSteps>);

#[derive(Hash, Eq, PartialEq, Clone, Debug, Default)]
enum CombatSteps {
    #[default]
    LayerStep,
    AttackStep,
    DefendStep,
    ReactionStep,
    DamageStep,
    ResolutionStep,
    LinkStep,
    CloseStep
}


#[derive(SystemSet, Hash, Eq, PartialEq, Clone, Debug)]
enum ScheduleSets {
    Read,
    Process,
    StartPhase,
    ActionPhase,
    EndPhase,
}

#[derive(Resource)]
struct TurnNumber(u16);

#[derive(Resource, Default)]
struct Priority {
    // Using a zipper struct for this
    holding: VecDeque<Entity>,
    passed: VecDeque<Entity>,

    // True when game is holding priority
    hold: bool,

    // This is hopefully a temporary hack :)
    // When true, priority is for blocks only
    blocks: bool,

    // More hacks!
    // If a player plays a card, priority should be reset at the end of the current cycle of
    // priorities
    card_played: bool

}

impl Priority {
    fn hold_priority(&mut self) {
        println!("Game is holding priority");
        self.hold = true
    }

    fn release_priority(&mut self) {
        println!("Game is releasing priority");
        self.hold = false
    }

    fn has_priority(&self, entity: &Entity) -> bool {
        self.holding
            .front()
            .map(|v| v == entity)
            .unwrap_or(false)
        && !self.blocks
    }

    fn is_blocking(&self, entity: &Entity) -> bool {
        self.holding
            .front()
            .map(|v| v == entity)
            .unwrap_or(false)
        && self.blocks
    }
    
    fn priority_hero(&self) -> Option<&Entity> {
        self.holding.front()
    }

    fn turn_player(&self) -> &Entity {
        if let Some(holding) = self.holding.front() {
            holding
        } else {
            self.passed
                .front()
                .expect("At least one player should exist")
        }
    }

    // Cycles priority and indicates if all players have passed
    fn pass_priority(&mut self) {
        if let Some(hero) = self.holding.pop_front() {
            self.passed.push_back(hero);
        }

        if self.passed.is_empty() && self.card_played {
            self.reset();
        }
    }

    // Checks that all players have passed priority
    fn all_passed(&self) -> bool {
        self.holding.is_empty() && !self.hold
    }

    fn cycle_priority(&mut self) -> &Self {
        self.reset();
        self.holding.rotate_left(1);
        self
    }

    fn reset(&mut self) -> &Self {
        // Shenanigans here:
        // To reset, we take those who passed and put them back in the front
        self.passed.append(&mut self.holding);
        self.holding = self.passed.drain(..).collect();

        self
    }

    fn someone_has_priority(&self) -> bool {
        !(self.holding.is_empty() || self.hold)
    }
}

#[derive(Event)]
struct PlayCard {
    hero: Entity,
    card: Entity,
    target: Option<Entity>
}

#[derive(Event)]
struct PitchCard {
    hero: Entity,
    card: Entity,
}

#[derive(Event)]
struct PassPriority {
    hero: Entity
}

#[derive(Event)]
struct DeclareBlocks {
    hero: Entity,
    blocks: Vec<Entity>
}

#[derive(Event)]
struct End; 

#[derive(Resource, Default)]
struct ProposedEvent(Option<GameEvent>);

mod read_systems {
    use super::*;

    pub fn read_card(
        target_query: Query<&CardName>,
        card_query: Query<(&CardName, &CardType, &CardSubTypes)>,
        mut priority: ResMut<Priority>,
        mut reader: EventReader<PlayCard>,
        mut proposed_event: ResMut<ProposedEvent>
    ) {
        for event in reader.read() {
            // Player can only play cards when they have priority
            if !priority.has_priority(&event.hero) {
                println!("Player does not have priority");
                return;
            }

            // Get card
            let (card_name, card_type, card_subtypes) = card_query.get(event.card).unwrap();

            // Check that card is playable
            if !card_type.is_playable() {
                println!("Card of type \"{:?}\" is not playable.", card_type);
                return;
            }

            if let Some(target) = event.target {
                let target_name = target_query.get(target).unwrap();
                println!("Card \"{}\" played, targeting \"{}\"", card_name.0, target_name.0);
            } else {
                println!("Card \"{}\" played", card_name.0);
            }

            if card_subtypes.requires_target() && event.target.is_none() {
                println!("Target needed");
                return;
            }

            proposed_event.0 = Some(
                GameEvent {
                    target: event.target,
                    card: event.card,
                    actor: event.hero,
                    attack: card_subtypes.has_attack(),
                }
            );
            priority.hold_priority();
        }
    }

    pub fn read_priority(
        query: Query<&PlayerName>,
        mut reader: EventReader<PassPriority>,
        mut priority: ResMut<Priority>
    ) {
        for event in reader.read() {
            // This should be relocated to somewhere better
            // Not sure where yet
            if !priority.has_priority(&event.hero) {
                println!("You do not have priority"); 
                return;
            }
            let player_name = query.get(event.hero).unwrap();
            println!("\"{}\" passed priority", player_name.0);
            priority.pass_priority();
            if priority.all_passed() {
                println!("All players passed priority");
            }
        }
    }

    pub fn read_pitch(
        mut reader: EventReader<PitchCard>,
        mut priority: ResMut<Priority>,
        proposed_event: Res<ProposedEvent>,
        mut hero_query: Query<(&mut HandZone, &mut PitchZone, &mut Resources)>,
        card_query: Query<(&CardName, &Color)>,
    ) {
        for event in reader.read() {
            if !priority.has_priority(&event.hero) {
                println!("Player does not have priority");
                return;
            }

            // Confident this is not a sufficient check
            // but should work for now
            if proposed_event.0.is_none() {
                println!("Cannot pitch to nothing");
                return;
            }

            let (card_name, color) = card_query.get(event.card).unwrap();
            println!("Card \"{}\" pitched for \"{}\"", card_name.0, color.pitch());

            // Make this a method of priority
            let (mut hand, mut pitch, mut resources) = hero_query
                .get_mut(event.hero)
                .expect("Invalid hero chosen");
            hand.0.retain(|c| *c != event.card);
            pitch.0.push_front(event.card);
            resources.0 += color.pitch();
            priority.hold_priority();
        }
    }

    pub fn read_blocks(
        mut reader: EventReader<DeclareBlocks>,
        mut chain: ResMut<Chain>,
        mut priority: ResMut<Priority>,
        card_query: Query<(&CardName, Option<&Defense>)>,
    ) {
        for event in reader.read() {
            if !priority.is_blocking(&event.hero) {
                println!("Player cannot block at this moment");
                return;
            }

            let mut blocks = Vec::new();
            for card in &event.blocks {
                if let Ok((card_name, defense)) = card_query.get(*card) {
                    if defense.is_none() {
                        println!("Card \"{}\" cannot block", card_name.0);
                        return;
                    } else {
                        blocks.push(*card);
                    }
                } else {
                    println!("Invalid entry declared for blocks");
                    return;
                }
            }

            chain.links
                .last_mut()
                .expect("Chain link missing")
                .blocks = blocks;
            
            // Hacky fix for progressing blocks
            priority.pass_priority();
        }
    }
}

mod game_systems {
    use super::*;

    pub fn evaluate_cost(
        cost_query: Query<(&CardName, &CardType, &Cost)>,
        mut resources_query: Query<(&mut Resources, &mut ActionPoints), With<Hero>>,
        mut proposed_event: ResMut<ProposedEvent>,
        mut priority: ResMut<Priority>,
        mut stack: ResMut<Stack>,
        mut attack_layer: ResMut<AttackLayer>,
    ) {
        // Check if card is being played
        if let Some(event) = &proposed_event.0 {
            // Get Details
            let (card_name, card_type, card_cost) = cost_query
                .get(event.card)
                .expect("Invalid card referenced");

            // Get resources and action points
            let priority_hero = priority.priority_hero();
            let (mut resources, mut action_points) = resources_query
                .get_mut(*priority_hero.unwrap())
                .expect("Heroes should have resources Component");

            // Check action points
            // This will obviously have to be changed for things like
            // 'Play next non-attack action as though it were an instant"
            if card_type.is_action() {
                if action_points.0 == 0 {
                    println!("Player does not have any action points.");
                    // Remove card from played card resource
                    proposed_event.0.take();
                    priority.release_priority();
                    return;
                }
            }

            // Check if cost is currently payable
            if resources.0 < card_cost.0 {
                let needed = card_cost.0 - resources.0;
                println!("Not enough resources. Player must pitch at least \"{}\" to play.", needed);
                priority.release_priority();
                return;
            }

            // Remove resources
            resources.0 -= card_cost.0;

            // This will obviously have to be changed for things like
            // 'Play next non-attack action as though it were an instant"
            if card_type.is_action() {
                action_points.0 -= 1;
            }

            // Add card to the stack
            let event = proposed_event.0.take().unwrap();
            if event.attack {
                attack_layer.0 = Some(event);
                priority.hold_priority();
            } else {
                stack.0.push_front(proposed_event.0.take().unwrap());
            }
            priority.card_played = true;

            println!("Card \"{}\" added to the stack", card_name.0);
            println!("\"{}\" floating", resources.0);
        }
    }

    // Maybe want to split this into a different function for triggering attack layer
    pub fn resolve_stack(
        card_query: Query<&CardSubTypes>,
        mut stack: ResMut<Stack>,
        mut combat_state: ResMut<CombatState>,
        priority: Res<Priority>
    ) {
        // Only begin resolving stack if all players have passed priority
        // And the stack is not empty
        if priority.all_passed() && !stack.0.is_empty() {
            let next = stack.0.pop_front().unwrap();
            if let Err(_) = card_query.get(next.card) {
                println!("Source on stack has ceased to exist.");
                if next.attack {
                    println!("Moving to Close Step");
                    combat_state.0 = Some(CombatSteps::CloseStep);
                }
                return;
            }
        }
    }
}

mod validation_systems {
    // use super::*;
}

trait Card {
    type Bundle: Bundle;
    fn card_id() -> CardId;
    fn card() -> Self::Bundle;
    fn add_systems(schedule: &mut Schedule);
}

mod card_systems {
    use super::*;

    #[derive(Component)]
    enum Until {
        EndOfTurn
    }

    pub struct ToxicityRed;

    impl Card for ToxicityRed {
        type Bundle = (CardName, Cost, Color, Defense, CardType, Id, GoAgain);

        fn card_id() -> CardId {
            CardId("OUT165".to_string())
        }

        fn card() -> Self::Bundle {
            (
                CardName("Toxicity".to_string()),
                Cost(0),
                Color::Red,
                Defense(2),
                CardType::Action,
                Id(Self::card_id()),
                GoAgain
            )
        }

        fn add_systems(schedule: &mut Schedule) {
            schedule.add_systems((
                Self::play,
                Self::on_attack,
                Self::on_hit
            ));
        }
    }

    impl ToxicityRed {
        fn play(
            played: ResMut<Played>,
            card_query: Query<Option<&Id>>,
            mut commands: Commands,
        ) {
            if played.is_changed() && played.0.is_some() {
                if let Ok(id_result) = card_query.get(played.0.unwrap()) {
                    if let Some(id) = id_result {
                        if id.0 == Self::card_id() {
                            commands.spawn((OnAttack(Self::card_id()), Until::EndOfTurn));
                        }
                    }
                }
            }
        }

        fn on_attack(
            attack_layer: Res<AttackLayer>,
            card_query: Query<&CardClass>,
            on_attack_query: Query<(Entity, &OnAttack)>,
            mut commands: Commands
        ) {
            if let Some((trigger_entity, _)) = on_attack_query.iter().find(|(_, trigger)| trigger.0 == Self::card_id()) {
                let card_type = card_query
                    .get(
                        attack_layer.0
                        .as_ref()
                        .expect("Attack should exist on OnAttack trigger")
                        .card
                    )
                    .expect("Attack unexpectedly ceased to exist.");

                    if card_type.contains(CardClassTypes::Assassin)
                        || card_type.contains(CardClassTypes::Ranger)
                    {
                        commands.spawn((OnHit(Self::card_id()), Until::EndOfTurn));
                        commands.entity(trigger_entity).despawn();
                        println!("Toxicity in effect.");
                    }
            }
        }

        fn on_hit(
            combat_state: Res<CombatState>,
            card_query: Query<(Entity, &OnHit)>,
            chain: Res<Chain>,
            mut target_query: Query<(&mut Health, Option<&Hero>, &CardName)>,
            mut commands: Commands
        ) {
            if combat_state.is_changed()
                && combat_state.0.is_some()
                && *combat_state.0.as_ref().unwrap() == CombatSteps::DamageStep
                && chain.links
                    .last()
                    .expect("Chain link ceased to exist unexpectedly.")
                    .hit
            {
                if let Some((entity, _)) = card_query
                    .iter()
                    .find(|(_, OnHit(card_id))| *card_id == Self::card_id())
                {
                    let target = chain.links
                        .last()
                        .expect("Chain link ceased to exist unexpectedly.")
                        .target;
                    if let Ok((mut health, hero, card_name)) = target_query.get_mut(target) {
                        if hero.is_some() {health.0 -= 3;}
                        println!("{} loses 3 life.", card_name.0);
                    }
                    commands.entity(entity).despawn();
                }
            }
        }
    }
}

mod combat_systems {
    use super::*;

    pub fn trigger_layer_step(
        mut attack_layer: ResMut<AttackLayer>,
        mut combat_state: ResMut<CombatState>,
        mut priority: ResMut<Priority>
    ) {
        // Layer step is triggered when an attack is added to the stack
        if !attack_layer.is_changed() || attack_layer.0.is_none(){
            return;
        }

        // Can only trigger layer step from no step or link step
        if !combat_state.0
            .as_ref()
            .map(|v| *v == CombatSteps::LinkStep)
            .unwrap_or(true)
        {
            println!("Attack incorrectly added to the stack");
            attack_layer.0.take();
            return;
        }

        // Switch to LayerStep
        println!("Moving to Layer Step");
        combat_state.0 = Some(CombatSteps::LayerStep);
        priority.release_priority();
    }

    pub fn trigger_attack_step(
        mut attack_layer: ResMut<AttackLayer>,
        mut combat_state: ResMut<CombatState>,
        mut chain: ResMut<Chain>,
        mut priority: ResMut<Priority>,
        target_query: Query<Entity>,
    ) {
        if combat_state.0 == Some(CombatSteps::LayerStep)
        && priority.is_changed()
        && priority.all_passed()
        {
            println!("Switching to Attack Step.");
            combat_state.0 = Some(CombatSteps::AttackStep);

            // Validate attack layer
            if attack_layer.0.is_none() {
                println!("Attack has ceased to exist. Moving to Close Step.");
                combat_state.0 = Some(CombatSteps::CloseStep);
                return;
            }

            // Remove attack from layer
            let attack = attack_layer.0.take().unwrap();
            
            // Check status of targets
            if attack.target.is_none()
                || target_query.get(attack.target.unwrap()).is_err()
            {
                println!("Invalid target. Moving to Close Step");
                combat_state.0 = Some(CombatSteps::CloseStep);
                return;
            }

            // Resolve Attack abilities
            // ... skipping for now ...

            // Add attack to the chain
            println!("Attack added to the chain");
            chain.add_chain_link(
                ChainLink::attack(
                    attack.target.unwrap(),
                    attack.actor,
                    attack.card
                )
            );

            // Turn player gains priority
            priority.reset();
        }
    }

    pub fn trigger_defend_step(
        chain: Res<Chain>,
        target_query: Query<Option<&Hero>>,
        stack: Res<Stack>,
        mut combat_state: ResMut<CombatState>,
        mut priority: ResMut<Priority>,
    ) {
        if combat_state.0 == Some(CombatSteps::AttackStep)
            && priority.is_changed()
            && priority.all_passed()
            && stack.is_empty()
        {
            println!("Switching to Defend Step.");
            combat_state.0 = Some(CombatSteps::DefendStep);
            priority.blocks = true;

            // Check if target is a hero
            // if not, no blocks are allowed
            let link = chain.links
                .last()
                .expect("Chain link ceased to exist during defense step");
            let target = target_query.get(link.target)
                .expect("Target ceased to exist during defense step");

            if target.is_none() {
                println!("Target is not a hero, so no blocks can be declared.");
            } else {
                priority.reset();
                priority.pass_priority();
            }
        }

        if combat_state.0 == Some(CombatSteps::DefendStep)
            && priority.is_changed()
            && priority.all_passed()
            && priority.blocks
        {
            println!("Blocks declared");
            priority.blocks = false;
            priority.reset();
        }
    }

    pub fn trigger_reaction_step(
        mut combat_state: ResMut<CombatState>,
        mut priority: ResMut<Priority>,
        stack: Res<Stack>
    ) {
        if combat_state.0 == Some(CombatSteps::DefendStep)
            && priority.is_changed()
            && priority.all_passed()
            && stack.is_empty()
        {
            println!("Moving to Reaction Step.");
            priority.reset();
            combat_state.0 = Some(CombatSteps::ReactionStep);
        }
    }

    pub fn trigger_damage_step(
        attack_query: Query<&Attack>,
        defense_query: Query<&Defense>,
        stack: Res<Stack>,
        mut defender_query: Query<(&CardName, &mut Health)>,
        mut combat_state: ResMut<CombatState>,
        mut priority: ResMut<Priority>,
        mut chain: ResMut<Chain>,
    ) {
        if combat_state.0 == Some(CombatSteps::ReactionStep)
            && priority.is_changed()
            && priority.all_passed()
            && stack.is_empty()
        {
            // Transition
            println!("Moving to Damage Step.");
            priority.hold_priority();
            combat_state.0 = Some(CombatSteps::DamageStep);

            // Calculate Damage
            let link = chain.links.last_mut().unwrap();
            let attack = attack_query.get(link.attack)
                .expect("Attack has ceased to exist during the damage step")
                .0;

            let mut total_defense = 0u16;
            for block in &link.blocks {
                if let Ok(defense) = defense_query.get(*block) {
                    total_defense += defense.0;
                }
            }
            for d_react in &link.defense_reactions {
                if let Ok(defense) = defense_query.get(*d_react) {
                    total_defense += defense.0;
                }
            }
            
            // Hit
            if attack >= total_defense {
                link.hit = true;
                // Something here to trigger hit effects
                let (name, mut health) = defender_query
                    .get_mut(link.target)
                    .expect("Target ceased to exist at damage step");
                let dmg = attack - total_defense;
                health.0 -= dmg;
                println!("{} taking {} damage, going to {}", name.0, dmg, health.0);
            }
        }
    }

    pub fn trigger_resolution_step(
        mut combat_state: ResMut<CombatState>,
        mut priority: ResMut<Priority>,
        mut chain: ResMut<Chain>,
    ) {
        if combat_state.0 == Some(CombatSteps::DamageStep)
        {
            // Change state
            println!("Moving to Resolution Step");
            combat_state.0 = Some(CombatSteps::ResolutionStep);

            // Close chain link
            let link = chain.links
                .last_mut()
                .expect("Chain link ceased to exist during resolution step.");
            link.closed = true;
            
            // Chain link resolution triggers here
            // ... skipping for now ...

            // Restore priority
            priority.reset();
            priority.release_priority();
        }
    }

    pub fn trigger_link_step(
        stack: Res<Stack>,
        mut combat_state: ResMut<CombatState>,
        mut priority: ResMut<Priority>,
    ) {
        if combat_state.0 == Some(CombatSteps::ResolutionStep)
            && priority.is_changed()
                && priority.all_passed()
                && stack.is_empty()
        {
            // Move to link step
            println!("Moving to Link Step");
            combat_state.0 = Some(CombatSteps::LinkStep);

            // Calculate go again
            // ... skipping for now ...

            // Reset priority
            priority.reset();
        }
    }

    pub fn trigger_close_step(
        stack: Res<Stack>,
        mut combat_state: ResMut<CombatState>,
        mut priority: ResMut<Priority>,
    ) {
        if combat_state.0 == Some(CombatSteps::LinkStep)
            && priority.is_changed()
                && priority.all_passed()
                && stack.is_empty()
        {
            // Move to close step
            println!("Moving to Close Step");
            combat_state.0 = Some(CombatSteps::CloseStep);

            // Chain close triggers
            // ... skipping for now ...

            // Reset priority
            priority.reset();
        }
    }
}

mod state_change_systems {
    use super::*;

    // For now, this does nothing.
    // In the future, we will query for start of start phase triggers
    pub fn start_start_phase(
        game_state: Res<GameState>
    ) {
        if game_state.is_changed()
            && game_state.0 == GamePhases::StartPhase
        {
            println!("Starting start phase");
        }
    }

    pub fn end_start_phase(
        stack: Res<Stack>,
        mut game_state: ResMut<GameState>
    ) {
        // Start phase ends when the stack is empty
        // No players get priority
        if game_state.0 == GamePhases::StartPhase && stack.0.is_empty() {
            game_state.0 = GamePhases::ActionPhase;

            println!("Ending start phase");
        }
    }

    pub fn start_action_phase(
        mut hero_query: Query<&mut ActionPoints, With<Hero>>,
        mut priority: ResMut<Priority>,
        game_state: Res<GameState>
    ) {
        if game_state.0 == GamePhases::ActionPhase
            && game_state.is_changed()
        {
            println!("Starting action phase");
            priority.cycle_priority();
            let turn_player = priority.turn_player();
            let mut ap = hero_query.get_mut(*turn_player).expect("Turn player should exist");

            // Give hero one action point
            ap.0 = 1;

        }
    }

    pub fn end_action_phase(
        mut hero_query: Query<&mut ActionPoints, With<Hero>>,
        stack: Res<Stack>,
        attack_layer: Res<AttackLayer>,
        chain: Res<Chain>,
        priority: Res<Priority>,
        mut game_state: ResMut<GameState>
    ) {
        // Action phase when the last player passes priority
        // and nothing is on the stack
        if stack.0.is_empty()
            && attack_layer.0.is_none()
            && priority.is_changed() && priority.all_passed()
            && !chain.open
            && game_state.0 == GamePhases::ActionPhase
        {
            let turn_player = priority.turn_player();
            let mut ap = hero_query
                .get_mut(*turn_player)
                .expect("Turn player should exist");

            // Set turn player action points to 0
            ap.0 = 0;

            game_state.0 = GamePhases::EndPhase;

            println!("Ending action phase");
        }
    }

    pub fn trigger_end_phase(
        mut game_state: ResMut<GameState>,
        mut combat_state: ResMut<CombatState>,
        stack: Res<Stack>,
        priority: Res<Priority>,
    ) {
        if game_state.0 == GamePhases::ActionPhase
            && combat_state.0 == Some(CombatSteps::CloseStep)
            && stack.is_empty()
                && priority.is_changed()
                && priority.all_passed()
        {
            game_state.0 = GamePhases::EndPhase;
            combat_state.0.take();
        }
    }

    // For now, this does nothing.
    // In the future, we will query for start of end phase triggers
    pub fn start_end_phase(game_state: Res<GameState>) {
        if game_state.0 == GamePhases::EndPhase 
            && game_state.is_changed()
        {
            println!("Starting end phase");
        }
    }

    pub fn end_end_phase(
        mut hero_query: Query<&mut Resources, With<Hero>>,
        priority: Res<Priority>,
        stack: Res<Stack>,
        mut game_state: ResMut<GameState>
    ) {
        // End phase ends when the stack is empty
        // No players get priority
        if game_state.0 == GamePhases::EndPhase && stack.0.is_empty() {
            let turn_player = priority.turn_player();
            let mut resources = hero_query
                .get_mut(*turn_player)
                .expect("Turn player should exist");
            // Set turn player resources to 0
            resources.0 = 0;

            game_state.0 = GamePhases::StartPhase;
            println!("Ending end phase");
        }
    }
}

mod start_up_systems {
    use super::*;

    pub fn roll_for_first(
        query: Query<(Entity, &PlayerName), With<Hero>>,
        mut priority: ResMut<Priority>,
    ) {
        let mut maxes: Vec<(Entity, &PlayerName, u32)> = Vec::new();
        let mut players: Vec<(Entity, &PlayerName)> = query.iter().collect();

        while maxes.len() == 0 {
            for (entity, player_name) in &players {
                let first_die = rand::thread_rng().gen_range(1..=6);
                let second_die = rand::thread_rng().gen_range(1..=6);
                let result = first_die + second_die;
                println!(
                    "\"{}\" rolled {} + {} = {}",
                    player_name.0,
                    first_die,
                    second_die,
                    result
                );
                maxes.push((*entity, player_name, result));
                let max = maxes.iter().map(|v| v.2).max().unwrap();
                maxes.retain(|v| v.2 == max);
            }
            if maxes.len() > 1 {
                println!("Rerolling ties");
                players = maxes.iter().map(|v| (v.0, v.1)).collect();
                maxes.clear();
            }
        }

        let turn_player = maxes.first().unwrap().0;
        for (entity, _) in query.iter().skip_while(|v| v.0 != turn_player) {
            priority.holding.push_back(entity);
        }
        for (entity, _) in query.iter().take_while(|v| v.0 != turn_player) {
            priority.holding.push_back(entity);
        }
        println!("Turn order {:?}", priority.holding);
    }
}


// #[derive(Debug)]
enum EventType {
    PlayCard(PlayCard),
    PassPriority(PassPriority),
    PitchCard(PitchCard),
    DeclareBlocks(DeclareBlocks),
    End
}

// Real dumbed down method to engage with the system
fn read_event_from_user(
) -> Result<EventType, String> {
    let mut buffer = String::new();
    io::stdin().read_line(&mut buffer)
        .map_err(|err| format!("IO error: {}", err))?;

    let buffer = buffer.trim();

    if buffer.to_lowercase().as_str() == "end" {
        return Ok(EventType::End);
    }

    // split command into pieces
    let mut pieces = buffer.split(" ");

    // get hero entity id
    let hero = pieces
        .next()
        .ok_or(String::from("No hero given"))?
        .parse::<u32>().map_err(|_| "Hero value not an int")?;
    let hero_entity = Entity::from_raw(hero);
    println!("Hero entity selected \"{}\"", hero_entity.index());

    // get event keyword
    let event = pieces.next()
        .ok_or("Event not specified")?;
    println!("Event selected \"{}\"", event);

    match event.to_lowercase().as_str().trim() {
        // Parse event to play card
        "play" => {
            // Parse card entity id
            let card = pieces.next()
                .ok_or("Card to play is not specified")?
                .parse::<u32>()
                .map_err(|_| String::from("Card must be an int"))?;
            let card_entity = Entity::from_raw(card);

            let target_entity = {
                if let Some(target_str) = pieces.next() {
                    println!("Target string \"{}\"", target_str);
                    let target = target_str.parse::<u32>()
                        .map_err(|_| String::from("Target must be int"))?;
                    Some(Entity::from_raw(target))
                } else { None }
            };
            Ok(EventType::PlayCard(
                PlayCard {
                    hero: hero_entity,
                    card: card_entity,
                    target: target_entity
                }
            ))
        },
        // Parse event to pass priority
        "pass" => Ok(
            EventType::PassPriority(
                PassPriority {hero: hero_entity}
            )
        ),
        "pitch" => {
            // Parse card entity id
            let card = pieces.next()
                .ok_or("Card to play is not specified")?
                .parse::<u32>()
                .map_err(|_| String::from("Card must be an int"))?;
            let card_entity = Entity::from_raw(card);

            Ok(EventType::PitchCard(
                PitchCard { hero: hero_entity, card: card_entity }
            ))
        },
        "block" => {
            // Parse card entities
            let cards = pieces
                .map(|p| {
                    p.parse::<u32>()
                    .map(|v| Entity::from_raw(v))
                    .map_err(|_|
                        String::from("Card must be an int")
                    )
                })
                .collect::<Result<Vec<Entity>, String>>()?;

            println!("{:?}", cards);

            Ok(EventType::DeclareBlocks(
                DeclareBlocks { hero: hero_entity, blocks: cards }
            ))
        },
        _ => Err(String::from("No Match"))
    }
}


fn main() {
    // Create a new empty World to hold our Entities and Components
    let mut world = World::new();
    // Events
    world.insert_resource(Events::<PlayCard>::default());
    world.insert_resource(Events::<PassPriority>::default());
    world.insert_resource(Events::<PitchCard>::default());
    world.insert_resource(Events::<DeclareBlocks>::default());

    // Resources
    world.insert_resource(AttackLayer::default());
    world.insert_resource(ProposedEvent::default());

    world.insert_resource(Priority::default());
    world.insert_resource(Stack::default());
    world.insert_resource(GameState::default());
    world.insert_resource(CombatState::default());
    world.insert_resource(Chain::default());
    world.insert_resource(Played::default());

    // Spawn entities
    let attack_card = world.spawn(
        (
            CardName(String::from("Basic Attack")),
            Cost(1),
            Attack(3),
            Defense(2),
            Color::Yellow,
            CardType::Action,
            CardSubTypes(vec![SubType::Attack]),
            CardClass::SingleClass(CardClassTypes::Generic)
        )
    ).id();
    println!("Attack card entity id {}", attack_card.index());

    let pitch_card = world.spawn(
        (
            CardName(String::from("Basic Resource")),
            Color::Yellow,
            CardType::Resource,
            CardClass::SingleClass(CardClassTypes::Generic),
            CardSubTypes::default(),
        )
    ).id();
    println!("Pitch card entity id {}", pitch_card.index());

    let hero1 = world.spawn(
        HeroBundle {
            player_name: PlayerName::from("Player 1"),
            ..Default::default()
        }
    ).id();
    println!("Hero 1 entity id {}", hero1.index());

    let hero2 = world.spawn(
        HeroBundle {
            player_name: PlayerName::from("Player 2"),
            ..Default::default()
        }
    ).id();
    println!("Hero 2 entity id {}", hero2.index());

    let toxicity_red = world.spawn(
        <card_systems::ToxicityRed as Card>::card()
    ).id();
    println!("Toxicity entity id {}", toxicity_red.index());


    // Create a new Schedule, which defines an execution strategy for Systems
    let mut schedule = Schedule::default();
    let mut start_up_schedule = Schedule::default();

    // Add systems to start up schedule
    start_up_schedule.add_systems(
        start_up_systems::roll_for_first
    );

    // Add systems to game schedule
    // Read Systems
    schedule.add_systems((
        read_systems::read_card.in_set(ScheduleSets::Read),
        read_systems::read_priority.in_set(ScheduleSets::Read),
        read_systems::read_pitch.in_set(ScheduleSets::Read),
        read_systems::read_blocks.in_set(ScheduleSets::Read),
    ));
    // Evaluate read systems
    schedule.add_systems(
        game_systems::evaluate_cost.in_set(ScheduleSets::Process),
    );
    schedule.add_systems((
        // Start phase triggers
        state_change_systems::start_start_phase.in_set(ScheduleSets::StartPhase),
        state_change_systems::end_start_phase
            .after(ScheduleSets::StartPhase)
            .before(ScheduleSets::ActionPhase),

        // Action phase triggers
        state_change_systems::start_action_phase.in_set(ScheduleSets::ActionPhase),

        // Combat triggers
        combat_systems::trigger_layer_step.after(ScheduleSets::ActionPhase),
        combat_systems::trigger_attack_step.after(ScheduleSets::ActionPhase),
        combat_systems::trigger_defend_step.after(ScheduleSets::ActionPhase),
        combat_systems::trigger_reaction_step.after(ScheduleSets::ActionPhase),
        combat_systems::trigger_damage_step.after(ScheduleSets::ActionPhase),
        combat_systems::trigger_resolution_step.after(ScheduleSets::ActionPhase),
        combat_systems::trigger_link_step.after(ScheduleSets::ActionPhase),
        combat_systems::trigger_close_step.after(ScheduleSets::ActionPhase),

        state_change_systems::end_action_phase
            .after(ScheduleSets::ActionPhase)
            .before(ScheduleSets::EndPhase),

        // End phase triggers
        state_change_systems::trigger_end_phase.in_set(ScheduleSets::EndPhase),
        state_change_systems::start_end_phase.in_set(ScheduleSets::EndPhase),
        state_change_systems::end_end_phase
            .after(ScheduleSets::EndPhase),

        // Misc
        game_systems::resolve_stack,
    ));

    <card_systems::ToxicityRed as Card>::add_systems(&mut schedule);

    // Initial runs
    start_up_schedule.run(&mut world);
    schedule.run(&mut world);

    // The idea is that the ECS will track game states for us based on updates
    // E.g. if a card is played, or an attack hits, run the rules to calculate
    // all the effects
    loop {
        if world.get_resource::<Priority>().unwrap().someone_has_priority() {
            let res = read_event_from_user();
            if let Ok(event) = res {
                match event {
                    EventType::PlayCard(event) => {
                        world.send_event(event);
                    },
                    EventType::PassPriority(event) => {
                        world.send_event(event);
                    }
                    EventType::PitchCard(event) => {
                        world.send_event(event);
                    }
                    EventType::DeclareBlocks(event) => {
                        world.send_event(event);
                    }
                    EventType::End => {break;}
                }
            } else { println!("{}", res.err().unwrap()); }
        }
        schedule.run(&mut world);
    }
}
