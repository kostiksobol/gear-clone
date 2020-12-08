use codec::{Encode, Decode};
use std::{rc::Rc, cell::RefCell, collections::HashMap};
use crate::program::ProgramId;

#[derive(Clone, Debug)]
pub enum Error {
    OutOfMemory,
    PageOccupied(PageNumber),
    InvalidFree(PageNumber),
}

#[derive(Clone, Copy, Debug, Decode, Encode, derive_more::From, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct PageNumber(u32);

impl PageNumber {
    pub fn raw(&self) -> u32 { self.0 }
}

#[derive(Clone, Debug, Default)]
pub struct Allocations(Rc<RefCell<HashMap<PageNumber, ProgramId>>>);

impl Allocations {
    pub fn new<I: IntoIterator<Item=(PageNumber, ProgramId)>>(items: I) -> Self {
        Self(Rc::new(RefCell::new(items.into_iter().collect::<HashMap<_, _, _>>())))
    }

    pub fn get(&self, page: PageNumber) -> Option<ProgramId> {
        self.0.borrow().get(&page).map(|pid| *pid)
    }

    pub fn occupied(&self, page: PageNumber) -> bool {
        self.0.borrow().contains_key(&page)
    }

    pub fn insert(&self, program_id: ProgramId, page: PageNumber) -> Result<(), Error> {
        if self.0.borrow().contains_key(&page) {
            return Err(Error::PageOccupied(page))
        }

        self.0.borrow_mut().insert(page, program_id);

        Ok(())
    }

    pub fn remove(&self, program_id: ProgramId, page: PageNumber) -> Result<(), Error> {
        if program_id != *self.0.borrow().get(&page).ok_or(Error::InvalidFree(page))? {
            return Err(Error::InvalidFree(page))
        }

        self.0.borrow_mut().remove(&page);

        Ok(())
    }

    pub fn drain(self) -> Vec<(PageNumber, ProgramId)> {
        self.0.borrow_mut().drain().collect::<Vec<_>>()
    }

    // TODO: optimize
    pub fn by_proram(&self, target_program_id: ProgramId) -> Vec<PageNumber> {
        self.0.borrow().iter()
            .filter_map(|(page, pid)| if *pid == target_program_id { Some(*page) } else { None })
            .collect()
    }

    pub fn clear(&self, program_id: ProgramId) {
        self.0.borrow_mut().retain(|_, pid| *pid != program_id);
    }

    pub fn len(&self) -> usize {
        self.0.borrow().len()
    }
}

#[derive(Clone)]
pub struct MemoryContext {
    program_id: ProgramId,
    wasm: wasmtime::Memory,
    allocations: Allocations,
    max_pages: PageNumber,
    static_pages: PageNumber,
}

impl MemoryContext {
    pub fn new(
        program_id: ProgramId,
        wasm_memory: wasmtime::Memory,
        allocations: Allocations,
        static_pages: PageNumber,
        max_pages: PageNumber,
    ) -> Self {
        Self { wasm: wasm_memory, program_id, allocations, static_pages, max_pages }
    }

    pub fn alloc(&self, pages: PageNumber) -> Result<PageNumber, Error> {
        // silly allocator, brute-forces fist continuous sector
        let mut candidate = self.static_pages.raw();
        let mut found = 0u32;

        while found < pages.raw() {
            if candidate + pages.raw() > self.max_pages.raw() {
                println!("candidate: {}, pages: {}, max_pages: {}", candidate, pages.raw(), self.max_pages.raw());
                return Err(Error::OutOfMemory);
            }

            if self.allocations.occupied((candidate + found).into()) {
                candidate += 1;
                found = 0;
                continue;
            }

            found += 1;
        }

        if candidate + found > self.wasm.size() {
            let extra_grow = candidate + found - self.wasm.size();
            self.wasm.grow(extra_grow).map_err(|_grow_err| Error::OutOfMemory)?;
        }

        for page_num in candidate..candidate+found {
            self.allocations.insert(self.program_id, page_num.into())?;
        }

        Ok(candidate.into())
    }

    pub fn free(&self, page: PageNumber) -> Result<(), Error> {
        if page < self.static_pages || page > self.max_pages {
            return Err(Error::InvalidFree(page));
        }

        self.allocations.remove(self.program_id, page)?;

        Ok(())
    }

    pub fn allocations(&self) -> &Allocations {
        &self.allocations
    }

    pub fn wasm(&self) -> &wasmtime::Memory {
        &self.wasm
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn new_test_memory(static_pages: u32, max_pages: u32) -> MemoryContext {
        use wasmtime::{Engine, Store, MemoryType, Memory as WasmMemory, Limits};

        let engine = Engine::default();
        let store = Store::new(&engine);

        let memory_ty = MemoryType::new(Limits::new(static_pages, Some(max_pages)));
        let memory = WasmMemory::new(&store, memory_ty);

        MemoryContext::new(
            0.into(),
            memory,
            Default::default(),
            static_pages.into(),
            max_pages.into(),
        )
    }

    #[test]
    fn smoky() {
        let mem = new_test_memory(16, 256);
        assert_eq!(mem.allocations().len(), 0);

        assert_eq!(mem.alloc(16.into()).expect("allocation failed"), 16.into());
        assert_eq!(mem.allocations().len(), 16);

        // there is a space for 14 more
        for _ in 0..14 { mem.alloc(16.into()).expect("allocation failed"); }

        // no more mem!
        assert!(mem.alloc(1.into()).is_err());

        // but we free some
        mem.free(137.into()).expect("free failed");

        // and now can allocate page that was freed
        assert_eq!(mem.alloc(1.into()).expect("allocation failed").raw(), 137);

        // if we have 2 in a row we can allocate even 2
        mem.free(117.into()).expect("free failed");
        mem.free(118.into()).expect("free failed");

        assert_eq!(mem.alloc(2.into()).expect("allocation failed").raw(), 117);

        // but if 2 are not in a row, bad luck
        mem.free(117.into()).expect("free failed");
        mem.free(158.into()).expect("free failed");

        assert!(mem.alloc(2.into()).is_err());
    }
}