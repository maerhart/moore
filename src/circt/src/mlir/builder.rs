// Copyright (c) 2016-2021 Fabian Schuiki

//! A builder for IR operations.

use crate::crate_prelude::*;

/// A builder for MLIR operations.
pub struct Builder {
    /// The surrounding MLIR context.
    pub(crate) cx: Context,
    /// The location to assign to the operations being built.
    pub(crate) loc: Location,
    /// The block we're currently inserting into.
    insert_block: Option<MlirBlock>,
    /// The insertion position within the block.
    insert_point: InsertPoint,
    /// The last block that was inserted. Used to order created blocks in
    /// sequence if there are no intermittent `set_insertion_point_*` calls.
    insert_block_after: Option<MlirBlock>,
}

impl Builder {
    /// Create a new builder.
    pub fn new(cx: Context) -> Self {
        Self {
            cx,
            loc: Location::unknown(cx),
            insert_block: None,
            insert_point: InsertPoint::BlockStart,
            insert_block_after: None,
        }
    }

    /// Set the location assigned to new operations.
    pub fn set_loc(&mut self, loc: Location) {
        self.loc = loc;
    }

    /// Get the current location that is assigned to new operations.
    pub fn loc(&self) -> Location {
        self.loc
    }

    /// Set the insertion point to the start of a block.
    pub fn set_insertion_point_to_start(&mut self, block: MlirBlock) {
        self.insert_block = Some(block);
        self.insert_point = InsertPoint::BlockStart;
        self.insert_block_after = self.insert_block;
    }

    /// Set the insertion point to the end of a block.
    pub fn set_insertion_point_to_end(&mut self, block: MlirBlock) {
        self.insert_block = Some(block);
        self.insert_point = InsertPoint::BlockEnd;
        self.insert_block_after = self.insert_block;
    }

    /// Set the insertion point to before an operation.
    pub fn set_insertion_point_before(&mut self, op: impl OperationExt) {
        self.insert_block = Some(op.parent_block());
        self.insert_point = InsertPoint::Before(op.raw());
        self.insert_block_after = self.insert_block;
    }

    /// Set the insertion point to after an operation.
    pub fn set_insertion_point_after(&mut self, op: impl OperationExt) {
        self.insert_block = Some(op.parent_block());
        self.insert_point = InsertPoint::After(op.raw());
        self.insert_block_after = self.insert_block;
    }

    /// Insert an operation at the currently configured position.
    pub fn insert(&mut self, op: impl WrapRaw<RawType = MlirOperation>) {
        let null_op = MlirOperation {
            ptr: std::ptr::null_mut(),
        };
        let op = op.raw();
        let block = self.insert_block.expect("insertion block not set");
        unsafe {
            match self.insert_point {
                InsertPoint::BlockStart => mlirBlockInsertOwnedOperationAfter(block, null_op, op),
                InsertPoint::BlockEnd => mlirBlockInsertOwnedOperationBefore(block, null_op, op),
                InsertPoint::After(ref_op) => mlirBlockInsertOwnedOperationAfter(block, ref_op, op),
                InsertPoint::Before(ref_op) => {
                    mlirBlockInsertOwnedOperationBefore(block, ref_op, op)
                }
            }
        }
        self.insert_point = InsertPoint::After(op);
    }

    /// Build an operation through a callback that populates an
    /// `OperationState`.
    pub fn build_with<Op: OperationExt + Copy>(
        &mut self,
        with: impl FnOnce(&mut Builder, &mut OperationState),
    ) -> Op {
        let mut state = OperationState::new(Op::operation_name(), self.loc.raw());
        with(self, &mut state);
        let op = state.build();
        self.insert(op);
        op
    }

    /// Create a new block after the current one.
    pub fn add_block(&mut self) -> MlirBlock {
        let block = self.insert_block.expect("insertion block not set");
        let after = self.insert_block_after.expect("insertion block not set");
        unsafe {
            let new_block = mlirBlockCreate(0, [].as_ptr());
            mlirRegionInsertOwnedBlockAfter(mlirBlockGetParentRegion(block), after, new_block);
            self.insert_block_after = Some(new_block);
            new_block
        }
    }
}

enum InsertPoint {
    BlockStart,
    BlockEnd,
    After(MlirOperation),
    Before(MlirOperation),
}
