use once_cell::sync::OnceCell;
use std::collections::HashMap;

pub struct SymbolTable {
    table: HashMap<String, u16>,
    next_address: u16,
}

impl SymbolTable {
    pub fn new() -> SymbolTable {
        static INITIAL_TABLE: OnceCell<HashMap<String, u16>> = OnceCell::new();
        #[cfg_attr(rustfmt, rustfmt_skip)]
        let table = INITIAL_TABLE.get_or_init(|| {
            let mut table = HashMap::new();
            table.insert("SP".to_owned(),     0x0000);
            table.insert("LCL".to_owned(),    0x0001);
            table.insert("ARG".to_owned(),    0x0002);
            table.insert("THIS".to_owned(),   0x0003);
            table.insert("THAT".to_owned(),   0x0004);
            table.insert("SCREEN".to_owned(), 0x4000);
            table.insert("KBD".to_owned(),    0x6000);
            for i in 0..16 {
                table.insert(format!("R{}", i), i);
            }
            return table;
        }).clone();
        SymbolTable {
            table,
            next_address: 0x0010,
        }
    }

    pub fn register(&mut self, symbol: &str, value: u16) -> bool {
        self.table.insert(symbol.to_owned(), value).is_none()
    }

    pub fn get_or_auto_register(&mut self, symbol: &str) -> u16 {
        let ret = self
            .table
            .entry(symbol.to_owned())
            .or_insert(self.next_address)
            .clone();
        if ret == self.next_address {
            self.next_address += 1;
        }
        ret
    }
}
