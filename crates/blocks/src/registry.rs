use std::io::Cursor;

use ahash::AHashMap;
use libcraft_blocks_data::{RawBlockState, RawBlockStateProperties};
use once_cell::sync::Lazy;

use crate::block_data::BlockData;

/// A block state.
///
/// A block state is composed of:
/// * A _kind_, represented by the [`BlockKind`](crate::BlockKind)
/// enum. Each block kind corresponds to a Minecraft block, like "red wool"
/// or "chest."
/// * _Data_, or properties, represented by structs implementing the [`BlockData`](crate::BlockData)
/// trait. For example, a chest has a "type" property in its block data
/// that determines whether the chest is single or double.
#[derive(Debug, Copy, Clone, PartialEq, Eq, Hash, PartialOrd, Ord)]
#[repr(transparent)]
pub struct BlockState {
    id: u16,
}

impl BlockState {
    /// Gets this block as a struct implementing the [`BlockData`](crate::BlockData)
    /// interface.
    ///
    /// If this block is not an instance of `T`, then returns `None`.
    ///
    /// # Warning
    /// The returned `BlockData` is not linked with this `BlockState` instance.
    /// You need to call [`set_data`] to apply any changes made to the block data.
    pub fn data_as<T: BlockData>(self) -> Option<T> {
        T::from_raw(&self.raw().properties)
    }

    /// Applies the given `BlockData` to this block state.
    ///
    /// All property values in `data` override existing properties
    /// in `self`.
    pub fn set_data<T: BlockData>(&mut self, data: T) {
        let mut raw = self.raw().properties.clone();
        data.apply(&mut raw);
        if let Some(new_block) = Self::from_raw(&raw) {
            *self = new_block;
        }
    }

    /// Returns whether this is the default block state for
    /// the block kind.
    pub fn is_default(self) -> bool {
        self.raw().default
    }

    /// Gets the ID of this block state.
    ///
    /// Block state IDs are not stable between Minecraft versions.
    pub fn id(self) -> u16 {
        self.id
    }

    /// Creates a block state from an ID.
    /// Returns `None` if the ID is invalid.
    ///
    /// Block state IDs are not stable between Minecraft versions.
    pub fn from_id(id: u16) -> Option<Self> {
        let _state = REGISTRY.raw_state(id)?;
        Some(Self { id })
    }

    /// Determines whether this block state is valid.
    pub fn is_valid(self) -> bool {
        REGISTRY.raw_state(self.id).is_some()
    }

    /// Gets the raw block state for this block state.
    pub(crate) fn raw(&self) -> &RawBlockState {
        REGISTRY.raw_state(self.id).expect("bad block")
    }

    /// Creates a block state from its raw properties.
    pub(crate) fn from_raw(raw: &RawBlockStateProperties) -> Option<Self> {
        let id = REGISTRY.id_for_state(raw)?;
        Some(Self { id })
    }
}

static REGISTRY: Lazy<BlockRegistry> = Lazy::new(|| BlockRegistry::new());

struct BlockRegistry {
    states: Vec<RawBlockState>,
    id_mapping: AHashMap<RawBlockStateProperties, u16>,
}

impl BlockRegistry {
    fn new() -> Self {
        const DATA: &[u8] = include_bytes!("../assets/raw_block_states.bc.gz");
        let reader = flate2::bufread::GzDecoder::new(Cursor::new(DATA));
        let states: Vec<RawBlockState> =
            bincode::deserialize_from(reader).expect("malformed block state data");

        // Ensure that indexes match IDs.
        #[cfg(debug_assertions)]
        {
            for (index, state) in states.iter().enumerate() {
                assert_eq!(index, state.id as usize);
            }
        }

        let id_mapping = states
            .iter()
            .map(|state| (state.properties.clone(), state.id))
            .collect();

        Self { states, id_mapping }
    }

    fn raw_state(&self, id: u16) -> Option<&RawBlockState> {
        self.states.get(id as usize)
    }

    fn id_for_state(&self, state: &RawBlockStateProperties) -> Option<u16> {
        self.id_mapping.get(state).copied()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn block_registry_creates_successfully() {
        let _ = BlockRegistry::new();
    }
}
