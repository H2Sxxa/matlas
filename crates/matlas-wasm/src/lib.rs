// WebAssembly bridge for the simulation.
//
// The core crate stays free of presentation concerns. This crate is the only place
// that knows about JavaScript: it exposes a single handle, hands out read-only views
// and accepts player intents. Every call either returns a view or a structured
// error, so the front end never reaches into the simulation.

use matlas::{
    entity::machines::{MachineError, MachineKind},
    game::{ClaimError, Game, PlaceError},
    node::{Direction, Pos},
};
use serde::{Deserialize, Serialize, de::DeserializeOwned};
use wasm_bindgen::prelude::*;

const SPLITMIX_INCREMENT: u64 = 0x9E37_79B9_7F4A_7C15;

// A failure the front end can react to. `code` is stable and meant to be matched,
// while `message` carries the underlying error rendered for a human.
#[derive(Debug, Serialize)]
struct ApiError {
    code: String,
    message: String,
}

// A run plus the presentation state that is not part of the simulation.
//
// Saving borrows the game so a snapshot never copies the grid. Loading rebuilds it,
// which is why the two sides are separate structs rather than one generic wrapper.
//
// A save travels as JSON text rather than as an object handed through the bridge.
// Storage is text, so this keeps the front end from encoding twice, and it keeps a
// save readable. It also stays clear of the one shape the object serialiser cannot
// express: the grid is a tagged enum whose variants hold optional buffers.
#[derive(Debug, Serialize)]
struct SaveRef<'a> {
    ticks: u64,
    game: &'a Game,
}

#[derive(Debug, Deserialize)]
struct Save {
    ticks: u64,
    game: Game,
}

fn raise(code: &str, message: impl std::fmt::Display) -> JsValue {
    let error = ApiError {
        code: code.to_string(),
        message: message.to_string(),
    };
    serde_wasm_bindgen::to_value(&error).expect("an error of two strings always serialises")
}

fn to_js<T: Serialize>(value: &T) -> Result<JsValue, JsValue> {
    serde_wasm_bindgen::to_value(value)
        .map_err(|error| raise("serialize", format!("cannot hand over the value: {error}")))
}

// Enums cross the boundary under their serialised names, so the front end and the
// simulation share one vocabulary instead of a second lookup table.
fn parse<T: DeserializeOwned>(name: &str, argument: &str) -> Result<T, JsValue> {
    serde_wasm_bindgen::from_value(JsValue::from_str(name)).map_err(|_| {
        raise(
            "unknown_variant",
            format!("{argument} = {name} is not a known name"),
        )
    })
}

fn place_error(error: PlaceError) -> JsValue {
    let code = match &error {
        PlaceError::Machine(MachineError::NoFreeSlot { .. }) => "machine_no_slot",
        PlaceError::Machine(MachineError::NotInStock { .. }) => "machine_not_in_stock",
        PlaceError::Graph(_) => "out_of_bounds",
    };
    raise(code, error)
}

fn claim_error(error: ClaimError) -> JsValue {
    let code = match &error {
        ClaimError::NoSuchOffer { .. } => "no_such_offer",
        ClaimError::Machine(MachineError::NoFreeSlot { .. }) => "machine_no_slot",
        ClaimError::Machine(MachineError::NotInStock { .. }) => "machine_not_in_stock",
    };
    raise(code, error)
}

fn splitmix64(state: &mut u64) -> u64 {
    *state = state.wrapping_add(SPLITMIX_INCREMENT);
    let mut mixed = *state;
    mixed = (mixed ^ (mixed >> 30)).wrapping_mul(0xBF58_476D_1CE4_E5B9);
    mixed = (mixed ^ (mixed >> 27)).wrapping_mul(0x94D0_49BB_1331_11EB);
    mixed ^ (mixed >> 31)
}

// Expands the small seed a player can type into the 16 bytes the simulation wants.
fn expand_seed(seed: u32) -> [u8; 16] {
    let mut state = u64::from(seed);
    let mut bytes = [0u8; 16];
    for chunk in bytes.chunks_mut(8) {
        chunk.copy_from_slice(&splitmix64(&mut state).to_le_bytes());
    }
    bytes
}

#[wasm_bindgen]
pub struct GameHandle {
    game: Game,
    ticks: u64,
}

#[wasm_bindgen]
impl GameHandle {
    // Starts a run with the starting kit already in stock.
    //
    // `extra_slots` is how many machines loot may hand out beyond the kit.
    #[wasm_bindgen(constructor)]
    pub fn new(seed: u32, width: usize, height: usize, extra_slots: usize) -> GameHandle {
        Self {
            game: Game::start(expand_seed(seed), (width, height), extra_slots),
            ticks: 0,
        }
    }

    #[wasm_bindgen(getter)]
    pub fn ticks(&self) -> u64 {
        self.ticks
    }

    // Read-only snapshot of the whole run.
    pub fn view(&self) -> Result<JsValue, JsValue> {
        to_js(&self.game.view())
    }

    // Advances the simulation by one step.
    pub fn step(&mut self) -> Result<(), JsValue> {
        self.game
            .tick()
            .map_err(|error| raise("graph", format!("tick rejected: {error:?}")))?;
        self.ticks += 1;
        Ok(())
    }

    // Advances the simulation by `steps`, stopping at the first rejection.
    pub fn advance(&mut self, steps: u32) -> Result<(), JsValue> {
        for _ in 0..steps {
            self.step()?;
        }
        Ok(())
    }

    // Places a machine from stock. Both names are the serialised enum names.
    pub fn place(
        &mut self,
        kind: &str,
        x: usize,
        y: usize,
        direction: &str,
    ) -> Result<(), JsValue> {
        let kind: MachineKind = parse(kind, "kind")?;
        let direction: Direction = parse(direction, "direction")?;
        self.game
            .place_machine(kind, Pos { x, y }, direction)
            .map(|_| ())
            .map_err(place_error)
    }

    // Removes the node at a cell and returns the machine it gave back, if any.
    pub fn remove(&mut self, x: usize, y: usize) -> Result<JsValue, JsValue> {
        let removed = self.game.remove_machine(&Pos { x, y });
        to_js(&removed)
    }

    // Claims a pending reward offer and returns the reward that was applied.
    pub fn claim(&mut self, index: usize) -> Result<JsValue, JsValue> {
        let reward = self.game.claim(index).map_err(claim_error)?;
        to_js(&reward)
    }

    // The whole run as JSON text, so the front end can keep it as a save.
    pub fn save(&self) -> Result<String, JsValue> {
        serde_json::to_string(&SaveRef {
            ticks: self.ticks,
            game: &self.game,
        })
        .map_err(|error| raise("save", format!("the run cannot be stored: {error}")))
    }

    // Rebuilds a run from the text `save` returned.
    pub fn load(value: &str) -> Result<GameHandle, JsValue> {
        let save: Save = serde_json::from_str(value)
            .map_err(|error| raise("load", format!("the save cannot be restored: {error}")))?;
        Ok(Self {
            game: save.game,
            ticks: save.ticks,
        })
    }
}
