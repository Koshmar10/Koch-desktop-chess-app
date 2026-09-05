use rusqlite::Row;

pub struct Opening {
    pub opening_name: String,
    pub uci: String,
}

impl TryFrom<&Row<'_>> for Opening {
    type Error = rusqlite::Error;

    fn try_from(row: &Row<'_>) -> Result<Self, Self::Error> {
        Ok(Self {
            opening_name: row.get("opening_name")?,
            uci: row.get("uci")?,
        })
    }
}
